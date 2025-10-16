//! Actor Development Patterns Example
//!
//! This example demonstrates the advanced patterns from the Actor Development Tutorial:
//! - State management (mutable state)
//! - Message patterns (commands, queries, events)
//! - Error handling (circuit breaker)
//! - Actor lifecycle
//!
//! Run with: cargo run --example actor_patterns

use airssys_rt::prelude::*;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::time::{Duration, Instant};

// =============================================================================
// Message Patterns
// =============================================================================

// Pattern 1: Command Messages (imperative, fire-and-forget)
#[derive(Debug, Clone, Serialize, Deserialize)]
enum CacheCommand {
    Set { key: String, value: String },
    Delete { key: String },
    Clear,
}

// Pattern 2: Query Messages (request/reply)
// Note: In a full implementation, queries would use oneshot channels.
// For this example, we'll use a simpler synchronous query pattern.
#[derive(Debug, Clone, Serialize, Deserialize)]
enum CacheQuery {
    Get { key: String },
    Size,
    Stats,
}

// Pattern 3: Event Messages (past tense, pub/sub)
#[derive(Debug, Clone, Serialize, Deserialize)]
enum CacheEvent {
    KeySet { key: String, size: usize },
    KeyDeleted { key: String },
    Cleared { count: usize },
}

// Combined message type
#[derive(Clone, Debug, Serialize, Deserialize)]
enum CacheMessage {
    Command(CacheCommand),
    Query(CacheQuery),
    Event(CacheEvent),
}

impl Message for CacheMessage {
    const MESSAGE_TYPE: &'static str = "cache";
}

// =============================================================================
// State Management Patterns
// =============================================================================

// Pattern 2: Mutable State (performance-oriented)
struct CacheActor {
    // Direct mutable state
    cache: HashMap<String, String>,
    hits: u64,
    misses: u64,
    
    // Circuit breaker state
    breaker: CircuitBreaker,
}

// Circuit Breaker Implementation
struct CircuitBreaker {
    state: BreakerState,
    failure_count: u32,
    threshold: u32,
    timeout: Duration,
    last_failure: Option<Instant>,
}

enum BreakerState {
    Closed,   // Normal operation
    Open,     // Failing, reject requests
    HalfOpen, // Testing recovery
}

impl CircuitBreaker {
    fn new(threshold: u32, timeout: Duration) -> Self {
        Self {
            state: BreakerState::Closed,
            failure_count: 0,
            threshold,
            timeout,
            last_failure: None,
        }
    }

    fn should_attempt(&mut self) -> bool {
        match self.state {
            BreakerState::Closed => true,
            BreakerState::Open => {
                if let Some(last) = self.last_failure {
                    if last.elapsed() > self.timeout {
                        println!("Circuit breaker: Attempting recovery (Half-Open)");
                        self.state = BreakerState::HalfOpen;
                        true
                    } else {
                        println!("Circuit breaker: OPEN - rejecting request");
                        false
                    }
                } else {
                    false
                }
            }
            BreakerState::HalfOpen => true,
        }
    }

    fn on_success(&mut self) {
        if matches!(self.state, BreakerState::HalfOpen) {
            println!("Circuit breaker: Recovery successful - CLOSED");
        }
        self.failure_count = 0;
        self.state = BreakerState::Closed;
    }

    fn on_failure(&mut self) {
        self.failure_count += 1;
        self.last_failure = Some(Instant::now());
        
        if self.failure_count >= self.threshold {
            println!("Circuit breaker: Failure threshold reached - OPEN");
            self.state = BreakerState::Open;
        } else {
            println!("Circuit breaker: Failure {} of {}", self.failure_count, self.threshold);
        }
    }
}

// =============================================================================
// Error Handling
// =============================================================================

#[derive(Debug)]
enum CacheError {
    StorageFull,
    BreakerOpen,
}

impl fmt::Display for CacheError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CacheError::StorageFull => write!(f, "Cache storage full"),
            CacheError::BreakerOpen => write!(f, "Circuit breaker open"),
        }
    }
}

impl std::error::Error for CacheError {}

// =============================================================================
// Actor Implementation
// =============================================================================

#[async_trait]
impl Actor for CacheActor {
    type Message = CacheMessage;
    type Error = CacheError;

    async fn pre_start<B: MessageBroker<Self::Message>>(
        &mut self,
        context: &mut ActorContext<Self::Message, B>,
    ) -> Result<(), Self::Error> {
        println!("[{}] Cache actor starting...", 
                 context.address().name().unwrap_or("cache"));
        Ok(())
    }

    async fn handle_message<B: MessageBroker<Self::Message>>(
        &mut self,
        message: Self::Message,
        context: &mut ActorContext<Self::Message, B>,
    ) -> Result<(), Self::Error> {
        // Check circuit breaker before processing
        if !self.breaker.should_attempt() {
            return Err(CacheError::BreakerOpen);
        }

        match message {
            CacheMessage::Command(cmd) => self.handle_command(cmd, context).await,
            CacheMessage::Query(query) => self.handle_query(query, context).await,
            CacheMessage::Event(event) => self.handle_event(event, context).await,
        }
    }

    async fn on_error<B: MessageBroker<Self::Message>>(
        &mut self,
        error: Self::Error,
        _context: &mut ActorContext<Self::Message, B>,
    ) -> ErrorAction {
        match error {
            CacheError::StorageFull => {
                // Temporary issue - clear cache and restart
                self.breaker.on_failure();
                ErrorAction::Restart
            }
            CacheError::BreakerOpen => {
                // Circuit breaker protection - stop processing
                ErrorAction::Resume
            }
        }
    }

    async fn post_stop<B: MessageBroker<Self::Message>>(
        &mut self,
        context: &mut ActorContext<Self::Message, B>,
    ) -> Result<(), Self::Error> {
        println!("[{}] Cache actor stopped. Final stats:", 
                 context.address().name().unwrap_or("cache"));
        println!("  - Entries: {}", self.cache.len());
        println!("  - Hits: {}", self.hits);
        println!("  - Misses: {}", self.misses);
        println!("  - Messages: {}", context.message_count());
        Ok(())
    }
}

impl CacheActor {
    fn new() -> Self {
        Self {
            cache: HashMap::new(),
            hits: 0,
            misses: 0,
            breaker: CircuitBreaker::new(3, Duration::from_secs(5)),
        }
    }

    async fn handle_command<B: MessageBroker<CacheMessage>>(
        &mut self,
        cmd: CacheCommand,
        context: &mut ActorContext<CacheMessage, B>,
    ) -> Result<(), CacheError> {
        match cmd {
            CacheCommand::Set { key, value } => {
                // Check capacity (simulate storage limit)
                if self.cache.len() >= 100 {
                    self.breaker.on_failure();
                    return Err(CacheError::StorageFull);
                }
                
                self.cache.insert(key.clone(), value.clone());
                self.breaker.on_success();
                println!("✓ Set key '{}' = '{}'", key, value);
                
                // Emit event
                context.send_event(CacheMessage::Event(CacheEvent::KeySet {
                    key,
                    size: value.len(),
                }));
            }
            CacheCommand::Delete { key } => {
                if self.cache.remove(&key).is_some() {
                    println!("✓ Deleted key '{}'", key);
                    context.send_event(CacheMessage::Event(CacheEvent::KeyDeleted { key }));
                }
                self.breaker.on_success();
            }
            CacheCommand::Clear => {
                let count = self.cache.len();
                self.cache.clear();
                self.hits = 0;
                self.misses = 0;
                println!("✓ Cleared {} entries", count);
                context.send_event(CacheMessage::Event(CacheEvent::Cleared { count }));
                self.breaker.on_success();
            }
        }
        
        context.record_message();
        Ok(())
    }

    async fn handle_query<B: MessageBroker<CacheMessage>>(
        &mut self,
        query: CacheQuery,
        context: &mut ActorContext<CacheMessage, B>,
    ) -> Result<(), CacheError> {
        match query {
            CacheQuery::Get { key } => {
                let value = self.cache.get(&key).cloned();
                if value.is_some() {
                    self.hits += 1;
                    println!("✓ Cache hit for key '{}': {:?}", key, value);
                } else {
                    self.misses += 1;
                    println!("✗ Cache miss for key '{}'", key);
                }
            }
            CacheQuery::Size => {
                let size = self.cache.len();
                println!("✓ Cache size: {}", size);
            }
            CacheQuery::Stats => {
                println!("✓ Cache stats: {} entries, {} hits, {} misses", 
                         self.cache.len(), self.hits, self.misses);
            }
        }
        
        context.record_message();
        Ok(())
    }

    async fn handle_event<B: MessageBroker<CacheMessage>>(
        &mut self,
        event: CacheEvent,
        context: &mut ActorContext<CacheMessage, B>,
    ) -> Result<(), CacheError> {
        // Events are typically broadcast to observers
        // Here we just log them
        println!("📢 Event: {:?}", event);
        context.record_message();
        Ok(())
    }
}

// Helper trait for event emission (not in actual API yet)
trait ContextExt<M: Message, B: MessageBroker<M>> {
    fn send_event(&self, _event: M) {
        // In real implementation, this would publish to broker
    }
}

impl<M: Message, B: MessageBroker<M>> ContextExt<M, B> for ActorContext<M, B> {}

// =============================================================================
// Main Example
// =============================================================================

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Actor Patterns Example ===\n");

    // Create cache actor
    let mut actor = CacheActor::new();
    let address = ActorAddress::named("cache");
    let broker = InMemoryMessageBroker::<CacheMessage>::new();
    let mut context = ActorContext::new(address, broker);
    let mut lifecycle = ActorLifecycle::new();

    // Start actor
    println!("1. Starting cache actor...");
    actor.pre_start(&mut context).await?;
    lifecycle.transition_to(ActorState::Running);
    println!();

    // Demonstrate command pattern
    println!("2. Testing command pattern (Set/Delete/Clear):");
    actor.handle_message(
        CacheMessage::Command(CacheCommand::Set {
            key: "name".to_string(),
            value: "AirsSys".to_string(),
        }),
        &mut context,
    ).await?;

    actor.handle_message(
        CacheMessage::Command(CacheCommand::Set {
            key: "version".to_string(),
            value: "0.1.0".to_string(),
        }),
        &mut context,
    ).await?;
    println!();

    // Demonstrate query pattern (simplified - no reply channels)
    println!("3. Testing query pattern:");
    
    actor.handle_message(
        CacheMessage::Query(CacheQuery::Get {
            key: "name".to_string(),
        }),
        &mut context,
    ).await?;

    actor.handle_message(
        CacheMessage::Query(CacheQuery::Size),
        &mut context,
    ).await?;
    println!();

    // Demonstrate error handling and circuit breaker
    println!("4. Testing circuit breaker (simulate failures):");
    for i in 1..=5 {
        // Simulate failure by setting too many items
        for j in 0..30 {
            let _ = actor.handle_message(
                CacheMessage::Command(CacheCommand::Set {
                    key: format!("key{}", i * 100 + j),
                    value: format!("value{}", j),
                }),
                &mut context,
            ).await;
        }
        
        if i < 5 {
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
    }
    println!();

    // Demonstrate cache clearing
    println!("5. Clearing cache:");
    actor.handle_message(
        CacheMessage::Command(CacheCommand::Clear),
        &mut context,
    ).await?;
    println!();

    // Final stats
    println!("6. Final statistics:");
    actor.handle_message(
        CacheMessage::Query(CacheQuery::Stats),
        &mut context,
    ).await?;
    println!();

    // Shutdown
    println!("7. Shutting down:");
    lifecycle.transition_to(ActorState::Stopping);
    actor.post_stop(&mut context).await?;
    lifecycle.transition_to(ActorState::Stopped);

    println!("\n=== Example Complete ===");
    Ok(())
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cache_set_get() {
        let mut actor = CacheActor::new();
        let address = ActorAddress::named("test-cache");
        let broker = InMemoryMessageBroker::new();
        let mut context = ActorContext::new(address, broker);

        // Set a value
        actor.handle_message(
            CacheMessage::Command(CacheCommand::Set {
                key: "test".to_string(),
                value: "data".to_string(),
            }),
            &mut context,
        ).await.unwrap();

        // Query the value
        actor.handle_message(
            CacheMessage::Query(CacheQuery::Get {
                key: "test".to_string(),
            }),
            &mut context,
        ).await.unwrap();

        // Value should be in cache (verified via printed output)
        assert_eq!(actor.cache.get("test"), Some(&"data".to_string()));
    }

    #[tokio::test]
    async fn test_circuit_breaker() {
        let mut actor = CacheActor::new();
        let address = ActorAddress::named("test-breaker");
        let broker = InMemoryMessageBroker::new();
        let mut context = ActorContext::new(address, broker);

        // Fill cache to trigger storage full
        for i in 0..100 {
            let _ = actor.handle_message(
                CacheMessage::Command(CacheCommand::Set {
                    key: format!("key{}", i),
                    value: "value".to_string(),
                }),
                &mut context,
            ).await;
        }

        // Next insert should fail
        let result = actor.handle_message(
            CacheMessage::Command(CacheCommand::Set {
                key: "overflow".to_string(),
                value: "data".to_string(),
            }),
            &mut context,
        ).await;

        assert!(result.is_err());
    }
}
