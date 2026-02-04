//! Single child builder for fluent supervisor child configuration.

use std::time::Duration;

use crate::monitoring::{Monitor, SupervisionEvent};
use crate::supervisor::traits::Supervisor;
use crate::supervisor::{
    Child, ChildId, ChildSpec, RestartPolicy, ShutdownPolicy, SupervisionStrategy, SupervisorError,
    SupervisorNode,
};

use super::constants::{
    DEFAULT_RESTART_POLICY, DEFAULT_SHUTDOWN_POLICY, DEFAULT_SHUTDOWN_TIMEOUT,
    DEFAULT_START_TIMEOUT,
};

/// Fluent builder for configuring a single supervised child.
///
/// This builder provides an ergonomic alternative to manual [`ChildSpec`] construction,
/// with sensible defaults for production use. All configuration is optional except the
/// factory function.
///
/// # Design Philosophy
///
/// - **Progressive Disclosure**: Simple cases use minimal configuration
/// - **Sensible Defaults**: Production-safe defaults for all optional settings
/// - **Full Customization**: All `ChildSpec` fields are configurable
/// - **Type Safety**: Compile-time validation of configuration
/// - **Zero Overhead**: No runtime cost compared to manual `ChildSpec`
///
/// # Examples
///
/// ## Minimal Configuration
///
/// ```rust,no_run
/// use airssys_rt::supervisor::*;
/// use airssys_rt::monitoring::{NoopMonitor, SupervisionEvent};
/// use async_trait::async_trait;
/// use std::time::Duration;
///
/// # #[derive(Clone)]
/// # struct MyWorker;
/// # #[async_trait]
/// # impl Child for MyWorker {
/// #     type Error = std::io::Error;
/// #     async fn start(&mut self) -> Result<(), Self::Error> { Ok(()) }
/// #     async fn stop(&mut self, _: Duration) -> Result<(), Self::Error> { Ok(()) }
/// # }
/// # async fn example() -> Result<(), SupervisorError> {
/// let mut supervisor = SupervisorNode::new(OneForOne, NoopMonitor::<SupervisionEvent>::new());
///
/// // Uses all defaults: Permanent restart, 5s graceful shutdown, 30s start timeout
/// let id = supervisor
///     .child("worker")
///     .factory(|| MyWorker)
///     .spawn()
///     .await?;
/// # Ok(())
/// # }
/// ```
///
/// ## Full Customization
///
/// ```rust,no_run
/// use airssys_rt::supervisor::*;
/// use airssys_rt::monitoring::{NoopMonitor, SupervisionEvent};
/// use async_trait::async_trait;
/// use std::time::Duration;
///
/// # #[derive(Clone)]
/// # struct MyWorker;
/// # #[async_trait]
/// # impl Child for MyWorker {
/// #     type Error = std::io::Error;
/// #     async fn start(&mut self) -> Result<(), Self::Error> { Ok(()) }
/// #     async fn stop(&mut self, _: Duration) -> Result<(), Self::Error> { Ok(()) }
/// # }
/// # async fn example() -> Result<(), SupervisorError> {
/// let mut supervisor = SupervisorNode::new(OneForOne, NoopMonitor::<SupervisionEvent>::new());
///
/// let id = supervisor
///     .child("critical")
///     .factory(|| MyWorker)
///     .restart_transient()                      // Custom restart policy
///     .shutdown_graceful(Duration::from_secs(15)) // Custom shutdown
///     .start_timeout(Duration::from_secs(60))    // Custom start timeout
///     .shutdown_timeout(Duration::from_secs(20)) // Custom shutdown timeout
///     .spawn()
///     .await?;
/// # Ok(())
/// # }
/// ```
///
/// ## Using Default Factory
///
/// ```rust,no_run
/// use airssys_rt::supervisor::*;
/// use airssys_rt::monitoring::{NoopMonitor, SupervisionEvent};
/// use async_trait::async_trait;
/// use std::time::Duration;
///
/// # #[derive(Clone, Default)]
/// # struct MyWorker;
/// # #[async_trait]
/// # impl Child for MyWorker {
/// #     type Error = std::io::Error;
/// #     async fn start(&mut self) -> Result<(), Self::Error> { Ok(()) }
/// #     async fn stop(&mut self, _: Duration) -> Result<(), Self::Error> { Ok(()) }
/// # }
/// # async fn example() -> Result<(), SupervisorError> {
/// let mut supervisor: SupervisorNode<OneForOne, MyWorker, _> =
///     SupervisorNode::new(OneForOne, NoopMonitor::<SupervisionEvent>::new());
///
/// // For types implementing Default
/// let id = supervisor
///     .child("worker")
///     .factory_default::<MyWorker>()
///     .spawn()
///     .await?;
/// # Ok(())
/// # }
/// ```
pub struct SingleChildBuilder<'a, S, C, M>
where
    S: SupervisionStrategy,
    C: Child,
    M: Monitor<SupervisionEvent>,
{
    supervisor: &'a mut SupervisorNode<S, C, M>,
    id: String,
    factory: Option<Box<dyn Fn() -> C + Send + Sync>>,
    restart_policy: Option<RestartPolicy>,
    shutdown_policy: Option<ShutdownPolicy>,
    start_timeout: Option<Duration>,
    shutdown_timeout: Option<Duration>,
}

impl<'a, S, C, M> SingleChildBuilder<'a, S, C, M>
where
    S: SupervisionStrategy,
    C: Child,
    M: Monitor<SupervisionEvent> + 'static,
{
    /// Creates a new builder for a child with the given ID.
    ///
    /// This is typically called via [`SupervisorNode::child`] rather than directly.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use airssys_rt::supervisor::*;
    /// use airssys_rt::monitoring::{NoopMonitor, SupervisionEvent};
    /// use async_trait::async_trait;
    /// use std::time::Duration;
    ///
    /// # #[derive(Clone)]
    /// # struct MyWorker;
    /// # #[async_trait]
    /// # impl Child for MyWorker {
    /// #     type Error = std::io::Error;
    /// #     async fn start(&mut self) -> Result<(), Self::Error> { Ok(()) }
    /// #     async fn stop(&mut self, _: Duration) -> Result<(), Self::Error> { Ok(()) }
    /// # }
    /// # async fn example() -> Result<(), SupervisorError> {
    /// let mut supervisor: SupervisorNode<OneForOne, MyWorker, _> =
    ///     SupervisorNode::new(OneForOne, NoopMonitor::<SupervisionEvent>::new());
    ///
    /// // Preferred: via SupervisorNode method
    /// let builder = supervisor.child("worker");
    /// # Ok(())
    /// # }
    /// ```
    pub(crate) fn new(supervisor: &'a mut SupervisorNode<S, C, M>, id: String) -> Self {
        Self {
            supervisor,
            id,
            factory: None,
            restart_policy: None,
            shutdown_policy: None,
            start_timeout: None,
            shutdown_timeout: None,
        }
    }

    // -------------------------------------------------------------------------
    // Factory Configuration
    // -------------------------------------------------------------------------

    /// Sets the factory function that creates child instances.
    ///
    /// The factory must be `Fn() -> C + Send + Sync` to support restarts. It will be called
    /// each time the child needs to be (re)started.
    ///
    /// **Required**: This method must be called before `spawn()` or `build()`.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use airssys_rt::supervisor::*;
    /// use airssys_rt::monitoring::{NoopMonitor, SupervisionEvent};
    /// use async_trait::async_trait;
    /// use std::time::Duration;
    ///
    /// # #[derive(Clone)]
    /// # struct MyWorker;
    /// # impl MyWorker { fn new() -> Self { Self } }
    /// # #[async_trait]
    /// # impl Child for MyWorker {
    /// #     type Error = std::io::Error;
    /// #     async fn start(&mut self) -> Result<(), Self::Error> { Ok(()) }
    /// #     async fn stop(&mut self, _: Duration) -> Result<(), Self::Error> { Ok(()) }
    /// # }
    /// # async fn example() -> Result<(), SupervisorError> {
    /// let mut supervisor = SupervisorNode::new(OneForOne, NoopMonitor::<SupervisionEvent>::new());
    ///
    /// let id = supervisor
    ///     .child("worker")
    ///     .factory(|| MyWorker::new())
    ///     .spawn()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn factory(mut self, f: impl Fn() -> C + Send + Sync + 'static) -> Self {
        self.factory = Some(Box::new(f));
        self
    }

    /// Sets the factory to use `Default::default()` for child creation.
    ///
    /// Convenience method for types implementing [`Default`]. Equivalent to
    /// `.factory(|| T::default())` but more concise.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use airssys_rt::supervisor::*;
    /// use airssys_rt::monitoring::{NoopMonitor, SupervisionEvent};
    /// use async_trait::async_trait;
    /// use std::time::Duration;
    ///
    /// # #[derive(Clone, Default)]
    /// # struct MyWorker;
    /// # #[async_trait]
    /// # impl Child for MyWorker {
    /// #     type Error = std::io::Error;
    /// #     async fn start(&mut self) -> Result<(), Self::Error> { Ok(()) }
    /// #     async fn stop(&mut self, _: Duration) -> Result<(), Self::Error> { Ok(()) }
    /// # }
    /// # async fn example() -> Result<(), SupervisorError> {
    /// let mut supervisor: SupervisorNode<OneForOne, MyWorker, _> =
    ///     SupervisorNode::new(OneForOne, NoopMonitor::<SupervisionEvent>::new());
    ///
    /// let id = supervisor
    ///     .child("worker")
    ///     .factory_default::<MyWorker>()
    ///     .spawn()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn factory_default<T>(self) -> Self
    where
        T: Default + Into<C> + 'static,
    {
        self.factory(|| T::default().into())
    }

    // -------------------------------------------------------------------------
    // Restart Policy Configuration
    // -------------------------------------------------------------------------

    /// Sets the restart policy to [`RestartPolicy::Permanent`].
    ///
    /// Permanent children are always restarted when they terminate, regardless
    /// of exit reason. This is the default and most common setting for production.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use airssys_rt::supervisor::*;
    /// use airssys_rt::monitoring::{NoopMonitor, SupervisionEvent};
    /// use async_trait::async_trait;
    /// use std::time::Duration;
    ///
    /// # #[derive(Clone)]
    /// # struct MyWorker;
    /// # #[async_trait]
    /// # impl Child for MyWorker {
    /// #     type Error = std::io::Error;
    /// #     async fn start(&mut self) -> Result<(), Self::Error> { Ok(()) }
    /// #     async fn stop(&mut self, _: Duration) -> Result<(), Self::Error> { Ok(()) }
    /// # }
    /// # async fn example() -> Result<(), SupervisorError> {
    /// let mut supervisor = SupervisorNode::new(OneForOne, NoopMonitor::<SupervisionEvent>::new());
    ///
    /// let id = supervisor
    ///     .child("worker")
    ///     .factory(|| MyWorker)
    ///     .restart_permanent()  // Explicit (same as default)
    ///     .spawn()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn restart_permanent(mut self) -> Self {
        self.restart_policy = Some(RestartPolicy::Permanent);
        self
    }

    /// Sets the restart policy to [`RestartPolicy::Transient`].
    ///
    /// Transient children are restarted only if they terminate abnormally
    /// (with an error). Normal termination does not trigger a restart.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use airssys_rt::supervisor::*;
    /// use airssys_rt::monitoring::{NoopMonitor, SupervisionEvent};
    /// use async_trait::async_trait;
    /// use std::time::Duration;
    ///
    /// # #[derive(Clone)]
    /// # struct MyTask;
    /// # #[async_trait]
    /// # impl Child for MyTask {
    /// #     type Error = std::io::Error;
    /// #     async fn start(&mut self) -> Result<(), Self::Error> { Ok(()) }
    /// #     async fn stop(&mut self, _: Duration) -> Result<(), Self::Error> { Ok(()) }
    /// # }
    /// # async fn example() -> Result<(), SupervisorError> {
    /// let mut supervisor = SupervisorNode::new(OneForOne, NoopMonitor::<SupervisionEvent>::new());
    ///
    /// // Worker that might exit normally on completion
    /// let id = supervisor
    ///     .child("task")
    ///     .factory(|| MyTask)
    ///     .restart_transient()  // Only restart on error
    ///     .spawn()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn restart_transient(mut self) -> Self {
        self.restart_policy = Some(RestartPolicy::Transient);
        self
    }

    /// Sets the restart policy to [`RestartPolicy::Temporary`].
    ///
    /// Temporary children are never restarted, regardless of how they terminate.
    /// Use for one-shot tasks or children that should not be automatically
    /// recovered.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use airssys_rt::supervisor::*;
    /// use airssys_rt::monitoring::{NoopMonitor, SupervisionEvent};
    /// use async_trait::async_trait;
    /// use std::time::Duration;
    ///
    /// # #[derive(Clone)]
    /// # struct MyInitTask;
    /// # #[async_trait]
    /// # impl Child for MyInitTask {
    /// #     type Error = std::io::Error;
    /// #     async fn start(&mut self) -> Result<(), Self::Error> { Ok(()) }
    /// #     async fn stop(&mut self, _: Duration) -> Result<(), Self::Error> { Ok(()) }
    /// # }
    /// # async fn example() -> Result<(), SupervisorError> {
    /// let mut supervisor = SupervisorNode::new(OneForOne, NoopMonitor::<SupervisionEvent>::new());
    ///
    /// // One-shot initialization task
    /// let id = supervisor
    ///     .child("init")
    ///     .factory(|| MyInitTask)
    ///     .restart_temporary()  // Never restart
    ///     .spawn()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn restart_temporary(mut self) -> Self {
        self.restart_policy = Some(RestartPolicy::Temporary);
        self
    }

    /// Sets a custom restart policy.
    ///
    /// For most cases, use the shortcut methods ([`restart_permanent`],
    /// [`restart_transient`], [`restart_temporary`]) instead. This method
    /// is provided for completeness and future restart policy extensions.
    ///
    /// [`restart_permanent`]: Self::restart_permanent
    /// [`restart_transient`]: Self::restart_transient
    /// [`restart_temporary`]: Self::restart_temporary
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use airssys_rt::supervisor::*;
    /// use airssys_rt::monitoring::{NoopMonitor, SupervisionEvent};
    /// use async_trait::async_trait;
    /// use std::time::Duration;
    ///
    /// # #[derive(Clone)]
    /// # struct MyWorker;
    /// # #[async_trait]
    /// # impl Child for MyWorker {
    /// #     type Error = std::io::Error;
    /// #     async fn start(&mut self) -> Result<(), Self::Error> { Ok(()) }
    /// #     async fn stop(&mut self, _: Duration) -> Result<(), Self::Error> { Ok(()) }
    /// # }
    /// # async fn example() -> Result<(), SupervisorError> {
    /// let mut supervisor = SupervisorNode::new(OneForOne, NoopMonitor::<SupervisionEvent>::new());
    ///
    /// let id = supervisor
    ///     .child("worker")
    ///     .factory(|| MyWorker)
    ///     .restart_policy(RestartPolicy::Transient)
    ///     .spawn()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn restart_policy(mut self, policy: RestartPolicy) -> Self {
        self.restart_policy = Some(policy);
        self
    }

    // -------------------------------------------------------------------------
    // Shutdown Policy Configuration
    // -------------------------------------------------------------------------

    /// Sets the shutdown policy to graceful with the specified timeout.
    ///
    /// Graceful shutdown calls [`Child::stop`] and waits up to the timeout
    /// for completion. This is the default behavior with a 5-second timeout.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use airssys_rt::supervisor::*;
    /// use airssys_rt::monitoring::{NoopMonitor, SupervisionEvent};
    /// use async_trait::async_trait;
    /// use std::time::Duration;
    ///
    /// # #[derive(Clone)]
    /// # struct MyDbConnection;
    /// # #[async_trait]
    /// # impl Child for MyDbConnection {
    /// #     type Error = std::io::Error;
    /// #     async fn start(&mut self) -> Result<(), Self::Error> { Ok(()) }
    /// #     async fn stop(&mut self, _: Duration) -> Result<(), Self::Error> { Ok(()) }
    /// # }
    /// # async fn example() -> Result<(), SupervisorError> {
    /// let mut supervisor = SupervisorNode::new(OneForOne, NoopMonitor::<SupervisionEvent>::new());
    ///
    /// // Database connection that needs 15s for cleanup
    /// let id = supervisor
    ///     .child("db")
    ///     .factory(|| MyDbConnection)
    ///     .shutdown_graceful(Duration::from_secs(15))
    ///     .spawn()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn shutdown_graceful(mut self, timeout: Duration) -> Self {
        self.shutdown_policy = Some(ShutdownPolicy::Graceful(timeout));
        self
    }

    /// Sets the shutdown policy to immediate termination.
    ///
    /// Immediate shutdown does not call [`Child::stop`], forcibly terminating
    /// the child. Use for children that cannot be gracefully stopped or where
    /// cleanup is not necessary.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use airssys_rt::supervisor::*;
    /// use airssys_rt::monitoring::{NoopMonitor, SupervisionEvent};
    /// use async_trait::async_trait;
    /// use std::time::Duration;
    ///
    /// # #[derive(Clone)]
    /// # struct MyStatelessWorker;
    /// # #[async_trait]
    /// # impl Child for MyStatelessWorker {
    /// #     type Error = std::io::Error;
    /// #     async fn start(&mut self) -> Result<(), Self::Error> { Ok(()) }
    /// #     async fn stop(&mut self, _: Duration) -> Result<(), Self::Error> { Ok(()) }
    /// # }
    /// # async fn example() -> Result<(), SupervisorError> {
    /// let mut supervisor = SupervisorNode::new(OneForOne, NoopMonitor::<SupervisionEvent>::new());
    ///
    /// // Stateless worker that doesn't need cleanup
    /// let id = supervisor
    ///     .child("worker")
    ///     .factory(|| MyStatelessWorker)
    ///     .shutdown_immediate()
    ///     .spawn()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn shutdown_immediate(mut self) -> Self {
        self.shutdown_policy = Some(ShutdownPolicy::Immediate);
        self
    }

    /// Sets the shutdown policy to wait indefinitely for graceful shutdown.
    ///
    /// Use when cleanup operations must complete no matter how long they take.
    /// Be careful with this setting as it can block supervisor shutdown
    /// indefinitely.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use airssys_rt::supervisor::*;
    /// use airssys_rt::monitoring::{NoopMonitor, SupervisionEvent};
    /// use async_trait::async_trait;
    /// use std::time::Duration;
    ///
    /// # #[derive(Clone)]
    /// # struct MyCriticalWriter;
    /// # #[async_trait]
    /// # impl Child for MyCriticalWriter {
    /// #     type Error = std::io::Error;
    /// #     async fn start(&mut self) -> Result<(), Self::Error> { Ok(()) }
    /// #     async fn stop(&mut self, _: Duration) -> Result<(), Self::Error> { Ok(()) }
    /// # }
    /// # async fn example() -> Result<(), SupervisorError> {
    /// let mut supervisor = SupervisorNode::new(OneForOne, NoopMonitor::<SupervisionEvent>::new());
    ///
    /// // Critical data writer that must flush completely
    /// let id = supervisor
    ///     .child("writer")
    ///     .factory(|| MyCriticalWriter)
    ///     .shutdown_infinity()
    ///     .spawn()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn shutdown_infinity(mut self) -> Self {
        self.shutdown_policy = Some(ShutdownPolicy::Infinity);
        self
    }

    /// Sets a custom shutdown policy.
    ///
    /// For most cases, use the shortcut methods ([`shutdown_graceful`],
    /// [`shutdown_immediate`], [`shutdown_infinity`]) instead.
    ///
    /// [`shutdown_graceful`]: Self::shutdown_graceful
    /// [`shutdown_immediate`]: Self::shutdown_immediate
    /// [`shutdown_infinity`]: Self::shutdown_infinity
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use airssys_rt::supervisor::*;
    /// use airssys_rt::monitoring::{NoopMonitor, SupervisionEvent};
    /// use async_trait::async_trait;
    /// use std::time::Duration;
    ///
    /// # #[derive(Clone)]
    /// # struct MyWorker;
    /// # #[async_trait]
    /// # impl Child for MyWorker {
    /// #     type Error = std::io::Error;
    /// #     async fn start(&mut self) -> Result<(), Self::Error> { Ok(()) }
    /// #     async fn stop(&mut self, _: Duration) -> Result<(), Self::Error> { Ok(()) }
    /// # }
    /// # async fn example() -> Result<(), SupervisorError> {
    /// let mut supervisor = SupervisorNode::new(OneForOne, NoopMonitor::<SupervisionEvent>::new());
    ///
    /// let id = supervisor
    ///     .child("worker")
    ///     .factory(|| MyWorker)
    ///     .shutdown_policy(ShutdownPolicy::Graceful(Duration::from_secs(10)))
    ///     .spawn()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn shutdown_policy(mut self, policy: ShutdownPolicy) -> Self {
        self.shutdown_policy = Some(policy);
        self
    }

    // -------------------------------------------------------------------------
    // Timeout Configuration
    // -------------------------------------------------------------------------

    /// Sets the timeout for child startup.
    ///
    /// If [`Child::start`] does not complete within this duration, the child
    /// is considered failed to start. Default is 30 seconds.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use airssys_rt::supervisor::*;
    /// use airssys_rt::monitoring::{NoopMonitor, SupervisionEvent};
    /// use async_trait::async_trait;
    /// use std::time::Duration;
    ///
    /// # #[derive(Clone)]
    /// # struct MyFastWorker;
    /// # #[async_trait]
    /// # impl Child for MyFastWorker {
    /// #     type Error = std::io::Error;
    /// #     async fn start(&mut self) -> Result<(), Self::Error> { Ok(()) }
    /// #     async fn stop(&mut self, _: Duration) -> Result<(), Self::Error> { Ok(()) }
    /// # }
    /// # async fn example() -> Result<(), SupervisorError> {
    /// let mut supervisor = SupervisorNode::new(OneForOne, NoopMonitor::<SupervisionEvent>::new());
    ///
    /// // Fast-starting worker
    /// let id = supervisor
    ///     .child("worker")
    ///     .factory(|| MyFastWorker)
    ///     .start_timeout(Duration::from_secs(5))
    ///     .spawn()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn start_timeout(mut self, timeout: Duration) -> Self {
        self.start_timeout = Some(timeout);
        self
    }

    /// Sets the maximum time to wait for child shutdown.
    ///
    /// This is the overall timeout for the shutdown process, which may include
    /// graceful shutdown and cleanup. Default is 10 seconds.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use airssys_rt::supervisor::*;
    /// use airssys_rt::monitoring::{NoopMonitor, SupervisionEvent};
    /// use async_trait::async_trait;
    /// use std::time::Duration;
    ///
    /// # #[derive(Clone)]
    /// # struct MyWorker;
    /// # #[async_trait]
    /// # impl Child for MyWorker {
    /// #     type Error = std::io::Error;
    /// #     async fn start(&mut self) -> Result<(), Self::Error> { Ok(()) }
    /// #     async fn stop(&mut self, _: Duration) -> Result<(), Self::Error> { Ok(()) }
    /// # }
    /// # async fn example() -> Result<(), SupervisorError> {
    /// let mut supervisor = SupervisorNode::new(OneForOne, NoopMonitor::<SupervisionEvent>::new());
    ///
    /// let id = supervisor
    ///     .child("worker")
    ///     .factory(|| MyWorker)
    ///     .shutdown_graceful(Duration::from_secs(15))
    ///     .shutdown_timeout(Duration::from_secs(20))  // Overall limit
    ///     .spawn()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn shutdown_timeout(mut self, timeout: Duration) -> Self {
        self.shutdown_timeout = Some(timeout);
        self
    }

    // -------------------------------------------------------------------------
    // Execution Methods
    // -------------------------------------------------------------------------

    /// Builds the child spec and spawns the child.
    ///
    /// This method validates the configuration (factory must be set), applies
    /// defaults for all optional settings, and immediately starts the child
    /// in the supervisor. Returns the child ID on success.
    ///
    /// # Errors
    ///
    /// Returns errors from both spec building (missing factory) and child
    /// startup (start timeout, child start failure).
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use airssys_rt::supervisor::*;
    /// use airssys_rt::monitoring::{NoopMonitor, SupervisionEvent};
    /// use async_trait::async_trait;
    /// use std::time::Duration;
    ///
    /// # #[derive(Clone)]
    /// # struct MyWorker;
    /// # #[async_trait]
    /// # impl Child for MyWorker {
    /// #     type Error = std::io::Error;
    /// #     async fn start(&mut self) -> Result<(), Self::Error> { Ok(()) }
    /// #     async fn stop(&mut self, _: Duration) -> Result<(), Self::Error> { Ok(()) }
    /// # }
    /// # async fn example() -> Result<(), SupervisorError> {
    /// let mut supervisor = SupervisorNode::new(OneForOne, NoopMonitor::<SupervisionEvent>::new());
    ///
    /// // Build and spawn in one call
    /// let child_id = supervisor
    ///     .child("worker")
    ///     .factory(|| MyWorker)
    ///     .spawn()
    ///     .await?;
    ///
    /// println!("Spawned child: {}", child_id);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn spawn(self) -> Result<ChildId, SupervisorError> {
        let factory = self
            .factory
            .ok_or_else(|| SupervisorError::InvalidConfiguration {
                reason: "Factory function is required. Call .factory() before .spawn()".into(),
            })?;

        let spec = ChildSpec {
            id: self.id,
            factory,
            restart_policy: self.restart_policy.unwrap_or(DEFAULT_RESTART_POLICY),
            shutdown_policy: self.shutdown_policy.unwrap_or(DEFAULT_SHUTDOWN_POLICY),
            start_timeout: self.start_timeout.unwrap_or(DEFAULT_START_TIMEOUT),
            shutdown_timeout: self.shutdown_timeout.unwrap_or(DEFAULT_SHUTDOWN_TIMEOUT),
        };

        self.supervisor.start_child(spec).await
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::panic, clippy::redundant_closure)]
mod tests {
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;
    use std::time::Duration;

    use super::*;
    use crate::monitoring::{NoopMonitor, SupervisionEvent};
    use crate::supervisor::{OneForOne, SupervisorNode};

    // Test child implementation
    #[derive(Clone)]
    struct TestChild {
        start_count: Arc<AtomicU32>,
        stop_count: Arc<AtomicU32>,
    }

    impl TestChild {
        fn new() -> Self {
            Self {
                start_count: Arc::new(AtomicU32::new(0)),
                stop_count: Arc::new(AtomicU32::new(0)),
            }
        }
    }

    #[async_trait::async_trait]
    impl Child for TestChild {
        type Error = std::io::Error;

        async fn start(&mut self) -> Result<(), Self::Error> {
            self.start_count.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }

        async fn stop(&mut self, _timeout: Duration) -> Result<(), Self::Error> {
            self.stop_count.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
    }

    #[derive(Clone, Default)]
    struct DefaultTestChild;

    #[async_trait::async_trait]
    impl Child for DefaultTestChild {
        type Error = std::io::Error;

        async fn start(&mut self) -> Result<(), Self::Error> {
            Ok(())
        }

        async fn stop(&mut self, _timeout: Duration) -> Result<(), Self::Error> {
            Ok(())
        }
    }

    // Helper to create supervisor
    fn create_supervisor() -> SupervisorNode<OneForOne, TestChild, NoopMonitor<SupervisionEvent>> {
        SupervisorNode::new(OneForOne, NoopMonitor::new())
    }

    // -------------------------------------------------------------------------
    // Builder Construction Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_builder_creation_via_supervisor() {
        let mut supervisor = create_supervisor();
        let _builder = supervisor.child("test");
        // Builder creation succeeds
    }

    // -------------------------------------------------------------------------
    // Factory Methods Tests
    // -------------------------------------------------------------------------

    #[tokio::test]
    async fn test_factory_method() {
        let mut supervisor = create_supervisor();

        let child_id = supervisor
            .child("test")
            .factory(|| TestChild::new())
            .spawn()
            .await
            .unwrap();

        // ChildId is UUID-based, so we compare the string representation
        // Note: The actual ID will be different due to UUID generation,
        // so we just verify it was returned successfully
        assert!(!child_id.to_string().is_empty());
    }

    #[tokio::test]
    async fn test_factory_default() {
        let mut supervisor: SupervisorNode<
            OneForOne,
            DefaultTestChild,
            NoopMonitor<SupervisionEvent>,
        > = SupervisorNode::new(OneForOne, NoopMonitor::new());

        let child_id = supervisor
            .child("test")
            .factory_default::<DefaultTestChild>()
            .spawn()
            .await
            .unwrap();

        // ChildId is UUID-based, verify it was returned
        assert!(!child_id.to_string().is_empty());
    }

    // -------------------------------------------------------------------------
    // Restart Policy Tests
    // -------------------------------------------------------------------------

    #[tokio::test]
    async fn test_restart_permanent() {
        let mut supervisor = create_supervisor();

        // Verify the builder API compiles and spawns successfully
        let child_id = supervisor
            .child("test")
            .factory(|| TestChild::new())
            .restart_permanent()
            .spawn()
            .await
            .unwrap();

        // Verify child was spawned
        assert!(supervisor.get_child(&child_id).is_some());
    }

    #[tokio::test]
    async fn test_restart_transient() {
        let mut supervisor = create_supervisor();

        let child_id = supervisor
            .child("test")
            .factory(|| TestChild::new())
            .restart_transient()
            .spawn()
            .await
            .unwrap();

        assert!(supervisor.get_child(&child_id).is_some());
    }

    #[tokio::test]
    async fn test_restart_temporary() {
        let mut supervisor = create_supervisor();

        let child_id = supervisor
            .child("test")
            .factory(|| TestChild::new())
            .restart_temporary()
            .spawn()
            .await
            .unwrap();

        assert!(supervisor.get_child(&child_id).is_some());
    }

    #[tokio::test]
    async fn test_restart_policy_custom() {
        let mut supervisor = create_supervisor();

        let child_id = supervisor
            .child("test")
            .factory(|| TestChild::new())
            .restart_policy(RestartPolicy::Transient)
            .spawn()
            .await
            .unwrap();

        assert!(supervisor.get_child(&child_id).is_some());
    }

    // -------------------------------------------------------------------------
    // Shutdown Policy Tests
    // -------------------------------------------------------------------------

    #[tokio::test]
    async fn test_shutdown_graceful() {
        let mut supervisor = create_supervisor();

        let child_id = supervisor
            .child("test")
            .factory(|| TestChild::new())
            .shutdown_graceful(Duration::from_secs(15))
            .spawn()
            .await
            .unwrap();

        assert!(supervisor.get_child(&child_id).is_some());
    }

    #[tokio::test]
    async fn test_shutdown_immediate() {
        let mut supervisor = create_supervisor();

        let child_id = supervisor
            .child("test")
            .factory(|| TestChild::new())
            .shutdown_immediate()
            .spawn()
            .await
            .unwrap();

        assert!(supervisor.get_child(&child_id).is_some());
    }

    #[tokio::test]
    async fn test_shutdown_infinity() {
        let mut supervisor = create_supervisor();

        let child_id = supervisor
            .child("test")
            .factory(|| TestChild::new())
            .shutdown_infinity()
            .spawn()
            .await
            .unwrap();

        assert!(supervisor.get_child(&child_id).is_some());
    }

    #[tokio::test]
    async fn test_shutdown_policy_custom() {
        let mut supervisor = create_supervisor();

        let child_id = supervisor
            .child("test")
            .factory(|| TestChild::new())
            .shutdown_policy(ShutdownPolicy::Immediate)
            .spawn()
            .await
            .unwrap();

        assert!(supervisor.get_child(&child_id).is_some());
    }

    // -------------------------------------------------------------------------
    // Timeout Configuration Tests
    // -------------------------------------------------------------------------

    #[tokio::test]
    async fn test_start_timeout() {
        let mut supervisor = create_supervisor();

        let child_id = supervisor
            .child("test")
            .factory(|| TestChild::new())
            .start_timeout(Duration::from_secs(60))
            .spawn()
            .await
            .unwrap();

        assert!(supervisor.get_child(&child_id).is_some());
    }

    #[tokio::test]
    async fn test_shutdown_timeout() {
        let mut supervisor = create_supervisor();

        let child_id = supervisor
            .child("test")
            .factory(|| TestChild::new())
            .shutdown_timeout(Duration::from_secs(20))
            .spawn()
            .await
            .unwrap();

        assert!(supervisor.get_child(&child_id).is_some());
    }

    // -------------------------------------------------------------------------
    // Defaults Application Tests
    // -------------------------------------------------------------------------

    #[tokio::test]
    async fn test_default_restart_policy_applied() {
        let mut supervisor = create_supervisor();

        // Defaults are applied internally - verify child spawns successfully
        let child_id = supervisor
            .child("test")
            .factory(|| TestChild::new())
            .spawn()
            .await
            .unwrap();

        assert!(supervisor.get_child(&child_id).is_some());
    }

    #[tokio::test]
    async fn test_default_shutdown_policy_applied() {
        let mut supervisor = create_supervisor();

        let child_id = supervisor
            .child("test")
            .factory(|| TestChild::new())
            .spawn()
            .await
            .unwrap();

        assert!(supervisor.get_child(&child_id).is_some());
    }

    #[tokio::test]
    async fn test_default_start_timeout_applied() {
        let mut supervisor = create_supervisor();

        let child_id = supervisor
            .child("test")
            .factory(|| TestChild::new())
            .spawn()
            .await
            .unwrap();

        assert!(supervisor.get_child(&child_id).is_some());
    }

    #[tokio::test]
    async fn test_default_shutdown_timeout_applied() {
        let mut supervisor = create_supervisor();

        let child_id = supervisor
            .child("test")
            .factory(|| TestChild::new())
            .spawn()
            .await
            .unwrap();

        assert!(supervisor.get_child(&child_id).is_some());
    }

    // -------------------------------------------------------------------------
    // Method Chaining Tests
    // -------------------------------------------------------------------------

    #[tokio::test]
    async fn test_full_configuration_chain() {
        let mut supervisor = create_supervisor();

        let child_id = supervisor
            .child("test")
            .factory(|| TestChild::new())
            .restart_transient()
            .shutdown_graceful(Duration::from_secs(15))
            .start_timeout(Duration::from_secs(60))
            .shutdown_timeout(Duration::from_secs(20))
            .spawn()
            .await
            .unwrap();

        // Verify all methods chained successfully and child was spawned
        assert!(supervisor.get_child(&child_id).is_some());
    }

    #[tokio::test]
    async fn test_minimal_configuration() {
        let mut supervisor = create_supervisor();

        // Should use all defaults and spawn successfully
        let child_id = supervisor
            .child("test")
            .factory(|| TestChild::new())
            .spawn()
            .await
            .unwrap();

        assert!(supervisor.get_child(&child_id).is_some());
    }

    // -------------------------------------------------------------------------
    // Build vs Spawn Tests
    // -------------------------------------------------------------------------

    #[tokio::test]
    async fn test_spawn_requires_factory() {
        let mut supervisor = create_supervisor();

        // spawn() without factory should fail
        let result = supervisor.child("test").spawn().await;

        assert!(result.is_err());
        if let Err(SupervisorError::InvalidConfiguration { reason }) = result {
            assert!(reason.contains("Factory function is required"));
        } else {
            panic!("Expected InvalidConfiguration error");
        }
    }

    #[tokio::test]
    async fn test_spawn_starts_child() {
        let mut supervisor = create_supervisor();

        let child_id = supervisor
            .child("test")
            .factory(|| TestChild::new())
            .spawn()
            .await
            .unwrap();

        // Verify child was actually started
        let child_ids = supervisor.child_ids();
        assert!(child_ids.contains(&child_id));
    }
}
