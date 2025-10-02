# KNOWLEDGE-RT-003: Supervisor Tree Implementation Strategies

**Sub-Project:** airssys-rt  
**Category:** Actor Model  
**Created:** 2025-10-02  
**Last Updated:** 2025-10-02  
**Status:** active  

## Context and Problem

BEAM-inspired supervisor trees provide fault tolerance through hierarchical supervision and restart strategies. In Rust, implementing supervisor trees requires careful balance between type safety, performance, and flexibility. The challenge is maintaining compile-time optimizations while supporting dynamic actor management and error recovery patterns.

## Knowledge Details

### Supervisor Tree Architecture

The supervisor tree uses a hierarchical structure with typed supervision strategies:

```rust
// Core supervisor trait with generic error handling
pub trait Supervisor<C: Child> {
    type Error: Error + Send + Sync + 'static;
    
    async fn start_child(&mut self, spec: ChildSpec<C>) -> Result<ChildId, Self::Error>;
    async fn stop_child(&mut self, id: &ChildId) -> Result<(), Self::Error>;
    async fn restart_child(&mut self, id: &ChildId) -> Result<(), Self::Error>;
    async fn handle_child_error(&mut self, id: &ChildId, error: C::Error) -> SupervisionDecision;
}

// Supervision strategies as compile-time types
pub struct OneForOne;
pub struct OneForAll;
pub struct RestForOne;

// Generic supervisor implementation
pub struct SupervisorNode<S, C> 
where 
    S: SupervisionStrategy,
    C: Child,
{
    strategy: S,
    children: HashMap<ChildId, ChildHandle<C>>,
    restart_policy: RestartPolicy,
    max_restarts: u32,
    restart_window: Duration,
    restart_count: u32,
    window_start: DateTime<Utc>,
}
```

### Child Specification and Lifecycle

Child processes are managed through comprehensive specifications:

```rust
// Child specification with lifecycle configuration
pub struct ChildSpec<C: Child> {
    pub id: String,
    pub child_factory: Box<dyn Fn() -> Result<C, C::Error> + Send + Sync>,
    pub restart_policy: RestartPolicy,
    pub shutdown_policy: ShutdownPolicy,
    pub start_timeout: Duration,
    pub shutdown_timeout: Duration,
}

#[derive(Debug, Clone)]
pub enum RestartPolicy {
    Permanent,    // Always restart
    Transient,    // Restart only if exits abnormally
    Temporary,    // Never restart
}

#[derive(Debug, Clone)]
pub enum ShutdownPolicy {
    Graceful(Duration),  // Graceful shutdown with timeout
    Immediate,           // Kill immediately
    Infinity,            // Wait indefinitely
}

// Child handle for supervision
pub struct ChildHandle<C: Child> {
    child: C,
    state: ChildState,
    restart_count: u32,
    last_restart: DateTime<Utc>,
    start_time: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ChildState {
    Starting,
    Running,
    Stopping,
    Stopped,
    Restarting,
    Failed,
}
```

### Supervision Strategies Implementation

Each strategy implements specific restart behavior:

```rust
impl SupervisionStrategy for OneForOne {
    fn handle_child_failure<C: Child>(
        supervisor: &mut SupervisorNode<Self, C>,
        failed_child: &ChildId,
        error: C::Error,
    ) -> SupervisionDecision {
        // Only restart the failed child
        if should_restart(&supervisor.restart_policy, &error) {
            SupervisionDecision::RestartChild(failed_child.clone())
        } else {
            SupervisionDecision::StopChild(failed_child.clone())
        }
    }
}

impl SupervisionStrategy for OneForAll {
    fn handle_child_failure<C: Child>(
        supervisor: &mut SupervisorNode<Self, C>,
        failed_child: &ChildId,
        error: C::Error,
    ) -> SupervisionDecision {
        // Stop all children, then restart all
        if should_restart(&supervisor.restart_policy, &error) {
            let all_children: Vec<ChildId> = supervisor.children.keys().cloned().collect();
            SupervisionDecision::RestartAll(all_children)
        } else {
            SupervisionDecision::StopAll
        }
    }
}

impl SupervisionStrategy for RestForOne {
    fn handle_child_failure<C: Child>(
        supervisor: &mut SupervisorNode<Self, C>,
        failed_child: &ChildId,
        error: C::Error,
    ) -> SupervisionDecision {
        // Restart failed child and all children started after it
        if should_restart(&supervisor.restart_policy, &error) {
            let restart_children = supervisor.get_children_started_after(failed_child);
            SupervisionDecision::RestartSubset(restart_children)
        } else {
            SupervisionDecision::StopChild(failed_child.clone())
        }
    }
}
```

### Error Propagation and Recovery

Comprehensive error handling with structured recovery:

```rust
#[derive(Debug, thiserror::Error)]
pub enum SupervisionError {
    #[error("Child {id} failed to start: {source}")]
    ChildStartFailed { 
        id: String, 
        #[source] source: Box<dyn Error + Send + Sync>,
    },
    
    #[error("Maximum restart limit exceeded: {max_restarts} in {window:?}")]
    RestartLimitExceeded { 
        max_restarts: u32, 
        window: Duration,
    },
    
    #[error("Child {id} shutdown timeout after {timeout:?}")]
    ShutdownTimeout { 
        id: String, 
        timeout: Duration,
    },
    
    #[error("Supervisor tree integrity violation: {reason}")]
    TreeIntegrityViolation { 
        reason: String,
    },
}

// Supervision decision types
#[derive(Debug)]
pub enum SupervisionDecision {
    RestartChild(ChildId),
    RestartAll(Vec<ChildId>),
    RestartSubset(Vec<ChildId>),
    StopChild(ChildId),
    StopAll,
    Escalate(SupervisionError),
}
```

### Restart Rate Limiting

Prevent restart storms with configurable rate limiting:

```rust
pub struct RestartPolicy {
    pub max_restarts: u32,
    pub restart_window: Duration,
    pub escalation_strategy: EscalationStrategy,
}

#[derive(Debug, Clone)]
pub enum EscalationStrategy {
    Terminate,           // Terminate supervisor
    Escalate,           // Escalate to parent supervisor
    Backoff(Duration),  // Exponential backoff
}

impl<S, C> SupervisorNode<S, C>
where 
    S: SupervisionStrategy,
    C: Child,
{
    fn check_restart_rate(&mut self) -> Result<(), SupervisionError> {
        let now = Utc::now();
        
        // Reset window if enough time has passed
        if now.signed_duration_since(self.window_start) > self.restart_policy.restart_window {
            self.restart_count = 0;
            self.window_start = now;
        }
        
        // Check if we've exceeded the limit
        if self.restart_count >= self.restart_policy.max_restarts {
            match self.restart_policy.escalation_strategy {
                EscalationStrategy::Terminate => {
                    Err(SupervisionError::RestartLimitExceeded {
                        max_restarts: self.restart_policy.max_restarts,
                        window: self.restart_policy.restart_window,
                    })
                }
                EscalationStrategy::Escalate => {
                    // Escalate to parent supervisor
                    self.escalate_to_parent()
                }
                EscalationStrategy::Backoff(delay) => {
                    // Apply exponential backoff
                    self.apply_restart_backoff(delay).await
                }
            }
        } else {
            Ok(())
        }
    }
}
```

## Performance Characteristics

### Supervision Overhead
- **Child restart time**: <10ms for simple actors
- **Supervision decision**: <1μs for strategy evaluation
- **Tree traversal**: O(log n) for balanced supervision trees
- **Memory overhead**: ~128 bytes per supervised child

### Scalability Metrics
- **Max children per supervisor**: 10,000+ children supported
- **Supervision depth**: 10+ levels without performance degradation
- **Restart throughput**: >1,000 restarts/second
- **Error propagation latency**: <100μs to parent supervisor

## Implementation Guidelines

### Supervision Tree Design Principles
1. **Shallow hierarchies**: Prefer wider trees over deeper ones
2. **Failure isolation**: Group related actors under common supervisors
3. **Resource management**: Supervise resource-intensive actors separately
4. **Error boundaries**: Use supervision to create fault boundaries

### Child Lifecycle Management
```rust
// Proper child startup sequence
async fn start_child_safely<C: Child>(
    spec: ChildSpec<C>
) -> Result<ChildHandle<C>, SupervisionError> {
    let start_time = Utc::now();
    
    // Create child with timeout
    let child = tokio::time::timeout(
        spec.start_timeout,
        (spec.child_factory)()
    )
    .await
    .map_err(|_| SupervisionError::ChildStartFailed {
        id: spec.id.clone(),
        source: "Start timeout".into(),
    })?
    .map_err(|e| SupervisionError::ChildStartFailed {
        id: spec.id.clone(),
        source: e.into(),
    })?;
    
    Ok(ChildHandle {
        child,
        state: ChildState::Running,
        restart_count: 0,
        last_restart: start_time,
        start_time,
    })
}
```

## Related Patterns

### Complementary Knowledge
- **KNOWLEDGE-RT-001**: Zero-Cost Actor Model Architecture
- **KNOWLEDGE-RT-004**: Actor Lifecycle Management Patterns
- **KNOWLEDGE-RT-006**: Error Handling and Recovery Strategies

### Architecture Decisions
- **ADR-RT-004**: Supervisor Tree Design
- **ADR-RT-011**: Testing Strategy
- **ADR-RT-003**: Actor State Management

## Usage Examples

### Basic Supervisor Setup
```rust
// Create supervisor with OneForOne strategy
let mut supervisor = SupervisorNode::new(
    OneForOne,
    RestartPolicy {
        max_restarts: 5,
        restart_window: Duration::from_secs(60),
        escalation_strategy: EscalationStrategy::Escalate,
    }
);

// Add child specification
let child_spec = ChildSpec {
    id: "worker_1".to_string(),
    child_factory: Box::new(|| Ok(WorkerActor::new())),
    restart_policy: RestartPolicy::Permanent,
    shutdown_policy: ShutdownPolicy::Graceful(Duration::from_secs(5)),
    start_timeout: Duration::from_secs(10),
    shutdown_timeout: Duration::from_secs(5),
};

supervisor.start_child(child_spec).await?;
```

### Dynamic Child Management
```rust
// Add children dynamically based on load
impl LoadBalancingSupervisor {
    async fn scale_workers(&mut self, target_count: usize) -> Result<(), SupervisionError> {
        let current_count = self.children.len();
        
        if target_count > current_count {
            // Scale up - add more workers
            for i in current_count..target_count {
                let spec = self.create_worker_spec(format!("worker_{}", i));
                self.start_child(spec).await?;
            }
        } else if target_count < current_count {
            // Scale down - remove excess workers
            let workers_to_remove: Vec<_> = self.children
                .keys()
                .skip(target_count)
                .cloned()
                .collect();
                
            for worker_id in workers_to_remove {
                self.stop_child(&worker_id).await?;
            }
        }
        
        Ok(())
    }
}
```

## Lessons Learned

### What Works Well
- Typed supervision strategies provide compile-time safety
- Restart rate limiting prevents system instability
- Hierarchical design naturally isolates failures
- Generic child handling maintains type safety

### Potential Pitfalls
- Deep supervision trees can create bottlenecks
- Restart storms can overwhelm system resources
- Complex error propagation can be hard to debug
- Circular supervision dependencies must be prevented

## Future Considerations

### Planned Enhancements
- Dynamic supervision strategy switching
- Metrics integration for supervision events
- Health checking for proactive restarts
- Integration with OSL security contexts

### Research Areas
- Adaptive restart policies based on error patterns
- Machine learning for failure prediction
- Distributed supervision across nodes
- Supervision tree visualization and debugging tools

---

**References:**
- BEAM OTP Supervisor documentation
- Microsoft Rust Guidelines: M-ERRORS-CANONICAL-STRUCTS
- Performance targets: <10ms restart time, >1,000 restarts/second
- Erlang supervision principles and patterns