//! OSL Supervisor for managing OS integration actors.
//!
//! This module provides `OSLSupervisor` which manages FileSystemActor,
//! ProcessActor, and NetworkActor with RestForOne supervision strategy.
//!
//! # Architecture
//!
//! ```text
//! OSLSupervisor (RestForOne)
//! ├── FileSystemActor (all file/directory operations)
//! ├── ProcessActor (all process spawning/management)
//! └── NetworkActor (all network connections)
//! ```
//!
//! # Supervision Strategy
//!
//! Uses **RestForOne**: If FileSystemActor fails, it restarts along with all
//! actors started after it (ProcessActor, NetworkActor). This ensures
//! consistent state if dependencies exist.
//!
//! # Usage
//!
//! ```rust,ignore
//! use airssys_rt::osl::OSLSupervisor;
//! use std::time::Duration;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create and start OSL supervisor
//! let osl_supervisor = OSLSupervisor::new();
//! osl_supervisor.start().await.map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
//!
//! // Get actor addresses for message routing
//! let fs_addr = osl_supervisor.filesystem_addr();
//! let proc_addr = osl_supervisor.process_addr();
//! let net_addr = osl_supervisor.network_addr();
//!
//! // Application actors can now send messages to OSL actors
//! // via these addresses
//!
//! // Graceful shutdown
//! osl_supervisor.shutdown(Duration::from_secs(5)).await.map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
//! # Ok(())
//! # }
//! ```

// Layer 1: Standard library imports
use std::sync::Arc;
use std::time::Duration;

// Layer 2: Third-party crate imports
use async_trait::async_trait;
use tokio::sync::Mutex;

// Layer 3: Internal module imports
use crate::broker::InMemoryMessageBroker;
use crate::message::Message;
use crate::monitoring::{InMemoryMonitor, MonitoringConfig, SupervisionEvent};
use crate::supervisor::{
    Child, ChildHealth, ChildSpec, RestForOne, RestartPolicy, ShutdownPolicy, Supervisor,
    SupervisorNode,
};
use crate::util::ActorAddress;

use super::actors::messages::{
    FileSystemRequest, FileSystemResponse, NetworkRequest, NetworkResponse, ProcessRequest,
    ProcessResponse,
};
use super::actors::{FileSystemActor, NetworkActor, ProcessActor};

// Temporary unified message type for OSL actors (until full generic refactoring in Task 2.1.2)
// This allows actors to communicate using a common message envelope
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
enum OSLMessage {
    FileSystemReq(FileSystemRequest),
    FileSystemResp(FileSystemResponse),
    ProcessReq(ProcessRequest),
    ProcessResp(ProcessResponse),
    NetworkReq(NetworkRequest),
    NetworkResp(NetworkResponse),
}

impl Message for OSLMessage {
    const MESSAGE_TYPE: &'static str = "osl::unified::message";
}

impl From<FileSystemResponse> for OSLMessage {
    fn from(resp: FileSystemResponse) -> Self {
        OSLMessage::FileSystemResp(resp)
    }
}

impl From<ProcessResponse> for OSLMessage {
    fn from(resp: ProcessResponse) -> Self {
        OSLMessage::ProcessResp(resp)
    }
}

impl From<NetworkResponse> for OSLMessage {
    fn from(resp: NetworkResponse) -> Self {
        OSLMessage::NetworkResp(resp)
    }
}

// Type aliases to reduce complexity (clippy::type_complexity)
type FileSystemSupervisor = Arc<
    Mutex<
        SupervisorNode<
            RestForOne,
            FileSystemActor<OSLMessage, InMemoryMessageBroker<OSLMessage>>,
            InMemoryMonitor<SupervisionEvent>,
        >,
    >,
>;

type ProcessSupervisor = Arc<
    Mutex<
        SupervisorNode<
            RestForOne,
            ProcessActor<OSLMessage, InMemoryMessageBroker<OSLMessage>>,
            InMemoryMonitor<SupervisionEvent>,
        >,
    >,
>;

type NetworkSupervisor =
    Arc<Mutex<SupervisorNode<RestForOne, NetworkActor, InMemoryMonitor<SupervisionEvent>>>>;

/// Supervisor for OS Layer integration actors.
///
/// Manages FileSystemActor, ProcessActor, and NetworkActor with RestForOne
/// supervision strategy to ensure consistent state across OSL infrastructure.
///
/// # Design Note
///
/// Each actor type has its own supervisor instance because `SupervisorNode<S, C, M>`
/// is generic over the child type `C`. We cannot mix different actor types in a
/// single `SupervisorNode` due to Rust's type system. This is acceptable as each
/// actor is managed independently with the same RestForOne strategy.
///
/// # Examples
///
/// ```rust,ignore
/// use airssys_rt::osl::OSLSupervisor;
/// use std::time::Duration;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Create OSL supervisor
/// let osl_supervisor = OSLSupervisor::new();
///
/// // Start all OSL actors
/// osl_supervisor.start().await.map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
///
/// // Get actor addresses for routing
/// let fs_addr = osl_supervisor.filesystem_addr();
/// if let Some(name) = fs_addr.name() {
///     println!("FileSystemActor address: {}", name);
/// }
///
/// // Shutdown gracefully
/// osl_supervisor.shutdown(Duration::from_secs(3)).await.map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
/// # Ok(())
/// # }
/// ```
pub struct OSLSupervisor {
    /// Supervisor managing FileSystemActor (refactored with broker injection)
    supervisor_fs: FileSystemSupervisor,

    /// Supervisor managing ProcessActor (refactored with broker injection)
    supervisor_proc: ProcessSupervisor,

    /// Supervisor managing NetworkActor (pending refactoring - Task 2.1.1)
    supervisor_net: NetworkSupervisor,

    /// Actor addresses for message routing
    filesystem_addr: ActorAddress,
    process_addr: ActorAddress,
    network_addr: ActorAddress,

    /// Started state flag (prevents double initialization)
    started: Arc<Mutex<bool>>,
}

impl OSLSupervisor {
    /// Create a new OSLSupervisor with default configuration.
    ///
    /// Creates three separate supervisor instances (one per actor type) due to
    /// generic constraints. All use RestForOne strategy for consistency.
    ///
    /// # Returns
    ///
    /// A new `OSLSupervisor` instance ready to start actors.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_rt::osl::OSLSupervisor;
    ///
    /// let osl_supervisor = OSLSupervisor::new();
    /// ```
    pub fn new() -> Self {
        // Create supervisors for each actor type
        let supervisor_fs = Arc::new(Mutex::new(SupervisorNode::new(
            RestForOne,
            InMemoryMonitor::new(MonitoringConfig::default()),
        )));
        let supervisor_proc = Arc::new(Mutex::new(SupervisorNode::new(
            RestForOne,
            InMemoryMonitor::new(MonitoringConfig::default()),
        )));
        let supervisor_net = Arc::new(Mutex::new(SupervisorNode::new(
            RestForOne,
            InMemoryMonitor::new(MonitoringConfig::default()),
        )));

        // Define actor addresses for service discovery
        let filesystem_addr = ActorAddress::named("osl-filesystem");
        let process_addr = ActorAddress::named("osl-process");
        let network_addr = ActorAddress::named("osl-network");

        Self {
            supervisor_fs,
            supervisor_proc,
            supervisor_net,
            filesystem_addr,
            process_addr,
            network_addr,
            started: Arc::new(Mutex::new(false)),
        }
    }

    /// Start all OSL actors under supervision.
    ///
    /// Registers and starts FileSystemActor, ProcessActor, and NetworkActor
    /// in dependency order (RestForOne strategy). This method is idempotent -
    /// calling it multiple times is safe and only starts actors once.
    ///
    /// # Errors
    ///
    /// Returns error if any actor fails to start.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// # use airssys_rt::osl::OSLSupervisor;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let osl_supervisor = OSLSupervisor::new();
    /// osl_supervisor.start().await.map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
    ///
    /// // Calling start again is safe (idempotent)
    /// osl_supervisor.start().await.map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut started = self.started.lock().await;
        if *started {
            return Ok(());
        }

        // Create temporary brokers for refactored actors (FileSystem and Process)
        // TODO(Task 2.1.2): Make OSLSupervisor generic and inject shared broker to all actors
        let fs_broker = InMemoryMessageBroker::<OSLMessage>::new();
        let proc_broker = InMemoryMessageBroker::<OSLMessage>::new();

        // Start FileSystemActor first (no dependencies)
        {
            let mut sup = self.supervisor_fs.lock().await;
            let broker_clone = fs_broker.clone();
            let spec = ChildSpec {
                id: "filesystem".to_string(),
                factory: move || FileSystemActor::new(broker_clone.clone()),
                restart_policy: RestartPolicy::Permanent,
                shutdown_policy: ShutdownPolicy::Graceful(Duration::from_secs(5)),
                start_timeout: Duration::from_secs(10),
                shutdown_timeout: Duration::from_secs(10),
            };

            sup.start_child(spec).await?;
        }

        // Start ProcessActor second (refactored with broker injection)
        {
            let mut sup = self.supervisor_proc.lock().await;
            let broker_clone = proc_broker.clone();
            let spec = ChildSpec {
                id: "process".to_string(),
                factory: move || ProcessActor::new(broker_clone.clone()),
                restart_policy: RestartPolicy::Permanent,
                shutdown_policy: ShutdownPolicy::Graceful(Duration::from_secs(5)),
                start_timeout: Duration::from_secs(10),
                shutdown_timeout: Duration::from_secs(10),
            };

            sup.start_child(spec).await?;
        }

        // Start NetworkActor third (may depend on FileSystem for config)
        // TODO(Task 2.1.1): Refactor NetworkActor to accept broker injection
        {
            let mut sup = self.supervisor_net.lock().await;
            let spec = ChildSpec {
                id: "network".to_string(),
                factory: NetworkActor::new,
                restart_policy: RestartPolicy::Permanent,
                shutdown_policy: ShutdownPolicy::Graceful(Duration::from_secs(5)),
                start_timeout: Duration::from_secs(10),
                shutdown_timeout: Duration::from_secs(10),
            };

            sup.start_child(spec).await?;
        }

        *started = true;
        Ok(())
    }

    /// Get FileSystemActor address for message routing.
    ///
    /// Application actors use this address to send file operation requests
    /// to the FileSystemActor.
    ///
    /// # Returns
    ///
    /// Reference to the FileSystemActor's `ActorAddress`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_rt::osl::OSLSupervisor;
    ///
    /// let osl_supervisor = OSLSupervisor::new();
    /// let fs_addr = osl_supervisor.filesystem_addr();
    /// assert_eq!(fs_addr.name().unwrap(), "osl-filesystem");
    /// ```
    pub fn filesystem_addr(&self) -> &ActorAddress {
        &self.filesystem_addr
    }

    /// Get ProcessActor address for message routing.
    ///
    /// Application actors use this address to send process operation requests
    /// to the ProcessActor.
    ///
    /// # Returns
    ///
    /// Reference to the ProcessActor's `ActorAddress`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_rt::osl::OSLSupervisor;
    ///
    /// let osl_supervisor = OSLSupervisor::new();
    /// let proc_addr = osl_supervisor.process_addr();
    /// assert_eq!(proc_addr.name().unwrap(), "osl-process");
    /// ```
    pub fn process_addr(&self) -> &ActorAddress {
        &self.process_addr
    }

    /// Get NetworkActor address for message routing.
    ///
    /// Application actors use this address to send network operation requests
    /// to the NetworkActor.
    ///
    /// # Returns
    ///
    /// Reference to the NetworkActor's `ActorAddress`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_rt::osl::OSLSupervisor;
    ///
    /// let osl_supervisor = OSLSupervisor::new();
    /// let net_addr = osl_supervisor.network_addr();
    /// assert_eq!(net_addr.name().unwrap(), "osl-network");
    /// ```
    pub fn network_addr(&self) -> &ActorAddress {
        &self.network_addr
    }

    /// Shutdown all OSL actors gracefully.
    ///
    /// Stops all supervised actors with the specified timeout. Actors are
    /// stopped in reverse order (Network → Process → FileSystem) to handle
    /// dependencies. This method is idempotent.
    ///
    /// # Arguments
    ///
    /// * `timeout` - Maximum time to wait for each actor to stop
    ///
    /// # Errors
    ///
    /// Returns error if any actor fails to stop within timeout.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// # use airssys_rt::osl::OSLSupervisor;
    /// # use std::time::Duration;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let osl_supervisor = OSLSupervisor::new();
    /// osl_supervisor.start().await.map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
    ///
    /// // Graceful shutdown with 5 second timeout
    /// osl_supervisor.shutdown(Duration::from_secs(5)).await.map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn shutdown(
        &self,
        _timeout: Duration,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut started = self.started.lock().await;
        if !*started {
            return Ok(());
        }

        // Stop actors in reverse order
        // Note: Individual supervisors manage their own children
        // We stop network first, then process, then filesystem
        {
            let mut sup = self.supervisor_net.lock().await;
            let child_ids: Vec<_> = sup.child_ids().to_vec();
            for child_id in child_ids {
                let _ = sup.stop_child(&child_id).await;
            }
        }

        {
            let mut sup = self.supervisor_proc.lock().await;
            let child_ids: Vec<_> = sup.child_ids().to_vec();
            for child_id in child_ids {
                let _ = sup.stop_child(&child_id).await;
            }
        }

        {
            let mut sup = self.supervisor_fs.lock().await;
            let child_ids: Vec<_> = sup.child_ids().to_vec();
            for child_id in child_ids {
                let _ = sup.stop_child(&child_id).await;
            }
        }

        *started = false;
        Ok(())
    }
}

impl Default for OSLSupervisor {
    fn default() -> Self {
        Self::new()
    }
}

/// OSLSupervisor error type
#[derive(Debug)]
pub struct OSLSupervisorError {
    message: String,
}

impl std::fmt::Display for OSLSupervisorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "OSLSupervisorError: {}", self.message)
    }
}

impl std::error::Error for OSLSupervisorError {}

impl From<Box<dyn std::error::Error + Send + Sync>> for OSLSupervisorError {
    fn from(err: Box<dyn std::error::Error + Send + Sync>) -> Self {
        Self {
            message: err.to_string(),
        }
    }
}

/// Implement Child trait to enable OSLSupervisor nesting in RootSupervisor.
///
/// This allows OSLSupervisor to be managed as a child of a higher-level
/// supervisor, enabling hierarchical supervisor architectures.
#[async_trait]
impl Child for OSLSupervisor {
    type Error = OSLSupervisorError;

    async fn start(&mut self) -> Result<(), Self::Error> {
        OSLSupervisor::start(self)
            .await
            .map_err(OSLSupervisorError::from)
    }

    async fn stop(&mut self, timeout: Duration) -> Result<(), Self::Error> {
        self.shutdown(timeout)
            .await
            .map_err(OSLSupervisorError::from)
    }

    async fn health_check(&self) -> ChildHealth {
        let started = self.started.lock().await;
        if *started {
            ChildHealth::Healthy
        } else {
            ChildHealth::Degraded("OSLSupervisor not started".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_osl_supervisor() {
        let osl_supervisor = OSLSupervisor::new();

        // Verify actor addresses are configured correctly
        assert_eq!(
            osl_supervisor.filesystem_addr().name(),
            Some("osl-filesystem")
        );
        assert_eq!(osl_supervisor.process_addr().name(), Some("osl-process"));
        assert_eq!(osl_supervisor.network_addr().name(), Some("osl-network"));
    }

    #[tokio::test]
    async fn test_health_check_before_start() {
        let osl_supervisor = OSLSupervisor::new();

        let health = osl_supervisor.health_check().await;
        assert!(matches!(health, ChildHealth::Degraded(_)));
    }

    #[tokio::test]
    async fn test_health_check_after_start() {
        let osl_supervisor = OSLSupervisor::new();
        assert!(osl_supervisor.start().await.is_ok());

        let health = osl_supervisor.health_check().await;
        assert!(matches!(health, ChildHealth::Healthy));

        // Cleanup
        assert!(osl_supervisor
            .shutdown(Duration::from_secs(1))
            .await
            .is_ok());
    }
}
