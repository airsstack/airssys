# Tutorial: Building Your First Supervision Tree

**Learning Objectives:**
- Build a supervision tree from scratch
- Understand restart strategies
- Implement fault tolerance
- Handle cascading failures

**Prerequisites:**
- Complete [Message Handling](./message-handling.md) tutorial
- Understanding of actor lifecycle
- Familiarity with error handling

**Estimated time:** 40-45 minutes

---

## What You'll Build

A fault-tolerant web scraper system:
- **Supervisor**: Manages worker lifecycle
- **3 Worker Actors**: Scrape different websites
- **Automatic Recovery**: Restarts failed workers
- **Graceful Degradation**: System survives individual failures

**By the end**, you'll understand how to build resilient actor systems with supervision.

---

## Step 1: Understand Supervision Concepts

### What is a Supervision Tree?

```
                    WebScraperSupervisor
                            |
         +------------------+------------------+
         |                  |                  |
    NewsWorker        BlogWorker        ForumWorker
```

- **Supervisor**: Monitors children, restarts on failure
- **Children**: Do the work, supervised by parent
- **Restart Strategy**: How to handle child failures

### The Three Restart Strategies

**OneForOne**: Only failed child restarts (independent workers)
```
Worker1 ✓    Worker1 ✗ → Restart    Worker1 ✓
Worker2 ✓ → Worker2 ✓ (unaffected) → Worker2 ✓
Worker3 ✓    Worker3 ✓ (unaffected)   Worker3 ✓
```

**OneForAll**: All children restart (coordinated state)
```
Worker1 ✓    Worker1 ✗ → Restart All    Worker1 ✓
Worker2 ✓ → Worker2 ✓ → Worker2 ✗ → Worker2 ✓
Worker3 ✓    Worker3 ✓    Worker3 ✗    Worker3 ✓
```

**RestForOne**: Failed + later children restart (dependencies)
```
Worker1 ✓    Worker1 ✓ (unaffected)   Worker1 ✓
Worker2 ✓ → Worker2 ✗ → Restart     → Worker2 ✓
Worker3 ✓    Worker3 ✓    Worker3 ✗   Worker3 ✓ (depends on Worker2)
```

---

## Step 2: Define Worker Messages

Create messages for the scraper workers:

```rust
use airssys_rt::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkerMessage {
    ScrapeUrl { url: String },
    GetStats,
    SimulateError,  // For testing supervision
}

impl Message for WorkerMessage {
    type Result = WorkerResult;
    const MESSAGE_TYPE: &'static str = "worker";
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkerResult {
    ScrapedData { url: String, content_length: usize },
    Stats { pages_scraped: usize, errors: usize },
    Ok,
}
```

---

## Step 3: Implement a Worker Actor

Workers do the actual scraping work:

```rust
use async_trait::async_trait;
use std::fmt;

pub struct ScraperWorker {
    name: String,
    pages_scraped: usize,
    error_count: usize,
    should_fail: bool,  // For testing
}

#[derive(Debug)]
pub enum WorkerError {
    NetworkError(String),
    ParseError(String),
    SimulatedFailure,
}

impl fmt::Display for WorkerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NetworkError(url) => write!(f, "Network error for {url}"),
            Self::ParseError(msg) => write!(f, "Parse error: {msg}"),
            Self::SimulatedFailure => write!(f, "Simulated failure"),
        }
    }
}

impl std::error::Error for WorkerError {}

impl ScraperWorker {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            pages_scraped: 0,
            error_count: 0,
            should_fail: false,
        }
    }

    async fn scrape_url(&mut self, url: &str) -> Result<WorkerResult, WorkerError> {
        // Simulate scraping work
        println!("  [{}] Scraping: {}", self.name, url);
        
        // Simulate occasional network errors
        if self.should_fail {
            self.error_count += 1;
            return Err(WorkerError::NetworkError(url.to_string()));
        }

        // Success
        self.pages_scraped += 1;
        Ok(WorkerResult::ScrapedData {
            url: url.to_string(),
            content_length: 1024,  // Simulated
        })
    }
}

#[async_trait]
impl Actor for ScraperWorker {
    type Message = WorkerMessage;
    type Error = WorkerError;

    async fn pre_start<B: MessageBroker<Self::Message>>(
        &mut self,
        _context: &mut ActorContext<Self::Message, B>,
    ) -> Result<(), Self::Error> {
        println!("🚀 [{}] Worker starting...", self.name);
        Ok(())
    }

    async fn handle_message<B: MessageBroker<Self::Message>>(
        &mut self,
        message: Self::Message,
        context: &mut ActorContext<Self::Message, B>,
    ) -> Result<WorkerResult, Self::Error> {
        match message {
            WorkerMessage::ScrapeUrl { url } => {
                let result = self.scrape_url(&url).await?;
                context.record_message();
                Ok(result)
            }

            WorkerMessage::GetStats => {
                let stats = WorkerResult::Stats {
                    pages_scraped: self.pages_scraped,
                    errors: self.error_count,
                };
                context.record_message();
                Ok(stats)
            }

            WorkerMessage::SimulateError => {
                self.should_fail = true;
                println!("  [{}] ⚠️  Failure mode enabled", self.name);
                context.record_message();
                Ok(WorkerResult::Ok)
            }
        }
    }

    async fn post_restart<B: MessageBroker<Self::Message>>(
        &mut self,
        _context: &mut ActorContext<Self::Message, B>,
    ) -> Result<(), Self::Error> {
        println!("  🔄 [{}] Worker restarted (clearing failure mode)", self.name);
        self.should_fail = false;  // Reset failure flag
        Ok(())
    }

    async fn post_stop<B: MessageBroker<Self::Message>>(
        &mut self,
        _context: &mut ActorContext<Self::Message, B>,
    ) -> Result<(), Self::Error> {
        println!("  🛑 [{}] Worker stopped (scraped: {}, errors: {})",
                 self.name, self.pages_scraped, self.error_count);
        Ok(())
    }
}
```

**Key supervision features:**
- **pre_start**: Initialize worker
- **post_restart**: Clean up state after restart
- **post_stop**: Final cleanup
- **Error handling**: Return `Err` to signal supervisor

---

## Step 4: Build a Supervisor

Create a supervisor using the builder pattern:

```rust
use airssys_rt::supervisor::{SupervisorBuilder, RestartStrategy};
use tokio::time::Duration;

pub async fn build_web_scraper_supervisor() -> Supervisor<WorkerMessage> {
    println!("📋 Building supervision tree...\n");

    // Create supervisor with OneForOne strategy
    let supervisor = SupervisorBuilder::new()
        .with_name("web_scraper_supervisor")
        .with_strategy(RestartStrategy::OneForOne)
        .with_max_restarts(3)
        .with_restart_window(Duration::from_secs(60))
        .build()
        .await
        .expect("Failed to build supervisor");

    println!("  ✓ Supervisor created: OneForOne strategy");
    println!("  ✓ Max restarts: 3 per 60s window\n");

    supervisor
}
```

**Supervisor configuration:**
- **RestartStrategy::OneForOne**: Independent workers
- **max_restarts: 3**: Max 3 restarts per window
- **restart_window: 60s**: Rolling time window
- **Exceeding limits**: Supervisor escalates or stops

---

## Step 5: Add Children to Supervisor

Spawn worker children:

```rust
pub async fn spawn_workers(
    supervisor: &mut Supervisor<WorkerMessage>,
) -> Vec<ActorRef<WorkerMessage>> {
    println!("👷 Spawning worker actors...\n");

    let mut workers = Vec::new();

    // Spawn news scraper
    let news_worker = ScraperWorker::new("NewsWorker");
    let news_ref = supervisor.spawn_child(news_worker).await
        .expect("Failed to spawn news worker");
    println!("  ✓ NewsWorker spawned");
    workers.push(news_ref);

    // Spawn blog scraper
    let blog_worker = ScraperWorker::new("BlogWorker");
    let blog_ref = supervisor.spawn_child(blog_worker).await
        .expect("Failed to spawn blog worker");
    println!("  ✓ BlogWorker spawned");
    workers.push(blog_ref);

    // Spawn forum scraper
    let forum_worker = ScraperWorker::new("ForumWorker");
    let forum_ref = supervisor.spawn_child(forum_worker).await
        .expect("Failed to spawn forum worker");
    println!("  ✓ ForumWorker spawned\n");
    workers.push(forum_ref);

    workers
}
```

---

## Step 6: Test Fault Tolerance

Simulate failures and observe supervision:

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Supervision Tree Demo ===\n");

    // Build supervisor
    let mut supervisor = build_web_scraper_supervisor().await;

    // Spawn workers
    let workers = spawn_workers(&mut supervisor).await;
    let [news_ref, blog_ref, forum_ref] = workers.as_slice() else {
        panic!("Expected 3 workers");
    };

    // Test 1: Normal operation
    println!("Test 1: Normal Operation\n");
    news_ref.send(WorkerMessage::ScrapeUrl {
        url: "https://news.example.com".to_string(),
    }).await?;
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Test 2: Simulate worker failure
    println!("\nTest 2: Worker Failure & Recovery\n");
    
    // Enable failure mode
    news_ref.send(WorkerMessage::SimulateError).await?;
    tokio::time::sleep(Duration::from_millis(50)).await;

    // This should fail and trigger restart
    println!("  Triggering failure...");
    match news_ref.send(WorkerMessage::ScrapeUrl {
        url: "https://news.example.com/failing".to_string(),
    }).await {
        Ok(_) => println!("  Worker handled message"),
        Err(e) => println!("  ✗ Worker failed: {e}"),
    }

    // Wait for supervisor to restart
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Test 3: Verify recovery
    println!("\nTest 3: Verify Recovery\n");
    news_ref.send(WorkerMessage::ScrapeUrl {
        url: "https://news.example.com/recovered".to_string(),
    }).await?;
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Test 4: Other workers unaffected (OneForOne)
    println!("\nTest 4: Other Workers Unaffected\n");
    blog_ref.send(WorkerMessage::ScrapeUrl {
        url: "https://blog.example.com".to_string(),
    }).await?;
    forum_ref.send(WorkerMessage::ScrapeUrl {
        url: "https://forum.example.com".to_string(),
    }).await?;
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Test 5: Get statistics
    println!("\nTest 5: Worker Statistics\n");
    let stats = news_ref.ask(WorkerMessage::GetStats).await?;
    println!("  NewsWorker stats: {stats:?}");

    // Graceful shutdown
    println!("\nShutting down supervision tree...\n");
    supervisor.shutdown().await?;

    println!("=== Demo Complete ===");
    Ok(())
}
```

---

## Step 7: Run and Observe

```bash
cargo run
```

**Expected output:**

```
=== Supervision Tree Demo ===

📋 Building supervision tree...

  ✓ Supervisor created: OneForOne strategy
  ✓ Max restarts: 3 per 60s window

👷 Spawning worker actors...

🚀 [NewsWorker] Worker starting...
  ✓ NewsWorker spawned
🚀 [BlogWorker] Worker starting...
  ✓ BlogWorker spawned
🚀 [ForumWorker] Worker starting...
  ✓ ForumWorker spawned

Test 1: Normal Operation

  [NewsWorker] Scraping: https://news.example.com

Test 2: Worker Failure & Recovery

  [NewsWorker] ⚠️  Failure mode enabled
  Triggering failure...
  ✗ Worker failed: Network error for https://news.example.com/failing
  🔄 [NewsWorker] Worker restarted (clearing failure mode)

Test 3: Verify Recovery

  [NewsWorker] Scraping: https://news.example.com/recovered

Test 4: Other Workers Unaffected

  [BlogWorker] Scraping: https://blog.example.com
  [ForumWorker] Scraping: https://forum.example.com

Test 5: Worker Statistics

  NewsWorker stats: Stats { pages_scraped: 2, errors: 1 }

Shutting down supervision tree...

  🛑 [NewsWorker] Worker stopped (scraped: 2, errors: 1)
  🛑 [BlogWorker] Worker stopped (scraped: 1, errors: 0)
  🛑 [ForumWorker] Worker stopped (scraped: 1, errors: 0)

=== Demo Complete ===
```

---

## Understanding What Happened

### 1. **Supervisor Creation**
```rust
SupervisorBuilder::new()
    .with_strategy(RestartStrategy::OneForOne)
    .with_max_restarts(3)
    .build()
```
- Created supervisor with OneForOne strategy
- Configured restart limits (3 per 60s)
- Ready to supervise children

### 2. **Worker Spawning**
```rust
supervisor.spawn_child(worker).await?
```
- Supervisor creates and monitors child
- Child lifecycle managed by supervisor
- Returns `ActorRef` for messaging

### 3. **Failure Detection**
```rust
// Worker returns Err
return Err(WorkerError::NetworkError(url));
```
- Worker signals failure by returning `Err`
- Supervisor detects failure
- Restart process initiated

### 4. **Automatic Restart**
```
Worker fails → Supervisor detects → Calls post_restart → Worker ready
```
- Supervisor calls `post_restart` hook
- Worker clears error state
- Worker ready to process messages again

### 5. **OneForOne Isolation**
```
NewsWorker ✗ → Restart    BlogWorker ✓ (unaffected)
```
- Only failed worker restarted
- Other workers continue normally
- Isolated failure handling

---

## Comparing Restart Strategies

Let's modify the example to try different strategies:

### Strategy 1: OneForOne (Current)

**Use when**: Workers are independent

```rust
.with_strategy(RestartStrategy::OneForOne)
```

**Behavior**:
- Worker1 fails → only Worker1 restarts
- Worker2, Worker3 unaffected
- **Performance**: Minimal disruption (~1.28µs overhead)

### Strategy 2: OneForAll

**Use when**: Workers share state, must stay synchronized

```rust
.with_strategy(RestartStrategy::OneForAll)
```

**Behavior**:
- Worker1 fails → all workers restart
- Worker2, Worker3 restart even if healthy
- **Performance**: Higher overhead (30-150µs), all workers recreated

**Example use case**:
```rust
// Database connection pool
// If one connection fails, restart all to reset pool state
SupervisorBuilder::new()
    .with_strategy(RestartStrategy::OneForAll)
    .build()
```

### Strategy 3: RestForOne

**Use when**: Workers have dependencies (later depends on earlier)

```rust
.with_strategy(RestartStrategy::RestForOne)
```

**Behavior**:
- Worker2 fails → Worker2 and Worker3 restart
- Worker1 unaffected (Worker2 depends on Worker1)
- **Performance**: Moderate overhead, proportional to chain length

**Example use case**:
```rust
// Pipeline: Fetcher → Parser → Saver
// If Parser fails, restart Parser and Saver (Saver depends on Parser)
SupervisorBuilder::new()
    .with_strategy(RestartStrategy::RestForOne)
    .build()
```

---

## Advanced: Nested Supervision Trees

Build hierarchical supervision:

```rust
pub async fn build_nested_supervision() -> Supervisor<WorkerMessage> {
    // Top-level supervisor
    let mut root_supervisor = SupervisorBuilder::new()
        .with_name("root_supervisor")
        .with_strategy(RestartStrategy::OneForOne)
        .build()
        .await?;

    // Child supervisor 1: News scrapers
    let mut news_supervisor = SupervisorBuilder::new()
        .with_name("news_supervisor")
        .with_strategy(RestartStrategy::OneForAll)  // Coordinated news workers
        .build()
        .await?;

    news_supervisor.spawn_child(ScraperWorker::new("CNN")).await?;
    news_supervisor.spawn_child(ScraperWorker::new("BBC")).await?;

    // Child supervisor 2: Blog scrapers
    let mut blog_supervisor = SupervisorBuilder::new()
        .with_name("blog_supervisor")
        .with_strategy(RestartStrategy::RestForOne)  // Pipeline dependencies
        .build()
        .await?;

    blog_supervisor.spawn_child(ScraperWorker::new("Fetcher")).await?;
    blog_supervisor.spawn_child(ScraperWorker::new("Parser")).await?;
    blog_supervisor.spawn_child(ScraperWorker::new("Saver")).await?;

    // Add child supervisors to root
    root_supervisor.spawn_supervisor(news_supervisor).await?;
    root_supervisor.spawn_supervisor(blog_supervisor).await?;

    Ok(root_supervisor)
}
```

**Nested tree structure:**
```
                RootSupervisor (OneForOne)
                       |
        +--------------+--------------+
        |                             |
  NewsSupervisor              BlogSupervisor
  (OneForAll)                 (RestForOne)
        |                             |
   +----+----+              +---------+---------+
   |         |              |         |         |
  CNN       BBC          Fetcher   Parser   Saver
```

**Benefits:**
- Different strategies at different levels
- Isolated fault domains
- Flexible failure handling

---

## Best Practices

### ✅ Choose Appropriate Strategy

```rust
// Independent workers → OneForOne
let supervisor = SupervisorBuilder::new()
    .with_strategy(RestartStrategy::OneForOne)
    .build().await?;

// Synchronized state → OneForAll
let supervisor = SupervisorBuilder::new()
    .with_strategy(RestartStrategy::OneForAll)
    .build().await?;

// Pipeline dependencies → RestForOne
let supervisor = SupervisorBuilder::new()
    .with_strategy(RestartStrategy::RestForOne)
    .build().await?;
```

### ✅ Set Realistic Restart Limits

```rust
// Prevent restart storms
SupervisorBuilder::new()
    .with_max_restarts(3)           // Max 3 restarts
    .with_restart_window(Duration::from_secs(60))  // Per 60s window
    .build()
```

### ✅ Clean Up in post_restart

```rust
async fn post_restart(...) -> Result<(), Self::Error> {
    // Reset error flags
    self.should_fail = false;
    
    // Reconnect to services
    self.reconnect().await?;
    
    // Clear stale state
    self.cache.clear();
    
    Ok(())
}
```

### ✅ Monitor Supervisor Health

```rust
// Get supervisor statistics
let health = supervisor.health_check().await?;
println!("Children: {}, Restarts: {}", 
         health.active_children, 
         health.total_restarts);
```

---

## Common Mistakes

### ❌ Wrong Strategy for Use Case

```rust
// ❌ OneForAll for independent workers (unnecessary restarts)
SupervisorBuilder::new()
    .with_strategy(RestartStrategy::OneForAll)
    .build()

// ✅ OneForOne for independent workers
SupervisorBuilder::new()
    .with_strategy(RestartStrategy::OneForOne)
    .build()
```

### ❌ No Restart Limits

```rust
// ❌ Unlimited restarts (restart storm)
SupervisorBuilder::new()
    .with_max_restarts(usize::MAX)
    .build()

// ✅ Reasonable limits
SupervisorBuilder::new()
    .with_max_restarts(3)
    .with_restart_window(Duration::from_secs(60))
    .build()
```

### ❌ Panic Instead of Returning Err

```rust
// ❌ Panic kills supervisor
async fn handle_message(...) {
    if error {
        panic!("Fatal error!");  // ❌ Supervisor can't handle this
    }
}

// ✅ Return Err for supervision
async fn handle_message(...) -> Result<(), Error> {
    if error {
        return Err(Error::Fatal);  // ✅ Supervisor handles it
    }
}
```

---

## Next Steps

Congratulations! You've built a fault-tolerant supervision tree:
- ✅ Created supervisor with restart strategy
- ✅ Spawned supervised worker actors
- ✅ Handled failures gracefully
- ✅ Understood strategy tradeoffs

### Continue Learning:
- **[Supervisor Patterns Guide](../guides/supervisor-patterns.md)** - Production patterns
- **[Supervision Explanation](../explanation/supervision.md)** - Deep dive into "let it crash"
- **[Monitoring Guide](../guides/monitoring.md)** - Observability patterns

### Explore Examples:
- `examples/supervisor_basic.rs` - Simple supervision
- `examples/supervisor_strategies.rs` - Strategy comparison
- `examples/supervisor_automatic_health.rs` - Health monitoring
- [API Reference: Supervisors](../reference/api/supervisors.md) - Complete API

---

## Quick Reference

### Supervisor Builder Template

```rust
let supervisor = SupervisorBuilder::new()
    .with_name("my_supervisor")
    .with_strategy(RestartStrategy::OneForOne)  // OneForOne | OneForAll | RestForOne
    .with_max_restarts(3)
    .with_restart_window(Duration::from_secs(60))
    .build()
    .await?;
```

### Strategy Selection Guide

| Strategy | Use When | Performance | Example |
|----------|----------|-------------|---------|
| **OneForOne** | Independent workers | ~1.28µs | Web scrapers |
| **OneForAll** | Shared state | 30-150µs | Connection pool |
| **RestForOne** | Dependencies | Moderate | Data pipeline |

### Lifecycle Hooks

```rust
impl Actor for MyActor {
    async fn pre_start(...) {
        // Initialize resources
    }

    async fn post_restart(...) {
        // Clean up after restart
    }

    async fn post_stop(...) {
        // Final cleanup
    }
}
```

**Congratulations on completing the tutorials!** You now understand actors, messaging, and supervision. Ready for production patterns? Check out the [How-To Guides](../guides/actor-development.md)!
