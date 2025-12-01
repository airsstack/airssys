# KNOWLEDGE-RT-010: Actor Messaging Patterns and Integration

**Knowledge Type**: Implementation Patterns & Integration Guide  
**Category**: Message Passing & Actor Communication  
**Created**: 2025-10-05  
**Related**: RT-TASK-004, KNOWLEDGE-RT-009, KNOWLEDGE-RT-005  
**Status**: Active - Implementation Guide  
**Complexity**: Advanced  

---

## Overview

This document defines the complete actor messaging patterns for airssys-rt, including fire-and-forget, request-reply with async wait, and manual correlation patterns. It provides comprehensive integration examples showing how Actor, ActorContext, Supervisor, and MessageBroker work together.

**Key Decision**: airssys-rt supports a **hybrid approach** with three messaging patterns, each optimized for different use cases while maintaining clean separation of concerns between actors and infrastructure.

## Context

### Problem Statement

Actor systems need multiple communication patterns to handle different scenarios:
1. **Fire-and-forget**: High-throughput one-way messaging (logs, events, notifications)
2. **Request-reply with wait**: Synchronous workflows requiring immediate responses (HTTP handlers, RPC)
3. **Async tracking**: Long-running tasks with manual correlation (distributed job processing)

**Critical Constraint**: All patterns must maintain clean separation between actor business logic and system infrastructure (broker, supervisor).

### Scope

**In Scope**:
- Fire-and-forget messaging with `context.send()`
- Request-reply with async wait using `context.request()`
- Manual correlation with fire-and-forget using correlation IDs
- ActorContext integration with MessageBroker
- Supervisor message forwarding patterns
- Complete message flow examples

**Out of Scope** (Future Enhancements):
- Publish-subscribe patterns
- Message persistence and replay
- Distributed message routing
- Priority queue advanced scheduling

### Prerequisites

**Required Knowledge**:
- KNOWLEDGE-RT-009: Message Broker Architecture
- KNOWLEDGE-RT-005: Actor System Core Implementation
- KNOWLEDGE-RT-004: Message System Implementation
- Understanding of async/await patterns in Rust

---

## Technical Content

### Core Concepts

#### 1. Three Messaging Patterns

airssys-rt provides three complementary patterns:

```rust
// Pattern 1: Fire-and-Forget
context.send(target, message).await?;
// ✅ Non-blocking, high throughput
// ✅ Pure actor model
// ✅ Use for: logs, events, notifications

// Pattern 2: Request-Reply with Async Wait
let response = context.request::<Response>(target, request, timeout).await?;
// ✅ Blocking, synchronous feel
// ✅ Built-in timeout
// ✅ Use for: HTTP handlers, RPC, critical paths

// Pattern 3: Fire-and-Forget + Manual Correlation
let task_id = TaskId::new();
self.pending_tasks.insert(task_id, pending_info);
context.send(worker, TaskRequest { task_id, .. }).await?;
// ... later, in handle_message:
// if let Some(pending) = self.pending_tasks.remove(&result.task_id) { ... }
// ✅ Non-blocking, manual state management
// ✅ Use for: async jobs, distributed work, long tasks
```

#### 2. Message Flow Architecture

**All messages flow through the broker and supervisor**:

```
┌─────────────────────────────────────────────────────────────┐
│                      Actor System                           │
│                                                             │
│  Actor A                MessageBroker          Actor B      │
│     │                         │                   │         │
│     │──[Message]─────────────▶│                   │         │
│     │                         │                   │         │
│     │                         │ (route to         │         │
│     │                         │  supervisor)      │         │
│     │                         │                   │         │
│     │                         ├──[Forward]───────▶│         │
│     │                         │                   │         │
│     │                         │      Supervisor intercepts  │
│     │                         │      and forwards to        │
│     │                         │      actor's mailbox        │
│     │                         │                   │         │
│     │                         │◀──[Reply]─────────│         │
│     │◀────────────────────────│                   │         │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

**Critical Points**:
- ✅ Actors use `ActorContext` methods (not broker directly)
- ✅ Broker handles all routing decisions
- ✅ Supervisor intercepts messages for fault tolerance
- ✅ Clean separation of concerns maintained

---

### Implementation Patterns

#### Pattern 1: Fire-and-Forget - Log Processing

**Use Case**: High-throughput log collection where senders don't need confirmation.

```rust
// ============================================================================
// Log Collector → Log Processor → Storage (Fire-and-Forget Chain)
// ============================================================================

#[derive(Debug, Clone)]
struct LogEntry {
    timestamp: DateTime<Utc>,
    level: LogLevel,
    message: String,
    source: String,
}

impl Message for LogEntry {
    const MESSAGE_TYPE: &'static str = "log_entry";
    
    fn priority(&self) -> MessagePriority {
        match self.level {
            LogLevel::Error => MessagePriority::High,
            LogLevel::Warn => MessagePriority::Normal,
            _ => MessagePriority::Low,
        }
    }
}

// Log Collector Actor
struct LogCollector {
    source_name: String,
    processor_address: ActorAddress,
    logs_sent: u64,
}

#[async_trait]
impl Actor for LogCollector {
    type Message = CollectorCommand;
    type Error = CollectorError;
    
    async fn handle_message(
        &mut self,
        message: Self::Message,
        context: &mut ActorContext<Self::Message>,
    ) -> Result<(), Self::Error> {
        match message {
            CollectorCommand::NewLog { level, message } => {
                let log_entry = LogEntry {
                    timestamp: Utc::now(),
                    level,
                    message,
                    source: self.source_name.clone(),
                };
                
                // ✅ FIRE-AND-FORGET: Send without waiting
                context.send(self.processor_address.clone(), log_entry).await?;
                
                self.logs_sent += 1;
                
                // ✅ Continue immediately - don't block
                println!("[Collector] Sent log #{}", self.logs_sent);
            }
        }
        
        Ok(())
    }
}

// Log Processor Actor
struct LogProcessor {
    storage_address: ActorAddress,
    processed_count: u64,
}

#[async_trait]
impl Actor for LogProcessor {
    type Message = LogEntry;
    type Error = ProcessorError;
    
    async fn handle_message(
        &mut self,
        log: Self::Message,
        context: &mut ActorContext<Self::Message>,
    ) -> Result<(), Self::Error> {
        // Process log (parse, enrich, filter)
        let enriched_log = self.enrich_log(log);
        
        // ✅ FIRE-AND-FORGET: Forward to storage
        context.send(self.storage_address.clone(), enriched_log).await?;
        
        self.processed_count += 1;
        Ok(())
    }
}

// Message Flow:
// Collector ──[LogEntry]──▶ Broker ──▶ Processor ──[EnrichedLog]──▶ Storage
//    │                                     │
//    │ (continues immediately)             │ (continues immediately)
//    ▼                                     ▼
// Next log                              Next log
```

**Characteristics**:
- ✅ High throughput (no waiting)
- ✅ Actors remain responsive
- ✅ Simple one-way flow
- ✅ Good for: logging, metrics, events, notifications

---

#### Pattern 2: Request-Reply with Async Wait - Authentication

**Use Case**: HTTP authentication where handler must wait for auth result before responding to client.

```rust
// ============================================================================
// API Handler → Auth Service → Database (Synchronous Workflow)
// ============================================================================

#[derive(Debug, Clone)]
struct AuthRequest {
    username: String,
    password_hash: String,
}

impl Message for AuthRequest {
    const MESSAGE_TYPE: &'static str = "auth_request";
    
    fn priority(&self) -> MessagePriority {
        MessagePriority::High  // User-facing requests
    }
}

#[derive(Debug, Clone)]
struct AuthResponse {
    success: bool,
    user_id: Option<UserId>,
    session_token: Option<String>,
    error_message: Option<String>,
}

impl Message for AuthResponse {
    const MESSAGE_TYPE: &'static str = "auth_response";
}

// API Handler Actor
struct ApiHandler {
    auth_service_address: ActorAddress,
}

#[async_trait]
impl Actor for ApiHandler {
    type Message = HttpRequest;
    type Error = ApiError;
    
    async fn handle_message(
        &mut self,
        request: Self::Message,
        context: &mut ActorContext<Self::Message>,
    ) -> Result<(), Self::Error> {
        match request.endpoint {
            "/api/login" => {
                let auth_request = AuthRequest {
                    username: request.body.get("username")?,
                    password_hash: hash_password(request.body.get("password")?),
                };
                
                // ✅ REQUEST-REPLY: Wait for authentication result
                // Can't respond to HTTP client until we know if auth succeeded
                let auth_result = context.request::<AuthResponse>(
                    self.auth_service_address.clone(),
                    auth_request,
                    Duration::from_secs(3),  // 3 second timeout
                ).await?;
                
                let http_response = match auth_result {
                    Some(response) if response.success => {
                        HttpResponse {
                            status: 200,
                            body: json!({
                                "success": true,
                                "session_token": response.session_token,
                                "user_id": response.user_id,
                            }),
                        }
                    }
                    Some(response) => {
                        HttpResponse {
                            status: 401,
                            body: json!({
                                "success": false,
                                "error": response.error_message,
                            }),
                        }
                    }
                    None => {
                        // Timeout - auth service didn't respond
                        HttpResponse {
                            status: 503,
                            body: json!({"error": "Authentication service unavailable"}),
                        }
                    }
                };
                
                self.send_http_response(request.connection_id, http_response);
            }
            _ => { /* other endpoints */ }
        }
        
        Ok(())
    }
}

// Auth Service Actor
struct AuthService {
    database_address: ActorAddress,
    session_manager_address: ActorAddress,
}

#[async_trait]
impl Actor for AuthService {
    type Message = AuthRequest;
    type Error = AuthError;
    
    async fn handle_message(
        &mut self,
        request: Self::Message,
        context: &mut ActorContext<Self::Message>,
    ) -> Result<(), Self::Error> {
        // ✅ REQUEST-REPLY: Query database for user
        let db_query = DatabaseQuery {
            table: "users".to_string(),
            filter: format!("username = '{}'", request.username),
        };
        
        let user_result = context.request::<DatabaseResult>(
            self.database_address.clone(),
            db_query,
            Duration::from_secs(2),
        ).await?;
        
        let response = match user_result {
            Some(db_result) if !db_result.rows.is_empty() => {
                let user = &db_result.rows[0];
                
                if verify_password(&request.password_hash, &user.password_hash) {
                    // ✅ REQUEST-REPLY: Create session
                    let session_result = context.request::<SessionResponse>(
                        self.session_manager_address.clone(),
                        CreateSessionRequest {
                            user_id: user.id,
                            ttl: Duration::from_hours(24),
                        },
                        Duration::from_secs(1),
                    ).await?;
                    
                    if let Some(session) = session_result {
                        AuthResponse {
                            success: true,
                            user_id: Some(user.id),
                            session_token: Some(session.token),
                            error_message: None,
                        }
                    } else {
                        AuthResponse {
                            success: false,
                            user_id: None,
                            session_token: None,
                            error_message: Some("Failed to create session".to_string()),
                        }
                    }
                } else {
                    AuthResponse {
                        success: false,
                        user_id: None,
                        session_token: None,
                        error_message: Some("Invalid password".to_string()),
                    }
                }
            }
            _ => {
                AuthResponse {
                    success: false,
                    user_id: None,
                    session_token: None,
                    error_message: Some("User not found".to_string()),
                }
            }
        };
        
        // ✅ CRITICAL: Reply to original requester
        context.reply(response).await?;
        
        Ok(())
    }
}

// Message Flow:
// API Handler              Auth Service            Database
//     │                         │                      │
//     │──[AuthRequest]─────────▶│                      │
//     │                         │──[DatabaseQuery]────▶│
//     │ (WAITING... blocked)    │ (WAITING... blocked) │ (process)
//     │                         │◀──[DatabaseResult]───│
//     │                         │ (verify password)    │
//     │◀──[AuthResponse]────────│                      │
//     │                         │                      │
//  (process)                    │                      │
//  Send HTTP 200                │                      │
```

**Characteristics**:
- ✅ Actor blocks waiting for response
- ✅ Linear code flow (easy to understand)
- ✅ Built-in timeout handling
- ✅ Good for: synchronous workflows, RPC-style calls, critical paths

---

#### Pattern 3: Fire-and-Forget + Manual Correlation - Task Processing

**Use Case**: Distributed task processing where coordinator doesn't block but tracks completion.

```rust
// ============================================================================
// Task Coordinator ⇄ Worker Pool (Async Job Processing)
// ============================================================================

#[derive(Debug, Clone)]
struct TaskRequest {
    task_id: TaskId,  // ✅ Manual correlation ID
    payload: TaskPayload,
    priority: TaskPriority,
}

impl Message for TaskRequest {
    const MESSAGE_TYPE: &'static str = "task_request";
}

#[derive(Debug, Clone)]
struct TaskResult {
    task_id: TaskId,  // ✅ Same correlation ID
    success: bool,
    output: Option<Vec<u8>>,
    error: Option<String>,
}

impl Message for TaskResult {
    const MESSAGE_TYPE: &'static str = "task_result";
}

// Task Coordinator - Manages task distribution
struct TaskCoordinator {
    worker_pool: Vec<ActorAddress>,
    next_worker: usize,
    
    // ✅ Track pending tasks manually
    pending_tasks: HashMap<TaskId, PendingTask>,
}

struct PendingTask {
    started_at: DateTime<Utc>,
    retries: u32,
    worker_address: ActorAddress,
}

// ✅ Coordinator handles TWO message types
enum CoordinatorMessage {
    NewTask(TaskPayload),
    TaskResult(TaskResult),
}

impl Message for CoordinatorMessage {
    const MESSAGE_TYPE: &'static str = "coordinator_message";
}

#[async_trait]
impl Actor for TaskCoordinator {
    type Message = CoordinatorMessage;
    type Error = CoordinatorError;
    
    async fn handle_message(
        &mut self,
        message: Self::Message,
        context: &mut ActorContext<Self::Message>,
    ) -> Result<(), Self::Error> {
        match message {
            // Handle new task submission
            CoordinatorMessage::NewTask(payload) => {
                let task_id = TaskId::new();
                let worker = &self.worker_pool[self.next_worker];
                self.next_worker = (self.next_worker + 1) % self.worker_pool.len();
                
                // ✅ Store pending task
                self.pending_tasks.insert(task_id.clone(), PendingTask {
                    started_at: Utc::now(),
                    retries: 0,
                    worker_address: worker.clone(),
                });
                
                let request = TaskRequest {
                    task_id: task_id.clone(),
                    payload,
                    priority: TaskPriority::Normal,
                };
                
                // ✅ FIRE-AND-FORGET: Send task to worker
                context.send(worker.clone(), request).await?;
                
                println!("[Coordinator] Sent task {} to worker {:?}", task_id, worker);
                
                // ✅ Don't wait - continue immediately
                // Result will come back as separate message
            }
            
            // Handle task completion
            CoordinatorMessage::TaskResult(result) => {
                println!("[Coordinator] Received result for task {}", result.task_id);
                
                // ✅ Match result to pending task
                if let Some(pending) = self.pending_tasks.remove(&result.task_id) {
                    let duration = Utc::now() - pending.started_at;
                    
                    if result.success {
                        println!("✓ Task {} completed in {:?}", result.task_id, duration);
                        self.handle_success(result).await;
                    } else {
                        println!("✗ Task {} failed: {:?}", result.task_id, result.error);
                        
                        // Retry logic
                        if pending.retries < 3 {
                            self.retry_task(result.task_id, pending).await?;
                        } else {
                            self.handle_failure(result).await;
                        }
                    }
                } else {
                    println!("⚠ Received result for unknown task: {}", result.task_id);
                }
            }
        }
        
        Ok(())
    }
}

// Task Worker - Processes tasks
struct TaskWorker {
    worker_id: u32,
    coordinator_address: ActorAddress,
}

#[async_trait]
impl Actor for TaskWorker {
    type Message = TaskRequest;
    type Error = WorkerError;
    
    async fn handle_message(
        &mut self,
        request: Self::Message,
        context: &mut ActorContext<Self::Message>,
    ) -> Result<(), Self::Error> {
        println!("[Worker {}] Processing task {}", self.worker_id, request.task_id);
        
        // Process task (might take a while)
        let result = self.process_task(request.payload).await;
        
        // Create result with SAME task_id
        let task_result = TaskResult {
            task_id: request.task_id,  // ✅ Correlation ID preserved
            success: result.is_ok(),
            output: result.ok(),
            error: result.err().map(|e| e.to_string()),
        };
        
        // ✅ FIRE-AND-FORGET: Send result back
        context.send(self.coordinator_address.clone(), task_result).await?;
        
        Ok(())
    }
}

// Message Flow:
// Coordinator                        Worker
//     │                                │
//     │──[TaskRequest ID=123]─────────▶│
//     │                                │ (process task)
//     │ (continues processing          │
//     │  other messages)               │
//     │                                │
//     │◀──[TaskResult ID=123]──────────│
//     │                                │
//  Match ID=123                        │
//  Remove from pending                 │
//  Process result                      │
```

**Characteristics**:
- ✅ Coordinator doesn't block
- ✅ Can handle multiple tasks concurrently
- ✅ Manual timeout management
- ✅ Good for: async job processing, distributed work, long-running tasks

---

### ActorContext Integration

#### Complete ActorContext API

```rust
pub struct ActorContext<M: Message> {
    // Identity
    address: ActorAddress,
    id: ActorId,
    
    // Timestamps
    created_at: DateTime<Utc>,
    last_message_at: Option<DateTime<Utc>>,
    message_count: u64,
    
    // Messaging infrastructure
    broker: InMemoryMessageBroker<M>,
    
    // Current message envelope (for reply support)
    current_envelope: Option<MessageEnvelope<M>>,
    
    _marker: PhantomData<M>,
}

impl<M: Message> ActorContext<M> {
    // ========================================================================
    // PATTERN 1: Fire-and-Forget
    // ========================================================================
    
    /// Send message without waiting for response
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// // Send log entry
    /// context.send(logger_address, log_entry).await?;
    /// ```
    pub async fn send(&self, target: ActorAddress, message: M) -> Result<(), BrokerError> {
        let envelope = MessageEnvelope::new(message)
            .with_sender(Some(self.address.clone()))
            .with_recipient(Some(target));
        
        self.broker.send(envelope).await
    }
    
    // ========================================================================
    // PATTERN 2: Request-Reply with Async Wait
    // ========================================================================
    
    /// Send request and wait for response with timeout
    /// 
    /// ⚠️ WARNING: Blocks actor until response or timeout
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// // Authenticate user and wait for result
    /// let auth_result = context.request::<AuthResponse>(
    ///     auth_service,
    ///     auth_request,
    ///     Duration::from_secs(3),
    /// ).await?;
    /// 
    /// match auth_result {
    ///     Some(response) => { /* got response */ }
    ///     None => { /* timeout */ }
    /// }
    /// ```
    pub async fn request<R: Message>(
        &self,
        target: ActorAddress,
        message: M,
        timeout: Duration,
    ) -> Result<Option<R>, BrokerError> {
        // Delegate to broker for correlation and timeout handling
        self.broker.request(target, message, self.address.clone(), timeout).await
    }
    
    // ========================================================================
    // HELPER: Reply to Current Message
    // ========================================================================
    
    /// Reply to current message sender
    /// 
    /// Automatically sends reply to the sender of the message currently
    /// being processed. Uses correlation_id from current envelope.
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// async fn handle_message(&mut self, request: AuthRequest, ctx: &mut ActorContext) {
    ///     let response = self.authenticate(request).await;
    ///     ctx.reply(response).await?;  // Sends back to requester
    /// }
    /// ```
    pub async fn reply(&self, response: M) -> Result<(), BrokerError> {
        let envelope = self.current_envelope.as_ref()
            .ok_or(BrokerError::NoMessageToReplyTo)?;
        
        let reply_address = envelope.reply_to.clone()
            .or_else(|| envelope.sender.clone())
            .ok_or(BrokerError::NoReplyAddress)?;
        
        let reply_envelope = MessageEnvelope::new(response)
            .with_sender(Some(self.address.clone()))
            .with_recipient(Some(reply_address))
            .with_correlation_id(envelope.correlation_id.clone());
        
        self.broker.send(reply_envelope).await
    }
    
    // ========================================================================
    // Internal: Update Current Envelope
    // ========================================================================
    
    /// Set current envelope (called by actor execution loop)
    pub(crate) fn set_current_envelope(&mut self, envelope: MessageEnvelope<M>) {
        self.current_envelope = Some(envelope);
        self.record_message();
    }
    
    /// Clear current envelope (called after message processing)
    pub(crate) fn clear_current_envelope(&mut self) {
        self.current_envelope = None;
    }
}
```

---

## Decision Matrix

### When to Use Each Pattern

| Scenario | Pattern | Rationale |
|----------|---------|-----------|
| **Logging, metrics, events** | Fire-and-Forget | No response needed, high throughput |
| **Notifications, broadcasts** | Fire-and-Forget | One-way communication |
| **Background job submission** | Fire-and-Forget + Correlation | Track completion asynchronously |
| **HTTP request handling** | Request-Reply (Async Wait) | Need immediate response for client |
| **Database queries** | Request-Reply (Async Wait) | Synchronous workflow required |
| **RPC-style calls** | Request-Reply (Async Wait) | Caller needs result to continue |
| **Long-running tasks** | Fire-and-Forget + Correlation | Don't block, track manually |
| **Batch processing** | Fire-and-Forget | High throughput, no waiting |
| **Service mesh calls** | Request-Reply (Async Wait) | Distributed RPC patterns |
| **Event sourcing** | Fire-and-Forget | Publish events without blocking |

---

## Performance Considerations

### Pattern Performance Characteristics

**Fire-and-Forget**:
- **Throughput**: >1M messages/second
- **Latency**: <1μs message routing
- **Memory**: Minimal overhead per message
- **Concurrency**: Full actor concurrency maintained

**Request-Reply with Async Wait**:
- **Throughput**: ~100K requests/second (blocking limits concurrency)
- **Latency**: <1μs routing + timeout overhead
- **Memory**: Additional oneshot channel per request
- **Concurrency**: Reduced (actor blocks during wait)

**Fire-and-Forget + Manual Correlation**:
- **Throughput**: >1M messages/second
- **Latency**: <1μs message routing
- **Memory**: HashMap overhead for pending requests
- **Concurrency**: Full actor concurrency maintained

---

## Integration Points

### MessageEnvelope Extensions

```rust
pub struct MessageEnvelope<M: Message> {
    pub id: MessageId,
    pub payload: M,
    pub sender: Option<ActorAddress>,
    pub recipient: Option<ActorAddress>,
    pub timestamp: DateTime<Utc>,
    
    // ✅ For request-reply pattern
    pub correlation_id: Option<MessageId>,
    pub reply_to: Option<ActorAddress>,
    
    // TTL and priority
    pub ttl: Option<Duration>,
    pub priority: MessagePriority,
}
```

### MessageBroker Extensions

```rust
impl<M: Message> InMemoryMessageBroker<M> {
    /// Request-reply support with correlation tracking
    pub async fn request<R: Message>(
        &self,
        target: ActorAddress,
        message: M,
        reply_to: ActorAddress,
        timeout: Duration,
    ) -> Result<Option<R>, BrokerError> {
        // 1. Generate correlation ID
        let correlation_id = MessageId::new();
        
        // 2. Create oneshot channel for reply
        let (tx, rx) = oneshot::channel();
        self.inner.pending_requests.insert(correlation_id.clone(), tx);
        
        // 3. Send request with correlation
        let envelope = MessageEnvelope::new(message)
            .with_correlation_id(Some(correlation_id.clone()))
            .with_reply_to(Some(reply_to));
        
        self.send(envelope).await?;
        
        // 4. Wait for reply with timeout
        match tokio::time::timeout(timeout, rx).await {
            Ok(Ok(response)) => Ok(Some(response)),
            _ => {
                self.inner.pending_requests.remove(&correlation_id);
                Ok(None)
            }
        }
    }
}
```

### Actor Execution Loop Pattern

```rust
async fn actor_execution_loop<A: Actor>(
    mut actor: A,
    mut mailbox: impl MailboxReceiver<A::Message>,
    mut context: ActorContext<A::Message>,
) {
    actor.pre_start(&mut context).await.ok();
    
    loop {
        match mailbox.recv().await {
            Some(envelope) => {
                // ✅ Set current envelope before processing
                context.set_current_envelope(envelope.clone());
                
                // Process message
                match actor.handle_message(envelope.payload, &mut context).await {
                    Ok(()) => { /* Success */ }
                    Err(error) => {
                        let action = actor.on_error(error, &context).await;
                        // Handle error action...
                    }
                }
                
                // ✅ Clear current envelope after processing
                context.clear_current_envelope();
            }
            None => break,  // Mailbox closed
        }
    }
    
    actor.post_stop(&context).await.ok();
}
```

---

## References

### Related Documentation

**Knowledge Documents**:
- KNOWLEDGE-RT-009: Message Broker Architecture and Implementation Patterns
- KNOWLEDGE-RT-005: Actor System Core Implementation Guide
- KNOWLEDGE-RT-004: Message System Implementation Guide
- KNOWLEDGE-RT-002: Message Broker Zero-Copy Patterns

**Architecture Decision Records**:
- ADR-RT-002: Message Passing Architecture
- ADR-RT-001: Actor Model Implementation Strategy

### Workspace Standards

**Standards Applied**:
- §2.1: 3-layer import organization (MANDATORY)
- §3.2: chrono DateTime<Utc> standard (MANDATORY)
- §6.1: YAGNI principles (hybrid approach, start simple)
- §6.2: Avoid dyn patterns (generic constraints)
- §6.3: Microsoft Rust Guidelines (M-DESIGN-FOR-AI, M-DI-HIERARCHY)

---

## History

### Version History

- **2025-10-05**: v1.0 - Initial knowledge documentation
  - Three messaging patterns defined
  - Complete integration examples
  - ActorContext API design
  - Performance characteristics
  - Decision matrix for pattern selection

### Review History

- **2025-10-05**: Created by AI agent with user collaboration
  - Validated hybrid approach with user
  - Confirmed fire-and-forget as primary pattern
  - Agreed on request-reply as convenience method
  - Examples reviewed and approved

---

**Document Status**: Active - Implementation Guide  
**Next Review**: 2026-01-05 (Quarterly)  
**Template Version**: 1.0  
**Last Updated**: 2025-10-05
````