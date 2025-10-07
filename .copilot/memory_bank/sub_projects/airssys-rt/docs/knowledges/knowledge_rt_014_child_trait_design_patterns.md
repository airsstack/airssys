# KNOWLEDGE-RT-014: Child Trait Design Patterns and Integration Strategies

**Sub-Project:** airssys-rt  
**Category:** Patterns  
**Created:** 2025-10-07  
**Last Updated:** 2025-10-07  
**Status:** active  

## Context and Problem

RT-TASK-007 (Supervisor Framework) requires a lifecycle management interface for supervised entities. The design decision (ADR-RT-004) establishes a separate `Child` trait independent from the `Actor` trait, with a blanket implementation bridge. This knowledge document provides comprehensive implementation patterns, integration strategies, and best practices for the Child trait architecture.

## Knowledge Details

### Core Architecture: Child Trait Separation

The Child trait provides a universal lifecycle interface for ANY supervisable entity, not just actors. This follows BEAM/Erlang OTP philosophy where supervisors manage processes, gen_servers, gen_statem, tasks, and custom behaviors.

```rust
//! Child trait - Universal lifecycle interface for supervised entities

// Layer 1: Standard library imports
use std::error::Error;
use std::time::Duration;

// Layer 2: Third-party crate imports
use async_trait::async_trait;

// Layer 3: Internal module imports
use super::types::ChildHealth;

/// Child trait for entities that can be supervised.
///
/// Any entity implementing this trait can be placed under supervisor management,
/// enabling fault-tolerant hierarchical supervision trees. This trait is
/// intentionally separate from `Actor` to allow supervision of diverse entity
/// types including actors, background tasks, I/O handlers, and system services.
///
/// # Design Philosophy
///
/// - **Universal Interface**: Any process-like entity can be supervised
/// - **BEAM Alignment**: Matches Erlang/OTP supervisor behavior model
/// - **Zero-Cost Bridge**: Blanket impl for Actor trait (zero overhead)
/// - **Composability**: Mix actors and non-actors in supervision trees
///
/// # Lifecycle Methods
///
/// - `start()`: Initialize and start the child process (REQUIRED)
/// - `stop()`: Gracefully shutdown with timeout (REQUIRED)
/// - `health_check()`: Report health status (OPTIONAL, default: Healthy)
///
/// # Blanket Implementation for Actors
///
/// All types implementing `Actor` automatically implement `Child` via blanket impl:
/// ```rust
/// #[async_trait]
/// impl<A> Child for A
/// where
///     A: Actor + Send + Sync + 'static,
///     A::Error: Error + Send + Sync + 'static,
/// {
///     type Error = A::Error;
///     
///     async fn start(&mut self) -> Result<(), Self::Error> {
///         self.pre_start().await  // Delegates to Actor lifecycle
///     }
///     
///     async fn stop(&mut self, _timeout: Duration) -> Result<(), Self::Error> {
///         self.post_stop().await  // Delegates to Actor lifecycle
///     }
/// }
/// ```
///
/// # Examples
///
/// ## Example 1: Actor Supervision (Automatic)
/// ```rust
/// use airssys_rt::{Actor, Child, ActorContext};
/// use async_trait::async_trait;
///
/// struct CounterActor { count: u32 }
///
/// #[async_trait]
/// impl Actor for CounterActor {
///     type Message = CounterMsg;
///     type Error = CounterError;
///     
///     async fn handle_message(&mut self, msg: Self::Message, ctx: &mut ActorContext<Self::Message>) 
///         -> Result<(), Self::Error> 
///     {
///         self.count += msg.delta;
///         Ok(())
///     }
/// }
///
/// // ✅ CounterActor automatically implements Child!
/// // No additional code needed - works with supervisors immediately
/// ```
///
/// ## Example 2: Non-Actor Background Task
/// ```rust
/// use airssys_rt::{Child, ChildHealth};
/// use async_trait::async_trait;
/// use std::time::Duration;
/// use tokio::task::JoinHandle;
///
/// struct BackgroundWorker {
///     task_queue: Vec<Task>,
///     handle: Option<JoinHandle<()>>,
/// }
///
/// #[async_trait]
/// impl Child for BackgroundWorker {
///     type Error = WorkerError;
///     
///     async fn start(&mut self) -> Result<(), Self::Error> {
///         let queue = self.task_queue.clone();
///         let handle = tokio::spawn(async move {
///             for task in queue {
///                 task.execute().await;
///             }
///         });
///         self.handle = Some(handle);
///         Ok(())
///     }
///     
///     async fn stop(&mut self, timeout: Duration) -> Result<(), Self::Error> {
///         if let Some(handle) = self.handle.take() {
///             tokio::select! {
///                 _ = handle => Ok(()),
///                 _ = tokio::time::sleep(timeout) => {
///                     Err(WorkerError::ShutdownTimeout)
///                 }
///             }
///         } else {
///             Ok(())
///         }
///     }
///     
///     async fn health_check(&self) -> ChildHealth {
///         if self.task_queue.is_empty() {
///             ChildHealth::Healthy
///         } else if self.task_queue.len() > 1000 {
///             ChildHealth::Degraded(format!("Queue backlog: {}", self.task_queue.len()))
///         } else {
///             ChildHealth::Healthy
///         }
///     }
/// }
/// ```
///
/// ## Example 3: I/O Handler Supervision
/// ```rust
/// struct FileWatcher {
///     path: PathBuf,
///     watcher: Option<notify::RecommendedWatcher>,
/// }
///
/// #[async_trait]
/// impl Child for FileWatcher {
///     type Error = FileWatcherError;
///     
///     async fn start(&mut self) -> Result<(), Self::Error> {
///         let watcher = notify::recommended_watcher(|event| {
///             println!("File event: {:?}", event);
///         })?;
///         self.watcher = Some(watcher);
///         Ok(())
///     }
///     
///     async fn stop(&mut self, _timeout: Duration) -> Result<(), Self::Error> {
///         self.watcher.take();  // Drop watcher to stop
///         Ok(())
///     }
///     
///     async fn health_check(&self) -> ChildHealth {
///         if self.watcher.is_some() {
///             ChildHealth::Healthy
///         } else {
///             ChildHealth::Failed("Watcher is None".into())
///         }
///     }
/// }
/// ```
#[async_trait]
pub trait Child: Send + Sync + 'static {
    /// Error type for child lifecycle operations
    type Error: Error + Send + Sync + 'static;

    /// Start the child process.
    ///
    /// This method should initialize all resources and begin operation.
    /// Implementations should be idempotent where possible - calling
    /// start() on an already-started child should either succeed or
    /// return an appropriate error.
    ///
    /// # Errors
    ///
    /// Returns error if initialization fails. The supervisor will handle
    /// the error according to the configured `RestartPolicy`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// async fn start(&mut self) -> Result<(), Self::Error> {
    ///     self.initialize_resources()?;
    ///     self.spawn_background_task()?;
    ///     Ok(())
    /// }
    /// ```
    async fn start(&mut self) -> Result<(), Self::Error>;

    /// Stop the child process gracefully.
    ///
    /// This method should perform graceful shutdown within the given timeout.
    /// After timeout expires, the supervisor may forcefully terminate the
    /// child depending on the `ShutdownPolicy` configuration.
    ///
    /// # Parameters
    ///
    /// - `timeout`: Maximum time to wait for graceful shutdown
    ///
    /// # Errors
    ///
    /// Returns error if shutdown fails or times out. Errors are logged
    /// but typically don't affect supervision decisions since the child
    /// is being stopped anyway.
    ///
    /// # Examples
    ///
    /// ```rust
    /// async fn stop(&mut self, timeout: Duration) -> Result<(), Self::Error> {
    ///     tokio::select! {
    ///         _ = self.graceful_shutdown() => Ok(()),
    ///         _ = tokio::time::sleep(timeout) => {
    ///             Err(Self::Error::ShutdownTimeout)
    ///         }
    ///     }
    /// }
    /// ```
    async fn stop(&mut self, timeout: Duration) -> Result<(), Self::Error>;

    /// Check the health status of the child.
    ///
    /// Used by supervisors to detect degraded or failing children before
    /// they completely fail. This enables proactive restart or recovery
    /// strategies.
    ///
    /// # Default Implementation
    ///
    /// The default implementation returns `ChildHealth::Healthy`. Override
    /// this method to provide custom health checking logic.
    ///
    /// # Returns
    ///
    /// - `ChildHealth::Healthy`: Child is operating normally
    /// - `ChildHealth::Degraded(reason)`: Child is operational but degraded
    /// - `ChildHealth::Failed(reason)`: Child has failed and needs restart
    ///
    /// # Examples
    ///
    /// ```rust
    /// async fn health_check(&self) -> ChildHealth {
    ///     if self.error_count > self.error_threshold {
    ///         ChildHealth::Failed(format!("Error count: {}", self.error_count))
    ///     } else if self.latency_ms > 1000 {
    ///         ChildHealth::Degraded(format!("High latency: {}ms", self.latency_ms))
    ///     } else {
    ///         ChildHealth::Healthy
    ///     }
    /// }
    /// ```
    async fn health_check(&self) -> ChildHealth {
        ChildHealth::Healthy
    }
}
```

### ChildHealth Status Type

```rust
/// Health status of a supervised child.
///
/// Used by `Child::health_check()` to report the operational status of a
/// supervised entity. Supervisors can use this information for proactive
/// failure detection and recovery strategies.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChildHealth {
    /// Child is operating normally with no issues detected.
    Healthy,
    
    /// Child is operational but showing signs of degradation.
    ///
    /// Examples: high latency, memory pressure, error rate increase,
    /// resource exhaustion warnings. The child can still process work
    /// but may need attention or proactive restart.
    Degraded(String),
    
    /// Child has failed and requires restart.
    ///
    /// The supervisor should initiate restart according to the configured
    /// `RestartPolicy` and `RestartStrategy`.
    Failed(String),
}

impl ChildHealth {
    /// Returns `true` if the child is healthy.
    pub fn is_healthy(&self) -> bool {
        matches!(self, ChildHealth::Healthy)
    }
    
    /// Returns `true` if the child is degraded.
    pub fn is_degraded(&self) -> bool {
        matches!(self, ChildHealth::Degraded(_))
    }
    
    /// Returns `true` if the child has failed.
    pub fn is_failed(&self) -> bool {
        matches!(self, ChildHealth::Failed(_))
    }
    
    /// Returns the reason string if degraded or failed.
    pub fn reason(&self) -> Option<&str> {
        match self {
            ChildHealth::Healthy => None,
            ChildHealth::Degraded(reason) | ChildHealth::Failed(reason) => Some(reason),
        }
    }
}
```

### Blanket Implementation: Actor → Child Bridge

```rust
//! Blanket implementation providing automatic Child trait for all Actors

// Layer 1: Standard library imports
use std::time::Duration;

// Layer 2: Third-party crate imports
use async_trait::async_trait;

// Layer 3: Internal module imports
use crate::actor::{Actor, ActorContext};
use super::traits::{Child, ChildHealth};

/// Blanket implementation: All Actors automatically implement Child.
///
/// This bridges the Actor trait with the Child trait, allowing existing
/// actors to be supervised without any code changes. The implementation
/// delegates to Actor lifecycle hooks:
///
/// - `Child::start()` → `Actor::pre_start()`
/// - `Child::stop()` → `Actor::post_stop()`
/// - `Child::health_check()` → Default (Healthy)
///
/// # Zero-Cost Abstraction
///
/// This blanket impl compiles to direct Actor method calls through
/// monomorphization, with zero runtime overhead.
///
/// # Examples
///
/// ```rust
/// // Define an actor
/// struct MyActor { state: u32 }
///
/// #[async_trait]
/// impl Actor for MyActor {
///     type Message = MyMessage;
///     type Error = MyError;
///     
///     async fn handle_message(&mut self, msg: Self::Message, ctx: &mut ActorContext<Self::Message>) 
///         -> Result<(), Self::Error> 
///     {
///         self.state += msg.value;
///         Ok(())
///     }
/// }
///
/// // ✅ MyActor now automatically implements Child!
/// // Can be used with supervisors immediately:
/// let supervisor = SupervisorNode::new(OneForOne, monitor);
/// supervisor.add_child(ChildSpec {
///     id: "my_actor".into(),
///     factory: || MyActor { state: 0 },
///     // ...
/// }).await?;
/// ```
#[async_trait]
impl<A> Child for A
where
    A: Actor + Send + Sync + 'static,
    A::Error: std::error::Error + Send + Sync + 'static,
{
    type Error = A::Error;

    async fn start(&mut self) -> Result<(), Self::Error> {
        // Delegate to Actor's pre_start lifecycle hook
        self.pre_start().await
    }

    async fn stop(&mut self, _timeout: Duration) -> Result<(), Self::Error> {
        // Delegate to Actor's post_stop lifecycle hook
        // Note: Timeout is not used here as Actor trait doesn't support it
        // If timeout enforcement is needed, implement Child explicitly
        self.post_stop().await
    }

    async fn health_check(&self) -> ChildHealth {
        // Default implementation: all actors are healthy
        // Actors can override this by implementing Child explicitly
        ChildHealth::Healthy
    }
}
```

### Pattern 1: Supervising Regular Actors (Zero Code Changes)

**Scenario**: Existing actors need supervision for fault tolerance

**Implementation**:
```rust
// ============================================================================
// NO CHANGES NEEDED TO ACTOR CODE
// ============================================================================

struct CounterActor {
    count: u32,
    max_count: u32,
}

#[async_trait]
impl Actor for CounterActor {
    type Message = CounterMsg;
    type Error = CounterError;
    
    async fn handle_message(
        &mut self, 
        msg: Self::Message, 
        _ctx: &mut ActorContext<Self::Message>
    ) -> Result<(), Self::Error> {
        if self.count >= self.max_count {
            return Err(CounterError::MaxReached);
        }
        self.count += msg.delta;
        Ok(())
    }
    
    // Optional: Actor lifecycle hooks still work
    async fn pre_start(&mut self) -> Result<(), Self::Error> {
        println!("Counter starting with max: {}", self.max_count);
        Ok(())
    }
    
    async fn post_stop(&mut self) -> Result<(), Self::Error> {
        println!("Counter stopped at count: {}", self.count);
        Ok(())
    }
}

// ============================================================================
// SUPERVISOR USAGE - Actor is automatically a Child
// ============================================================================

async fn setup_supervised_counter() -> Result<(), SupervisorError> {
    let monitor = InMemoryMonitor::new(MonitoringConfig::default());
    let mut supervisor = SupervisorNode::<OneForOne, _, _>::new(OneForOne, monitor);
    
    // Add actor to supervisor - works immediately via blanket impl
    supervisor.add_child(ChildSpec {
        id: "counter_actor".into(),
        factory: || CounterActor { count: 0, max_count: 100 },
        restart_policy: RestartPolicy::Permanent,
        shutdown_policy: ShutdownPolicy::Graceful(Duration::from_secs(5)),
        start_timeout: Duration::from_secs(10),
        shutdown_timeout: Duration::from_secs(10),
    }).await?;
    
    Ok(())
}
```

**Benefits**:
- ✅ Zero code changes to existing actors
- ✅ Automatic Child implementation via blanket impl
- ✅ Actor lifecycle hooks (pre_start, post_stop) reused
- ✅ Type-safe supervision with compile-time guarantees

### Pattern 2: Supervising Non-Actor Background Tasks

**Scenario**: Background tasks, compute workers, or long-running operations need supervision

**Implementation**:
```rust
use tokio::task::JoinHandle;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Background worker processing tasks from a queue
struct BackgroundWorker {
    name: String,
    task_queue: Arc<Mutex<VecDeque<WorkItem>>>,
    worker_handle: Option<JoinHandle<()>>,
    shutdown_signal: Option<tokio::sync::oneshot::Sender<()>>,
}

#[derive(Debug, thiserror::Error)]
enum WorkerError {
    #[error("Failed to start worker: {0}")]
    StartFailed(String),
    
    #[error("Shutdown timeout exceeded")]
    ShutdownTimeout,
    
    #[error("Worker panicked: {0}")]
    WorkerPanicked(String),
}

#[async_trait]
impl Child for BackgroundWorker {
    type Error = WorkerError;
    
    async fn start(&mut self) -> Result<(), Self::Error> {
        if self.worker_handle.is_some() {
            return Err(WorkerError::StartFailed("Already started".into()));
        }
        
        let queue = Arc::clone(&self.task_queue);
        let name = self.name.clone();
        let (shutdown_tx, mut shutdown_rx) = tokio::sync::oneshot::channel();
        
        let handle = tokio::spawn(async move {
            println!("[{}] Worker starting", name);
            
            loop {
                tokio::select! {
                    _ = &mut shutdown_rx => {
                        println!("[{}] Shutdown signal received", name);
                        break;
                    }
                    _ = tokio::time::sleep(Duration::from_millis(100)) => {
                        let mut queue = queue.lock().await;
                        if let Some(item) = queue.pop_front() {
                            println!("[{}] Processing: {:?}", name, item);
                            item.process().await;
                        }
                    }
                }
            }
            
            println!("[{}] Worker stopped", name);
        });
        
        self.worker_handle = Some(handle);
        self.shutdown_signal = Some(shutdown_tx);
        
        Ok(())
    }
    
    async fn stop(&mut self, timeout: Duration) -> Result<(), Self::Error> {
        // Send shutdown signal
        if let Some(signal) = self.shutdown_signal.take() {
            let _ = signal.send(());
        }
        
        // Wait for worker to finish with timeout
        if let Some(handle) = self.worker_handle.take() {
            tokio::select! {
                result = handle => {
                    result.map_err(|e| WorkerError::WorkerPanicked(e.to_string()))
                }
                _ = tokio::time::sleep(timeout) => {
                    Err(WorkerError::ShutdownTimeout)
                }
            }
        } else {
            Ok(())
        }
    }
    
    async fn health_check(&self) -> ChildHealth {
        // Check if worker is still running
        if self.worker_handle.is_none() {
            return ChildHealth::Failed("Worker handle is None".into());
        }
        
        // Check queue backlog
        let queue_size = self.task_queue.blocking_lock().len();
        if queue_size > 10000 {
            ChildHealth::Degraded(format!("Large queue backlog: {}", queue_size))
        } else {
            ChildHealth::Healthy
        }
    }
}
```

**Benefits**:
- ✅ Supervise non-message-passing tasks
- ✅ Graceful shutdown with timeout handling
- ✅ Health monitoring based on queue backlog
- ✅ Automatic restart on failure via supervisor

### Pattern 3: Supervising I/O Handlers

**Scenario**: File watchers, network listeners, or I/O resources need supervision

**Implementation**:
```rust
use std::path::PathBuf;
use notify::{Watcher, RecursiveMode, Event};
use tokio::sync::mpsc;

/// File system watcher that monitors path changes
struct FileWatcher {
    path: PathBuf,
    watcher: Option<notify::RecommendedWatcher>,
    event_tx: mpsc::Sender<Event>,
}

#[derive(Debug, thiserror::Error)]
enum FileWatcherError {
    #[error("Failed to create watcher: {0}")]
    WatcherCreationFailed(#[from] notify::Error),
    
    #[error("Path does not exist: {0}")]
    PathNotFound(PathBuf),
}

#[async_trait]
impl Child for FileWatcher {
    type Error = FileWatcherError;
    
    async fn start(&mut self) -> Result<(), Self::Error> {
        if !self.path.exists() {
            return Err(FileWatcherError::PathNotFound(self.path.clone()));
        }
        
        let tx = self.event_tx.clone();
        let mut watcher = notify::recommended_watcher(move |result: Result<Event, _>| {
            if let Ok(event) = result {
                let _ = tx.blocking_send(event);
            }
        })?;
        
        watcher.watch(&self.path, RecursiveMode::Recursive)?;
        self.watcher = Some(watcher);
        
        println!("FileWatcher started for path: {:?}", self.path);
        Ok(())
    }
    
    async fn stop(&mut self, _timeout: Duration) -> Result<(), Self::Error> {
        if let Some(mut watcher) = self.watcher.take() {
            let _ = watcher.unwatch(&self.path);
        }
        println!("FileWatcher stopped for path: {:?}", self.path);
        Ok(())
    }
    
    async fn health_check(&self) -> ChildHealth {
        if self.watcher.is_none() {
            ChildHealth::Failed("Watcher is not running".into())
        } else if !self.path.exists() {
            ChildHealth::Failed(format!("Watched path no longer exists: {:?}", self.path))
        } else {
            ChildHealth::Healthy
        }
    }
}
```

### Pattern 4: Mixed Supervision Tree (Actors + Tasks)

**Scenario**: Supervisor managing both actors and non-actor entities

**Implementation**:
```rust
async fn setup_mixed_supervision_tree() -> Result<(), SupervisorError> {
    let monitor = InMemoryMonitor::new(MonitoringConfig::default());
    
    // Create supervisor with OneForAll strategy
    // If any child fails, restart all children
    let mut supervisor = SupervisorNode::<OneForAll, _, _>::new(
        OneForAll,
        monitor,
    );
    
    // ========================================================================
    // Child 1: Actor (automatic Child via blanket impl)
    // ========================================================================
    supervisor.add_child(ChildSpec {
        id: "counter_actor".into(),
        factory: || CounterActor { count: 0, max_count: 1000 },
        restart_policy: RestartPolicy::Permanent,
        shutdown_policy: ShutdownPolicy::Graceful(Duration::from_secs(5)),
        start_timeout: Duration::from_secs(10),
        shutdown_timeout: Duration::from_secs(10),
    }).await?;
    
    // ========================================================================
    // Child 2: Background Worker (explicit Child implementation)
    // ========================================================================
    let task_queue = Arc::new(Mutex::new(VecDeque::new()));
    supervisor.add_child(ChildSpec {
        id: "background_worker".into(),
        factory: {
            let queue = Arc::clone(&task_queue);
            move || BackgroundWorker {
                name: "worker-1".into(),
                task_queue: Arc::clone(&queue),
                worker_handle: None,
                shutdown_signal: None,
            }
        },
        restart_policy: RestartPolicy::Permanent,
        shutdown_policy: ShutdownPolicy::Graceful(Duration::from_secs(10)),
        start_timeout: Duration::from_secs(5),
        shutdown_timeout: Duration::from_secs(10),
    }).await?;
    
    // ========================================================================
    // Child 3: File Watcher (explicit Child implementation)
    // ========================================================================
    let (event_tx, mut event_rx) = mpsc::channel(100);
    supervisor.add_child(ChildSpec {
        id: "file_watcher".into(),
        factory: {
            let tx = event_tx.clone();
            move || FileWatcher {
                path: PathBuf::from("/tmp/watched"),
                watcher: None,
                event_tx: tx.clone(),
            }
        },
        restart_policy: RestartPolicy::Transient,  // Only restart if abnormal exit
        shutdown_policy: ShutdownPolicy::Immediate,  // File watchers can stop immediately
        start_timeout: Duration::from_secs(5),
        shutdown_timeout: Duration::from_secs(2),
    }).await?;
    
    // ✅ Supervisor now manages 3 heterogeneous children:
    //    - 1 Actor (automatic Child)
    //    - 1 Background Task (explicit Child)
    //    - 1 I/O Handler (explicit Child)
    
    println!("Mixed supervision tree started with {} children", supervisor.child_count());
    Ok(())
}
```

### Pattern 5: Custom Health Checking for Actors

**Scenario**: Actor needs custom health checking beyond default "always healthy"

**Implementation**:
```rust
/// Actor with custom health checking logic
struct HealthAwareActor {
    error_count: AtomicU32,
    error_threshold: u32,
    last_success: Arc<Mutex<DateTime<Utc>>>,
}

#[async_trait]
impl Actor for HealthAwareActor {
    type Message = HealthMsg;
    type Error = HealthError;
    
    async fn handle_message(
        &mut self, 
        msg: Self::Message, 
        _ctx: &mut ActorContext<Self::Message>
    ) -> Result<(), Self::Error> {
        match msg.process() {
            Ok(_) => {
                self.error_count.store(0, Ordering::Relaxed);
                *self.last_success.lock().await = Utc::now();
                Ok(())
            }
            Err(e) => {
                self.error_count.fetch_add(1, Ordering::Relaxed);
                Err(HealthError::ProcessingFailed(e))
            }
        }
    }
}

// ============================================================================
// Override Child implementation for custom health checking
// ============================================================================

#[async_trait]
impl Child for HealthAwareActor {
    type Error = HealthError;
    
    async fn start(&mut self) -> Result<(), Self::Error> {
        // Delegate to Actor lifecycle
        self.pre_start().await
    }
    
    async fn stop(&mut self, _timeout: Duration) -> Result<(), Self::Error> {
        // Delegate to Actor lifecycle
        self.post_stop().await
    }
    
    async fn health_check(&self) -> ChildHealth {
        let error_count = self.error_count.load(Ordering::Relaxed);
        let last_success = *self.last_success.lock().await;
        let time_since_success = Utc::now().signed_duration_since(last_success);
        
        // Failed if error threshold exceeded
        if error_count >= self.error_threshold {
            return ChildHealth::Failed(
                format!("Error threshold exceeded: {} errors", error_count)
            );
        }
        
        // Degraded if no success in last 60 seconds
        if time_since_success.num_seconds() > 60 {
            return ChildHealth::Degraded(
                format!("No successful processing in {}s", time_since_success.num_seconds())
            );
        }
        
        // Degraded if error rate is high but not critical
        if error_count > self.error_threshold / 2 {
            return ChildHealth::Degraded(
                format!("Elevated error rate: {} errors", error_count)
            );
        }
        
        ChildHealth::Healthy
    }
}
```

**Benefits**:
- ✅ Custom health logic for specific actor needs
- ✅ Proactive failure detection before complete failure
- ✅ Supervisor can restart degraded actors preemptively
- ✅ Explicit Child impl overrides blanket impl

## Best Practices

### 1. **Prefer Blanket Impl for Actors** ⭐⭐⭐⭐⭐
- Use automatic Child implementation for most actors
- Only implement Child explicitly when custom health checking is needed
- Keep Actor trait focused on message handling

### 2. **Idempotent start() Methods** ⭐⭐⭐⭐
- Make start() safe to call multiple times
- Check if already started and return appropriate result
- Avoid resource leaks on repeated starts

### 3. **Graceful Shutdown with Timeout** ⭐⭐⭐⭐⭐
- Always honor the timeout parameter in stop()
- Use `tokio::select!` for timeout enforcement
- Clean up resources even on timeout

### 4. **Meaningful Health Checks** ⭐⭐⭐⭐
- Provide actionable health status information
- Include metrics in degraded/failed reasons
- Enable proactive restart before complete failure

### 5. **Error Context** ⭐⭐⭐⭐
- Use structured errors with context (thiserror)
- Include entity name/ID in error messages
- Log lifecycle events for debugging

## Performance Characteristics

### Blanket Implementation Overhead
**ZERO RUNTIME OVERHEAD** ✅
- Monomorphization eliminates abstraction cost
- Compiles to direct Actor method calls
- No vtable lookups or dynamic dispatch
- Static dispatch throughout

### Health Check Performance
- Default impl: Instant (returns constant)
- Custom impl: Depends on implementation
- Recommendation: Keep health checks fast (<1ms)
- Use atomic operations for metrics

## Testing Patterns

### Testing Actors with Supervision

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_actor_supervision_lifecycle() {
        let monitor = NoopMonitor::new();  // Zero overhead for tests
        let mut supervisor = SupervisorNode::<OneForOne, _, _>::new(
            OneForOne,
            monitor,
        );
        
        supervisor.add_child(ChildSpec {
            id: "test_actor".into(),
            factory: || TestActor::new(),
            restart_policy: RestartPolicy::Permanent,
            shutdown_policy: ShutdownPolicy::Graceful(Duration::from_secs(1)),
            start_timeout: Duration::from_secs(5),
            shutdown_timeout: Duration::from_secs(5),
        }).await.unwrap();
        
        // ✅ Actor is started automatically
        // ✅ Actor can be supervised like any other Child
    }
}
```

### Testing Custom Child Implementations

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_background_worker_lifecycle() {
        let queue = Arc::new(Mutex::new(VecDeque::new()));
        let mut worker = BackgroundWorker {
            name: "test-worker".into(),
            task_queue: Arc::clone(&queue),
            worker_handle: None,
            shutdown_signal: None,
        };
        
        // Test start
        worker.start().await.unwrap();
        assert!(worker.worker_handle.is_some());
        
        // Test health check
        assert!(worker.health_check().await.is_healthy());
        
        // Test stop
        worker.stop(Duration::from_secs(1)).await.unwrap();
        assert!(worker.worker_handle.is_none());
    }
}
```

## Migration Guide

### For Existing Actors
**NO CHANGES NEEDED** ✅

Existing actors automatically become supervisable:

```rust
// Before: Actor without supervision
struct MyActor { state: u32 }

#[async_trait]
impl Actor for MyActor {
    type Message = MyMessage;
    type Error = MyError;
    
    async fn handle_message(&mut self, msg: Self::Message, ctx: &mut ActorContext<Self::Message>) 
        -> Result<(), Self::Error> 
    {
        // ... existing logic ...
        Ok(())
    }
}

// After: Same actor, now supervisable - NO CODE CHANGES!
// ✅ MyActor automatically implements Child via blanket impl
// ✅ Can be added to supervisor immediately
// ✅ Existing lifecycle hooks (pre_start, post_stop) reused
```

### For Custom Child Types
Implement Child trait explicitly:

```rust
#[async_trait]
impl Child for MyCustomEntity {
    type Error = MyError;
    
    async fn start(&mut self) -> Result<(), Self::Error> {
        // Custom start logic
        Ok(())
    }
    
    async fn stop(&mut self, timeout: Duration) -> Result<(), Self::Error> {
        // Custom stop logic with timeout
        Ok(())
    }
    
    async fn health_check(&self) -> ChildHealth {
        // Optional: custom health logic
        ChildHealth::Healthy
    }
}
```

## Integration with Supervisor

### Generic Supervision (All Child Types)

```rust
pub struct SupervisorNode<S, C, M>
where
    S: SupervisionStrategy,
    C: Child,  // ← Works with ANY Child (actors or non-actors)
    M: Monitor<SupervisionEvent>,
{
    strategy: S,
    children: HashMap<ChildId, ChildHandle<C>>,
    monitor: M,
}

impl<S, C, M> SupervisorNode<S, C, M>
where
    S: SupervisionStrategy,
    C: Child,
    M: Monitor<SupervisionEvent>,
{
    pub async fn add_child<F>(&mut self, spec: ChildSpec<C, F>) -> Result<ChildId, SupervisorError>
    where
        F: Fn() -> C + Send + Sync + 'static,
    {
        let mut child = (spec.factory)();
        
        // Start child using Child trait method
        child.start().await.map_err(|e| {
            SupervisorError::ChildStartFailed {
                id: spec.id.clone(),
                source: Box::new(e),
            }
        })?;
        
        // Monitor supervision event
        self.monitor.record(SupervisionEvent {
            kind: SupervisionEventKind::ChildStarted,
            child_id: spec.id.clone(),
            timestamp: Utc::now(),
            severity: EventSeverity::Info,
        }).await?;
        
        // Store child
        let child_id = ChildId::new();
        self.children.insert(child_id.clone(), ChildHandle {
            id: spec.id,
            child,
            state: ChildState::Running,
            restart_count: 0,
            last_restart: None,
        });
        
        Ok(child_id)
    }
}
```

## Conclusion

The separate Child trait architecture provides:

✅ **Maximum Flexibility**: Supervise actors, tasks, I/O handlers, any entity  
✅ **Zero Breaking Changes**: Blanket impl makes all actors automatically supervisable  
✅ **BEAM Alignment**: True Erlang/OTP supervisor philosophy  
✅ **Clean Architecture**: Clear separation between message handling and lifecycle  
✅ **Zero Performance Overhead**: Static dispatch via generic constraints  
✅ **Future-Proof**: Easy integration with WASM components, OSL services  

This design follows workspace standards (§6.2 avoid dyn, §6.1 YAGNI) and Microsoft Rust Guidelines (M-DI-HIERARCHY), providing production-ready fault tolerance for the airssys-rt actor system.

## References

- **ADR-RT-004**: Child Trait Separation from Actor Trait (design decision)
- **KNOWLEDGE-RT-003**: Supervisor Tree Implementation Strategies
- **KNOWLEDGE-RT-013**: RT-TASK-007 and RT-TASK-010 Action Plans
- **Workspace Standards**: §6.2 (Avoid dyn), §6.1 (YAGNI), §2.1 (Import organization)
- **Microsoft Rust Guidelines**: M-DI-HIERARCHY, M-SERVICES-CLONE
- **Erlang/OTP**: Supervisor behavior documentation

---

**Status**: ✅ **Active** - Ready for RT-TASK-007 Phase 1 implementation
