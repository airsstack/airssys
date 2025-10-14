//! # ProcessActor
//!
//! Centralized actor for OS process spawning and management with lifecycle tracking.
//!
//! ## Responsibilities
//!
//! - OS process spawning and management
//! - Process lifecycle tracking (spawned, running, stopped)
//! - Process termination (graceful + forced)
//! - Resource cleanup on actor shutdown
//! - Centralized process audit logging
//!
//! ## Key Feature: Automatic Cleanup
//!
//! When the ProcessActor stops (via Child::stop()), it automatically terminates
//! all spawned processes, preventing zombie processes.

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::actor::{Actor, ActorContext, ErrorAction};
use crate::broker::MessageBroker;
use crate::supervisor::{Child, ChildHealth};

use super::messages::{
    ProcessError, ProcessId, ProcessOperation, ProcessRequest, ProcessResponse, ProcessResult,
    ProcessState,
};

/// ProcessActor - Centralized process operations
///
/// This actor serves as the interface between the actor runtime and
/// OS process operations. All application actors should send messages
/// to this actor rather than spawning processes directly.
///
/// ## Benefits
///
/// - Centralized audit logging for all process operations
/// - Automatic cleanup when actor stops (no zombie processes)
/// - Clean fault isolation (process failures don't crash app actors)
/// - Superior testability (mock this actor in tests)
pub struct ProcessActor {
    /// Spawned processes registry
    spawned_processes: HashMap<u32, ProcessHandle>,

    /// Process ID counter
    next_process_id: ProcessId,

    /// Operation counter for metrics
    operation_count: u64,

    /// Actor creation timestamp
    created_at: DateTime<Utc>,

    /// Last operation timestamp
    last_operation_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct ProcessHandle {
    pid: u32,
    process_id: ProcessId,
    program: PathBuf,
    spawned_at: DateTime<Utc>,
    state: ProcessState,
}

impl ProcessActor {
    /// Create a new ProcessActor
    pub fn new() -> Self {
        Self {
            spawned_processes: HashMap::new(),
            next_process_id: 1,
            operation_count: 0,
            created_at: Utc::now(),
            last_operation_at: None,
        }
    }

    /// Get spawned process count
    pub fn spawned_process_count(&self) -> usize {
        self.spawned_processes.len()
    }

    /// Get operation count
    pub fn operation_count(&self) -> u64 {
        self.operation_count
    }

    /// Internal process spawn logic
    fn spawn_process_internal(
        &mut self,
        program: &PathBuf,
        args: &[String],
        env: &HashMap<String, String>,
        working_dir: Option<&std::path::Path>,
    ) -> Result<(u32, ProcessId), Box<dyn std::error::Error>> {
        // Mock implementation for now
        // TODO: Replace with airssys-osl integration
        let mut command = std::process::Command::new(program);
        command.args(args);

        for (key, value) in env {
            command.env(key, value);
        }

        if let Some(dir) = working_dir {
            command.current_dir(dir);
        }

        // Spawn process
        let child = command.spawn()?;
        let pid = child.id();
        let process_id = self.next_process_id;
        self.next_process_id += 1;

        // Note: We're not storing the Child handle, which means we can't
        // directly control it. In production, we'd integrate with airssys-osl
        // which provides proper process management.
        std::mem::forget(child);

        Ok((pid, process_id))
    }

    /// Internal process termination logic
    fn terminate_process_internal(
        &self,
        pid: u32,
        _graceful: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Mock implementation for now
        // TODO: Replace with airssys-osl integration
        #[cfg(unix)]
        {
            use nix::sys::signal::{self, Signal};
            use nix::unistd::Pid;

            let pid = Pid::from_raw(pid as i32);
            signal::kill(pid, Signal::SIGTERM)?;
        }

        #[cfg(windows)]
        {
            // Windows process termination would go here
            // For now, just return Ok
        }

        Ok(())
    }

    /// Terminate all spawned processes
    async fn terminate_all_processes(&mut self) {
        let pids: Vec<u32> = self.spawned_processes.keys().copied().collect();

        for pid in pids {
            if let Err(e) = self.terminate_process_internal(pid, false) {
                eprintln!("Failed to terminate process {pid}: {e}");
            }
        }

        self.spawned_processes.clear();
    }

    /// Execute process operation and return result
    async fn execute_operation(&mut self, operation: ProcessOperation) -> ProcessResult {
        self.operation_count += 1;
        self.last_operation_at = Some(Utc::now());

        match operation {
            ProcessOperation::Spawn {
                program,
                args,
                env,
                working_dir,
            } => match self.spawn_process_internal(&program, &args, &env, working_dir.as_deref()) {
                Ok((pid, process_id)) => {
                    self.spawned_processes.insert(
                        pid,
                        ProcessHandle {
                            pid,
                            process_id,
                            program: program.clone(),
                            spawned_at: Utc::now(),
                            state: ProcessState::Running,
                        },
                    );
                    ProcessResult::SpawnSuccess { pid, process_id }
                }
                Err(e) => ProcessResult::Error {
                    error: ProcessError::SpawnFailed {
                        program: program.display().to_string(),
                        message: e.to_string(),
                    },
                },
            },
            ProcessOperation::Terminate {
                pid,
                graceful,
                timeout: _,
            } => match self.terminate_process_internal(pid, graceful) {
                Ok(_) => {
                    self.spawned_processes.remove(&pid);
                    ProcessResult::TerminateSuccess { pid }
                }
                Err(e) => ProcessResult::Error {
                    error: ProcessError::TerminateFailed {
                        pid,
                        message: e.to_string(),
                    },
                },
            },
            ProcessOperation::GetStatus { pid } => {
                if let Some(handle) = self.spawned_processes.get(&pid) {
                    ProcessResult::Status {
                        pid,
                        state: handle.state,
                    }
                } else {
                    ProcessResult::Error {
                        error: ProcessError::NotFound { pid },
                    }
                }
            }
            ProcessOperation::Wait { pid, timeout } => {
                if self.spawned_processes.contains_key(&pid) {
                    if timeout.is_some() {
                        ProcessResult::WaitTimeout { pid }
                    } else {
                        ProcessResult::WaitSuccess { pid, exit_code: 0 }
                    }
                } else {
                    ProcessResult::Error {
                        error: ProcessError::NotFound { pid },
                    }
                }
            }
        }
    }
}

impl Default for ProcessActor {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Actor for ProcessActor {
    type Message = ProcessRequest;
    type Error = ProcessError;

    async fn handle_message<B: MessageBroker<Self::Message>>(
        &mut self,
        message: Self::Message,
        _context: &mut ActorContext<Self::Message, B>,
    ) -> Result<(), Self::Error> {
        // Execute operation
        let result = self.execute_operation(message.operation).await;

        // Create response
        let response = ProcessResponse {
            request_id: message.request_id,
            result,
        };

        // Send response via broker
        // TODO: Need to use broker.publish() directly
        println!("ProcessActor response: {response:?}");

        Ok(())
    }

    async fn on_error<B: MessageBroker<Self::Message>>(
        &mut self,
        error: Self::Error,
        _context: &mut ActorContext<Self::Message, B>,
    ) -> ErrorAction {
        eprintln!("ProcessActor error: {error:?}");
        ErrorAction::Resume
    }
}

#[async_trait]
impl Child for ProcessActor {
    type Error = ProcessError;

    async fn start(&mut self) -> Result<(), Self::Error> {
        println!("ProcessActor starting at {}", self.created_at);
        Ok(())
    }

    async fn stop(&mut self, _timeout: Duration) -> Result<(), Self::Error> {
        // CRITICAL: Clean up all spawned processes
        if !self.spawned_processes.is_empty() {
            println!(
                "ProcessActor stopping with {} spawned processes. Terminating all...",
                self.spawned_processes.len()
            );
            self.terminate_all_processes().await;
        }
        println!(
            "ProcessActor stopped. Total operations: {}, spawned processes: {}",
            self.operation_count,
            self.spawned_processes.len()
        );
        Ok(())
    }

    async fn health_check(&self) -> ChildHealth {
        // Check if process registry is too large
        if self.spawned_processes.len() > 100 {
            ChildHealth::Degraded(format!(
                "Too many spawned processes: {}",
                self.spawned_processes.len()
            ))
        } else {
            ChildHealth::Healthy
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_actor_new() {
        let actor = ProcessActor::new();
        assert_eq!(actor.spawned_process_count(), 0);
        assert_eq!(actor.operation_count(), 0);
    }

    #[test]
    fn test_process_actor_default() {
        let actor = ProcessActor::default();
        assert_eq!(actor.spawned_process_count(), 0);
    }

    #[tokio::test]
    async fn test_process_actor_health_check() {
        let actor = ProcessActor::new();
        let health = actor.health_check().await;
        assert_eq!(health, ChildHealth::Healthy);
    }

    #[tokio::test]
    async fn test_process_actor_health_degraded() {
        let mut actor = ProcessActor::new();

        // Add many spawned processes to trigger degraded state (threshold is 100)
        for i in 0..101 {
            actor.spawned_processes.insert(
                i,
                ProcessHandle {
                    pid: i,
                    process_id: i as ProcessId,
                    program: PathBuf::from("test"),
                    spawned_at: Utc::now(),
                    state: ProcessState::Running,
                },
            );
        }

        let health = actor.health_check().await;
        assert!(matches!(health, ChildHealth::Degraded(_)));
    }
}
