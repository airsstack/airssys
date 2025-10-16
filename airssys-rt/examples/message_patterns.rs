//! Message Passing Patterns Examples
//!
//! This example demonstrates various message passing patterns and best practices
//! covered in the Message Passing guide (docs/src/guides/message-passing.md).
//!
//! Examples included:
//! 1. Small message design
//! 2. Zero-copy with Arc<T>
//! 3. Message batching
//!
//! Run with: cargo run --example message_patterns

use airssys_rt::prelude::*;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

// =============================================================================
// Example 1: Small Message Design
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CounterMsg {
    Increment,
    Decrement,
    Reset,
}

impl Message for CounterMsg {
    const MESSAGE_TYPE: &'static str = "counter";
}

pub struct Counter {
    count: i32,
    name: String,
}

impl Counter {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            count: 0,
            name: name.into(),
        }
    }
}

#[derive(Debug)]
pub struct CounterError(String);

impl fmt::Display for CounterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for CounterError {}

#[async_trait]
impl Actor for Counter {
    type Message = CounterMsg;
    type Error = CounterError;

    async fn handle_message<B: MessageBroker<Self::Message>>(
        &mut self,
        message: Self::Message,
        context: &mut ActorContext<Self::Message, B>,
    ) -> Result<(), Self::Error> {
        match message {
            CounterMsg::Increment => {
                self.count += 1;
                println!("[{}] Incremented to {}", self.name, self.count);
            }
            CounterMsg::Decrement => {
                self.count -= 1;
                println!("[{}] Decremented to {}", self.name, self.count);
            }
            CounterMsg::Reset => {
                self.count = 0;
                println!("[{}] Reset to 0", self.name);
            }
        }

        context.record_message();
        Ok(())
    }
}

// =============================================================================
// Example 2: Zero-Copy with Arc<T>
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessData {
    pub data_id: String,
    #[serde(skip)]
    pub large_data: Option<Arc<Vec<u8>>>,
    pub source: String,
}

impl Message for ProcessData {
    const MESSAGE_TYPE: &'static str = "process_data";
}

pub struct DataProcessor {
    name: String,
    processed_count: usize,
}

impl DataProcessor {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            processed_count: 0,
        }
    }
}

#[derive(Debug)]
pub struct ProcessorError(String);

impl fmt::Display for ProcessorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for ProcessorError {}

#[async_trait]
impl Actor for DataProcessor {
    type Message = ProcessData;
    type Error = ProcessorError;

    async fn handle_message<B: MessageBroker<Self::Message>>(
        &mut self,
        message: Self::Message,
        context: &mut ActorContext<Self::Message, B>,
    ) -> Result<(), Self::Error> {
        // Data is shared via Arc - no copy!
        if let Some(data) = &message.large_data {
            let data_size = data.len();
            self.processed_count += 1;

            println!(
                "[{}] Processed {} bytes from {} (ID: {}, total: {})",
                self.name, data_size, message.source, message.data_id, self.processed_count
            );

            // Simulate processing
            sleep(Duration::from_millis(10)).await;
        }

        context.record_message();
        Ok(())
    }
}

// =============================================================================
// Example 3: Message Batching
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BatchMsg {
    Item(String),
    Flush,
}

impl Message for BatchMsg {
    const MESSAGE_TYPE: &'static str = "batch";
}

pub struct BatchProcessor {
    name: String,
    batch: Vec<String>,
    batch_size: usize,
    batches_processed: usize,
}

impl BatchProcessor {
    pub fn new(name: impl Into<String>, batch_size: usize) -> Self {
        Self {
            name: name.into(),
            batch: Vec::with_capacity(batch_size),
            batch_size,
            batches_processed: 0,
        }
    }

    async fn process_batch(&mut self) {
        if !self.batch.is_empty() {
            self.batches_processed += 1;
            println!(
                "[{}] Processing batch #{} with {} items: {:?}",
                self.name,
                self.batches_processed,
                self.batch.len(),
                self.batch
            );

            // Simulate batch processing
            sleep(Duration::from_millis(50)).await;

            self.batch.clear();
        }
    }
}

#[derive(Debug)]
pub struct BatchError(String);

impl fmt::Display for BatchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for BatchError {}

#[async_trait]
impl Actor for BatchProcessor {
    type Message = BatchMsg;
    type Error = BatchError;

    async fn handle_message<B: MessageBroker<Self::Message>>(
        &mut self,
        message: Self::Message,
        context: &mut ActorContext<Self::Message, B>,
    ) -> Result<(), Self::Error> {
        match message {
            BatchMsg::Item(item) => {
                self.batch.push(item);

                // Process when batch is full
                if self.batch.len() >= self.batch_size {
                    self.process_batch().await;
                }
            }
            BatchMsg::Flush => {
                // Force process current batch
                self.process_batch().await;
            }
        }

        context.record_message();
        Ok(())
    }
}

// =============================================================================
// Main Examples
// =============================================================================

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Message Passing Patterns Examples ===\n");

    // -------------------------------------------------------------------------
    // Example 1: Small Message Design
    // -------------------------------------------------------------------------
    println!("--- Example 1: Small Message Design (< 64 bytes) ---");
    println!("Small messages = faster send latency (~50-100ns)\n");

    let mut counter = Counter::new("MainCounter");
    let address = ActorAddress::named("counter");
    let broker = InMemoryMessageBroker::<CounterMsg>::new();
    let mut context = ActorContext::new(address, broker);
    let mut lifecycle = ActorLifecycle::new();

    counter.pre_start(&mut context).await?;
    lifecycle.transition_to(ActorState::Running);

    println!("Sending small messages:");
    counter
        .handle_message(CounterMsg::Increment, &mut context)
        .await?;
    counter
        .handle_message(CounterMsg::Increment, &mut context)
        .await?;
    counter
        .handle_message(CounterMsg::Increment, &mut context)
        .await?;
    counter
        .handle_message(CounterMsg::Decrement, &mut context)
        .await?;
    counter
        .handle_message(CounterMsg::Reset, &mut context)
        .await?;

    println!(
        "Message stats: {} messages processed",
        context.message_count()
    );
    println!();

    // -------------------------------------------------------------------------
    // Example 2: Zero-Copy with Arc<T>
    // -------------------------------------------------------------------------
    println!("--- Example 2: Zero-Copy with Arc<T> ---");
    println!("Arc<T> clone cost: ~10ns vs 1ms for 1MB copy (100x faster!)\n");

    let mut processor = DataProcessor::new("DataProcessor");
    let address = ActorAddress::named("processor");
    let broker = InMemoryMessageBroker::<ProcessData>::new();
    let mut context = ActorContext::new(address, broker);
    let mut lifecycle = ActorLifecycle::new();

    processor.pre_start(&mut context).await?;
    lifecycle.transition_to(ActorState::Running);

    // Create large data (1MB)
    let large_data = vec![0u8; 1_000_000];
    println!("Created 1MB data buffer");

    // Wrap in Arc for zero-copy sharing
    let shared_data = Arc::new(large_data);

    // Send to processor - only Arc is cloned (cheap!)
    let msg1 = ProcessData {
        data_id: "data-001".to_string(),
        large_data: Some(Arc::clone(&shared_data)),
        source: "sensor-001".to_string(),
    };

    let msg2 = ProcessData {
        data_id: "data-002".to_string(),
        large_data: Some(Arc::clone(&shared_data)),
        source: "sensor-002".to_string(),
    };

    println!("Sending 2 messages with shared 1MB data:");
    processor.handle_message(msg1, &mut context).await?;
    processor.handle_message(msg2, &mut context).await?;

    println!(
        "Message stats: {} messages processed",
        context.message_count()
    );
    println!("Arc reference count: {}", Arc::strong_count(&shared_data));
    println!();

    // -------------------------------------------------------------------------
    // Example 3: Message Batching
    // -------------------------------------------------------------------------
    println!("--- Example 3: Message Batching ---");
    println!("Batching: 100ns/msg vs 1,000ns/msg individual (10x improvement!)\n");

    let mut batch_processor = BatchProcessor::new("BatchProc", 5);
    let address = ActorAddress::named("batch_processor");
    let broker = InMemoryMessageBroker::<BatchMsg>::new();
    let mut context = ActorContext::new(address, broker);
    let mut lifecycle = ActorLifecycle::new();

    batch_processor.pre_start(&mut context).await?;
    lifecycle.transition_to(ActorState::Running);

    // Send individual items
    println!("Sending 12 items (batch size = 5):");
    for i in 1..=12 {
        batch_processor
            .handle_message(BatchMsg::Item(format!("item-{i}")), &mut context)
            .await?;
        sleep(Duration::from_millis(5)).await;
    }

    sleep(Duration::from_millis(100)).await;

    // Flush remaining items
    println!("\nFlushing remaining items:");
    batch_processor
        .handle_message(BatchMsg::Flush, &mut context)
        .await?;

    println!(
        "Message stats: {} messages processed",
        context.message_count()
    );
    println!();

    // -------------------------------------------------------------------------
    // Summary
    // -------------------------------------------------------------------------
    println!("--- Summary ---");
    println!("✅ Small messages: <64 bytes for ~50-100ns latency");
    println!("✅ Arc<T> sharing: 100x faster than copying large data");
    println!("✅ Batching: 10x throughput improvement for bulk processing");
    println!();
    println!("Performance Targets (from BENCHMARKING.md §6.2):");
    println!("  - Small message (<64B): 50-100ns send latency");
    println!("  - Arc clone vs copy: 10ns vs 1ms for 1MB (100x)");
    println!("  - Single actor: 1M+ messages/sec");
    println!("  - Batched: 5M+ messages/sec");
    println!();
    println!("See docs/src/guides/message-passing.md for complete patterns");

    Ok(())
}
