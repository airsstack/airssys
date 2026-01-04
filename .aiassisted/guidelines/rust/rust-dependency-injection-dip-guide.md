# Dependency Injection & Dependency Inversion in Rust

## Overview

### What is Dependency Injection (DI)?

Dependency Injection is a design pattern where an object receives its dependencies from an external source rather than creating them internally. This promotes **Inversion of Control (IoC)** and leads to loosely coupled, testable code.

### What is Dependency Inversion Principle (DIP)?

The Dependency Inversion Principle states:

1. High-level modules should not depend on low-level modules. Both should depend on **abstractions**.
2. Abstractions should not depend on details. Details should depend on abstractions.

In Rust, **abstractions are traits**.

### Why DI and DIP Matter in Rust

- **Testability**: Easy to swap real implementations with mocks
- **Modularity**: Components are loosely coupled and can be developed independently
- **Flexibility**: Change implementations without modifying depending code
- **Maintainability**: Clear separation between business logic and low-level details

### When to Apply DI/DIP

**Apply When:**
- Building complex systems with multiple layers
- Writing libraries that need to support multiple implementations
- Developing systems requiring extensive testing
- Building plugin or extension systems
- Working with teams developing in parallel
- Needing to support multiple configurations/environments

**May Skip When:**
- Small, simple applications
- Prototype or proof-of-concept code
- Single-developer projects with short lifespan
- Performance-critical code with known, stable implementation

---

## Section 1: Rust-Specific DI Patterns

### Pattern 1: Direct Constructor Injection

**Best for:** Simple cases with 3-5 dependencies, all required

**Concept:** Pass all dependencies as constructor parameters. Simplest approach with no magic.

```rust
// Abstraction
pub trait Database {
    fn query(&self, sql: &str) -> Result<Vec<String>, Error>;
}

// Concrete implementation
pub struct PostgresDatabase {
    connection_string: String,
}

impl PostgresDatabase {
    pub fn new(connection_string: String) -> Self {
        Self { connection_string }
    }
}

impl Database for PostgresDatabase {
    fn query(&self, sql: &str) -> Result<Vec<String>, Error> {
        // Real implementation
        todo!()
    }
}

// Client depends on abstraction, not concrete implementation
pub struct UserRepository {
    database: Arc<dyn Database>,
}

impl UserRepository {
    // Constructor injection: all dependencies passed as parameters
    pub fn new(database: Arc<dyn Database>) -> Self {
        Self { database }
    }

    pub fn get_users(&self) -> Result<Vec<String>, Error> {
        self.database.query("SELECT * FROM users")
    }
}

// Usage
let db = Arc::new(PostgresDatabase::new("postgres://...".to_string()));
let repo = UserRepository::new(db);
```

**Benefits:**
- Explicit dependencies - easy to see what's needed
- No hidden state or magic
- Simple to understand and debug
- Works with `Arc<dyn Trait>` for runtime polymorphism

**Drawbacks:**
- Constructor can get long with many dependencies (use Builder Pattern instead)

---

### Pattern 2: Builder Pattern

**Best for:** Complex cases with optional dependencies, validation needed, many parameters

**Concept:** Use a builder to construct complex dependencies step-by-step with validation.

```rust
use std::sync::Arc;

// Dependencies
pub trait Cache {
    fn get(&self, key: &str) -> Option<String>;
    fn set(&self, key: &str, value: &str);
}

pub trait Logger {
    fn log(&self, message: &str);
}

pub trait Metrics {
    fn record(&self, name: &str, value: f64);
}

// Concrete implementations
pub struct RedisCache;
impl Cache for RedisCache {
    fn get(&self, key: &str) -> Option<String> { todo!() }
    fn set(&self, key: &str, value: &str) { todo!() }
}

pub struct StdoutLogger;
impl Logger for StdoutLogger {
    fn log(&self, message: &str) { println!("{}", message); }
}

pub struct PrometheusMetrics;
impl Metrics for PrometheusMetrics {
    fn record(&self, name: &str, value: f64) { todo!() }
}

// Builder for MyService
pub struct MyServiceBuilder {
    cache: Option<Arc<dyn Cache>>,
    logger: Option<Arc<dyn Logger>>,
    metrics: Option<Arc<dyn Metrics>>,
    max_retries: Option<u32>,
}

impl MyServiceBuilder {
    pub fn new() -> Self {
        Self {
            cache: None,
            logger: None,
            metrics: None,
            max_retries: Some(3),
        }
    }

    pub fn with_cache(mut self, cache: Arc<dyn Cache>) -> Self {
        self.cache = Some(cache);
        self
    }

    pub fn with_logger(mut self, logger: Arc<dyn Logger>) -> Self {
        self.logger = Some(logger);
        self
    }

    pub fn with_metrics(mut self, metrics: Arc<dyn Metrics>) -> Self {
        self.metrics = Some(metrics);
        self
    }

    pub fn with_max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = Some(max_retries);
        self
    }

    pub fn build(self) -> Result<MyService, String> {
        let cache = self.cache.ok_or("cache is required")?;
        let logger = self.logger.ok_or("logger is required")?;
        let metrics = self.metrics.ok_or("metrics is required")?;
        let max_retries = self.max_retries.ok_or("max_retries is required")?;

        if max_retries == 0 {
            return Err("max_retries must be > 0".to_string());
        }

        Ok(MyService {
            cache,
            logger,
            metrics,
            max_retries,
        })
    }
}

// Service constructed by builder
pub struct MyService {
    cache: Arc<dyn Cache>,
    logger: Arc<dyn Logger>,
    metrics: Arc<dyn Metrics>,
    max_retries: u32,
}

// Usage
let service = MyServiceBuilder::new()
    .with_cache(Arc::new(RedisCache))
    .with_logger(Arc::new(StdoutLogger))
    .with_metrics(Arc::new(PrometheusMetrics))
    .with_max_retries(5)
    .build()?;
```

**Benefits:**
- Optional dependencies can be omitted
- Validation before object creation
- Readable construction code
- Default values for optional parameters

**Drawbacks:**
- More boilerplate code
- More complex than simple constructor injection

---

### Pattern 3: Factory Pattern

**Best for:** Multiple configurations (prod, test, custom), creating instances with different setups

**Concept:** Encapsulate object creation logic in factory methods.

```rust
use std::sync::Arc;

// Abstractions and implementations as above...

// Factory for creating different configurations
pub struct ServiceFactory;

impl ServiceFactory {
    // Create service for production environment
    pub fn create_production(
        redis_url: String,
    ) -> Result<MyService, String> {
        let cache = Arc::new(RedisCache);
        let logger = Arc::new(StructuredLogger::new("production".to_string()));
        let metrics = Arc::new(PrometheusMetrics::new("prod-metrics"));

        MyServiceBuilder::new()
            .with_cache(cache)
            .with_logger(logger)
            .with_metrics(metrics)
            .with_max_retries(5)
            .build()
    }

    // Create service for testing environment
    pub fn create_test() -> Result<MyService, String> {
        let cache = Arc::new(InMemoryCache::new());
        let logger = Arc::new(TestLogger::new());
        let metrics = Arc::new(NoOpMetrics);

        MyServiceBuilder::new()
            .with_cache(cache)
            .with_logger(logger)
            .with_metrics(metrics)
            .with_max_retries(1)
            .build()
    }

    // Create service with custom configuration
    pub fn create_custom(config: ServiceConfig) -> Result<MyService, String> {
        let cache: Arc<dyn Cache> = if config.use_redis {
            Arc::new(RedisCache)
        } else {
            Arc::new(InMemoryCache::new())
        };

        let logger = Arc::new(StructuredLogger::new(config.environment));
        let metrics = Arc::new(PrometheusMetrics::new(&config.metrics_prefix));

        MyServiceBuilder::new()
            .with_cache(cache)
            .with_logger(logger)
            .with_metrics(metrics)
            .with_max_retries(config.max_retries)
            .build()
    }
}

// Configuration struct
pub struct ServiceConfig {
    pub use_redis: bool,
    pub environment: String,
    pub metrics_prefix: String,
    pub max_retries: u32,
}

// Mock/test implementations
pub struct InMemoryCache;
impl Cache for InMemoryCache {
    fn get(&self, key: &str) -> Option<String> { todo!() }
    fn set(&self, key: &str, value: &str) { todo!() }
}

pub struct StructuredLogger {
    env: String,
}
impl StructuredLogger {
    pub fn new(env: String) -> Self { Self { env } }
}
impl Logger for StructuredLogger {
    fn log(&self, message: &str) {
        println!("[{}] {}", self.env, message);
    }
}

pub struct TestLogger;
impl TestLogger {
    pub fn new() -> Self { Self }
}
impl Logger for TestLogger {
    fn log(&self, message: &str) { /* Silent in tests */ }
}

pub struct NoOpMetrics;
impl Metrics for NoOpMetrics {
    fn record(&self, _name: &str, _value: f64) {}
}

// Usage
let prod_service = ServiceFactory::create_production("redis://localhost".to_string())?;
let test_service = ServiceFactory::create_test()?;
let custom_service = ServiceFactory::create_custom(ServiceConfig {
    use_redis: true,
    environment: "staging".to_string(),
    metrics_prefix: "myapp".to_string(),
    max_retries: 3,
})?;
```

**Benefits:**
- Centralized creation logic
- Easy to switch configurations
- Encapsulates complex setup
- Reusable across application

**Drawbacks:**
- Another layer of indirection
- May become complex with many configurations

---

### Pattern 4: Newtype Pattern (Hiding Traits)

**Best for:** Hiding internal traits from public API, maintaining encapsulation

**Concept:** Wrap `Arc<dyn Trait>` in a concrete struct to hide trait details.

```rust
use std::sync::Arc;

// Private trait - not exposed to users
trait DatabaseInternal {
    fn query_internal(&self, sql: &str) -> Result<Vec<String>, Error>;
}

// Concrete implementation (also private)
struct PostgresDbInternal {
    connection_string: String,
}

impl DatabaseInternal for PostgresDbInternal {
    fn query_internal(&self, sql: &str) -> Result<Vec<String>, Error> {
        // Implementation details hidden
        todo!()
    }
}

// Public newtype - wraps the private trait
#[derive(Clone)]
pub struct DatabaseConnection(Arc<dyn DatabaseInternal + Send + Sync + 'static>);

impl DatabaseConnection {
    // Factory methods - users don't see the trait
    pub fn connect_postgres(connection_string: String) -> Result<Self, Error> {
        let db = PostgresDbInternal { connection_string };
        Ok(DatabaseConnection(Arc::new(db)))
    }

    // Public methods delegate to internal implementation
    pub fn query(&self, sql: &str) -> Result<Vec<String>, Error> {
        self.0.query_internal(sql)
    }
}

// Business logic depends only on the newtype, not the trait
pub struct UserService {
    db: DatabaseConnection,
}

impl UserService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub fn get_all_users(&self) -> Result<Vec<String>, Error> {
        self.db.query("SELECT * FROM users")
    }
}

// Usage - users never see DatabaseInternal trait
let conn = DatabaseConnection::connect_postgres("postgres://...".to_string())?;
let service = UserService::new(conn);
```

**Benefits:**
- Hides implementation details
- Prevents dependency on internal types
- Clean public API
- Compiler can detect dead code in private types

**Drawbacks:**
- More boilerplate code
- Delegation methods for each trait method

---

## Section 2: Trait Objects vs Generics

### When to use `Arc<dyn Trait>` (Trait Objects)

**Use when:**
- You need to store trait objects in fields
- You need runtime polymorphism (swappable implementations)
- For dependency injection (ability to pass mocks)
- You have different implementations chosen at runtime

```rust
// GOOD: Trait object for DI
pub struct Client {
    service: Arc<dyn Service>,  // Can swap implementations at runtime
}

impl Client {
    pub fn new(service: Arc<dyn Service>) -> Self {
        Self { service }
    }
}

// Runtime polymorphism
fn create_service(use_real: bool) -> Arc<dyn Service> {
    if use_real {
        Arc::new(RealService::new())
    } else {
        Arc::new(MockService::new())
    }
}
```

### When to use Generic Parameters `<T: Trait>` (Generics)

**Use when:**
- You need maximum performance (static dispatch)
- Implementation type is known at compile time
- You don't need trait objects
- You want to monomorphize for zero-cost abstraction

```rust
// GOOD: Generics for performance
pub struct Client<T: Service> {
    service: Arc<T>,  // Static dispatch
}

impl<T: Service> Client<T> {
    pub fn new(service: Arc<T>) -> Self {
        Self { service }
    }
}

// Compile-time polymorphism
let client_real: Client<RealService> = Client::new(Arc::new(RealService::new()));
let client_mock: Client<MockService> = Client::new(Arc::new(MockService::new()));
```

### Comparison Table

| Aspect | `Arc<dyn Trait>` | `<T: Trait>` (Generics) |
|--------|-----------------|--------------------------|
| **Dispatch** | Dynamic (runtime) | Static (compile-time) |
| **Performance** | Minor overhead (vtable lookup) | Zero-cost (monomorphization) |
| **Binary Size** | Smaller (single implementation) | Larger (code duplicated for each type) |
| **Flexibility** | High (can swap at runtime) | Low (type fixed at compile) |
| **DI Support** | Excellent | Good (but requires generics) |
| **Use Case** | DI, plugin systems | Performance-critical code |

### Choosing the Right Approach

```rust
// Scenario 1: DI with runtime configuration
// Use Arc<dyn Trait>
struct Application {
    db: Arc<dyn Database>,
    cache: Arc<dyn Cache>,
}

// Scenario 2: Performance-critical generic algorithm
// Use Generics
struct Processor<T: Algorithm> {
    algo: Arc<T>,
}

// Scenario 3: Mixed approach
struct Application<T: Cache> {
    db: Arc<dyn Database>,  // Runtime-swappable
    cache: Arc<T>,          // Fixed at compile time
}
```

---

## Section 3: Mocking Strategies for Testing

### Strategy 1: Simple Mock Struct

**Best for:** Simple traits, unit tests, no complex verification needs

```rust
// Trait to mock
pub trait EmailService {
    fn send_email(&self, to: &str, subject: &str, body: &str) -> Result<(), Error>;
}

// Mock implementation
#[cfg(test)]
struct MockEmailService {
    emails_sent: Arc<Mutex<Vec<(String, String, String)>>>,
}

#[cfg(test)]
impl MockEmailService {
    fn new() -> Self {
        Self {
            emails_sent: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn get_emails(&self) -> Vec<(String, String, String)> {
        self.emails_sent.lock().unwrap().clone()
    }
}

#[cfg(test)]
impl EmailService for MockEmailService {
    fn send_email(&self, to: &str, subject: &str, body: &str) -> Result<(), Error> {
        self.emails_sent.lock().unwrap().push((
            to.to_string(),
            subject.to_string(),
            body.to_string(),
        ));
        Ok(())
    }
}

// Test
#[cfg(test)]
#[test]
fn test_notification_sends_email() {
    let mock_email = Arc::new(MockEmailService::new());
    let notifier = NotificationService::new(mock_email.clone());

    notifier.notify_user("user@example.com", "Test", "Test body").unwrap();

    let emails = mock_email.get_emails();
    assert_eq!(emails.len(), 1);
    assert_eq!(emails[0].0, "user@example.com");
    assert_eq!(emails[0].1, "Test");
}
```

### Strategy 2: Mockall Integration

**Best for:** Complex traits, sophisticated verification, parameter matching

```rust
// Add to Cargo.toml: mockall = "0.12"
use mockall::{automock, mock, predicate::*};

// Option 1: Use #[automock] on existing trait
#[automock]
pub trait EmailService {
    fn send_email(&self, to: &str, subject: &str, body: &str) -> Result<(), Error>;
}

// Test with mockall
#[cfg(test)]
#[test]
fn test_notification_with_mockall() {
    let mut mock_email = MockEmailService::new();

    // Set expectations
    mock_email
        .expect_send_email()
        .with(eq("user@example.com"), eq("Test"), always())
        .times(1)
        .returning(|_, _, _| Ok(()));

    let notifier = NotificationService::new(Arc::new(mock_email));
    notifier.notify_user("user@example.com", "Test", "Test body").unwrap();
}

// Option 2: Manual mock with mockall
#[cfg(test)]
mock! {
    pub EmailService {}
}

#[cfg(test)]
impl MockEmailService {
    fn expect_send_email_success(&mut self, to: &str) {
        self.expect_send_email()
            .with(eq(to), always(), always())
            .returning(|_, _, _| Ok(()));
    }

    fn expect_send_email_failure(&mut self, to: &str) {
        self.expect_send_email()
            .with(eq(to), always(), always())
            .returning(|_, _, _| Err(Error::EmailFailed));
    }
}
```

### Strategy 3: In-Memory Test Doubles

**Best for:** Dependencies like databases, caches, external APIs

```rust
// Real trait
pub trait Database {
    fn get_user(&self, id: u64) -> Result<Option<User>, Error>;
    fn save_user(&self, user: &User) -> Result<(), Error>;
}

// In-memory test double
#[cfg(test)]
pub struct InMemoryDatabase {
    users: Arc<Mutex<HashMap<u64, User>>>,
}

#[cfg(test)]
impl InMemoryDatabase {
    pub fn new() -> Self {
        Self {
            users: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn with_initial_users(users: Vec<User>) -> Self {
        let db = Self::new();
        for user in users {
            db.users.lock().unwrap().insert(user.id, user);
        }
        db
    }

    pub fn clear(&self) {
        self.users.lock().unwrap().clear();
    }
}

#[cfg(test)]
impl Database for InMemoryDatabase {
    fn get_user(&self, id: u64) -> Result<Option<User>, Error> {
        Ok(self.users.lock().unwrap().get(&id).cloned())
    }

    fn save_user(&self, user: &User) -> Result<(), Error> {
        self.users.lock().unwrap().insert(user.id, user.clone());
        Ok(())
    }
}

// Test
#[cfg(test)]
#[test]
fn test_user_service_with_in_memory_db() {
    let test_db = Arc::new(InMemoryDatabase::with_initial_users(vec![
        User { id: 1, name: "Alice".to_string() },
    ]));
    let service = UserService::new(test_db);

    let user = service.get_user(1).unwrap();
    assert_eq!(user.unwrap().name, "Alice");
}
```

---

## Section 4: Common Pitfalls in Rust DI

### Pitfall 1: Creating Dependencies in Constructor

**Problem:** Creates tight coupling, makes testing difficult.

```rust
// BAD: Creates dependencies internally
pub struct MyStruct {
    dep: Arc<dyn Service>,
}

impl MyStruct {
    pub fn new() -> Self {
        Self {
            dep: Arc::new(ConcreteService::new()),  // ❌ Tight coupling
        }
    }
}

// GOOD: Dependencies injected
pub struct MyStruct {
    dep: Arc<dyn Service>,
}

impl MyStruct {
    pub fn new(dep: Arc<dyn Service>) -> Self {
        Self { dep }  // ✅ Loose coupling
    }
}
```

### Pitfall 2: Using Concrete Types Instead of Trait Objects

**Problem:** Can't swap implementations, defeats purpose of DI.

```rust
// BAD: Field uses concrete type
pub struct MyStruct {
    dep: Arc<ConcreteService>,  // ❌ Can't swap implementations
}

// GOOD: Field uses trait object
pub struct MyStruct {
    dep: Arc<dyn Service>,  // ✅ Can swap implementations
}
```

### Pitfall 3: Unnecessary Arc Clones

**Problem:** Overuse of Arc when simple references would work.

```rust
// OK: Arc is necessary for sharing across threads
let shared = Arc::new(Service::new());
let cloned1 = Arc::clone(&shared);
let cloned2 = Arc::clone(&shared);

// GOOD: Use simple references if no thread safety needed
fn use_service(service: &dyn Service) {
    service.do_something();
}

// BAD: Creating too many Arcs unnecessarily
let service = Arc::new(Service::new());
let cloned1 = Arc::clone(&service);  // OK if needed
let cloned2 = Arc::clone(&service);  // OK if needed
let cloned3 = Arc::clone(&cloned2);  // ❌ Unnecessary
```

### Pitfall 4: Over-Engineering Simple Cases

**Problem:** Using complex DI patterns for trivial dependencies.

```rust
// BAD: Over-engineered for simple case
pub struct SimpleStruct {
    db: Arc<dyn Database>,
    cache: Arc<dyn Cache>,
    logger: Arc<dyn Logger>,
    metrics: Arc<dyn Metrics>,
    config: Arc<dyn Config>,
}

// GOOD: Simple case uses direct injection
pub struct SimpleStruct {
    db: Arc<dyn Database>,
}

impl SimpleStruct {
    pub fn new(db: Arc<dyn Database>) -> Self {
        Self { db }
    }
}
```

### Pitfall 5: Violating Dependency Inversion

**Problem:** High-level modules depend on low-level implementations.

```rust
// BAD: Violates DIP
mod business_logic {
    use crate::database::PostgresDatabase;  // ❌ Depends on implementation

    pub struct UserService {
        db: PostgresDatabase,  // ❌ Concrete type
    }
}

// GOOD: Follows DIP
mod core {
    pub trait Database {  // ✅ Abstraction
        fn get_user(&self, id: u64) -> Result<Option<User>, Error>;
    }
}

mod database_impl {
    use crate::core::Database;

    pub struct PostgresDatabase;  // ✅ Implementation
    impl Database for PostgresDatabase { }
}

mod business_logic {
    use crate::core::Database;  // ✅ Depends on abstraction

    pub struct UserService {
        db: Arc<dyn Database>,  // ✅ Trait object
    }
}
```

---

## Section 5: Verification Checklist

Before code review, verify:

### Dependency Injection
- [ ] No `::new()` or `::default()` called on dependencies in constructors
- [ ] All dependencies injected via constructor parameters or builder
- [ ] Dependencies are traits or abstractions, not concrete implementations

### Type System
- [ ] Fields use `Arc<dyn Trait>` where appropriate (for DI/polymorphism)
- [ ] Generics `<T: Trait>` used where performance is critical
- [ ] No unnecessary `Arc` clones

### Testability
- [ ] Can create mocks of all traits used
- [ ] Unit tests use mocks for dependencies
- [ ] Integration tests use real implementations

### Architecture
- [ ] High-level modules depend on abstractions, not implementations
- [ ] Dependency direction follows DIP (high-level → abstraction ← low-level)
- [ ] No circular dependencies between modules

### Code Quality
- [ ] All tests pass (`cargo test`)
- [ ] Code compiles without warnings (`cargo build`)
- [ ] Clippy passes (`cargo clippy --all-targets --all-features -- -D warnings`)

---

## Section 6: When to Use Each Pattern

| Pattern | Use When | Complexity | Flexibility | Performance |
|----------|-----------|------------|------------|-------------|
| **Direct Constructor** | Simple cases, 3-5 deps, all required | Low | Medium | High |
| **Builder** | Complex cases, optional deps, validation needed | High | High | High |
| **Factory** | Multiple configurations (prod, test, custom) | Medium | High | High |
| **Newtype** | Hiding internal traits, public API concerns | Medium | High | Medium |
| **DI Container** | Very large projects, many components | Very High | Very High | Medium |

### Decision Flow

```
Need to create a complex object?
├─ Yes
│  ├─ Has optional dependencies?
│  │  ├─ Yes → Use Builder Pattern
│  │  └─ No → Use Direct Constructor Injection
│  │
│  ├─ Need multiple configurations?
│  │  ├─ Yes → Use Factory Pattern
│  │  └─ No → Use Direct Constructor Injection
│  │
│  ├─ Need to hide internal traits?
│  │  ├─ Yes → Use Newtype Pattern
│  │  └─ No → Use Direct Constructor Injection
│  │
│  └─ Very large project (100+ components)?
│     ├─ Yes → Consider DI Container
│     └─ No → Use Factory/Builder
│
└─ No (simple case)
   └─ Use Direct Constructor Injection
```

---

## Section 7: Complete Example

Putting it all together: a complete application demonstrating DI patterns.

```rust
use std::sync::Arc;
use std::error::Error;

// ============== Core Abstractions ==============

pub trait Logger: Send + Sync {
    fn log(&self, message: &str);
}

pub trait Cache: Send + Sync {
    fn get(&self, key: &str) -> Option<String>;
    fn set(&self, key: &str, value: &str);
}

pub trait Database: Send + Sync {
    fn query(&self, sql: &str) -> Result<Vec<String>, Box<dyn Error>>;
}

// ============== Concrete Implementations ==============

pub struct StdoutLogger;
impl Logger for StdoutLogger {
    fn log(&self, message: &str) {
        println!("[LOG] {}", message);
    }
}

pub struct FileLogger {
    path: String,
}
impl FileLogger {
    pub fn new(path: String) -> Self {
        Self { path }
    }
}
impl Logger for FileLogger {
    fn log(&self, message: &str) {
        // In real implementation, write to file
        println!("[FILE {}] {}", self.path, message);
    }
}

pub struct RedisCache;
impl Cache for RedisCache {
    fn get(&self, key: &str) -> Option<String> { todo!() }
    fn set(&self, key: &str, value: &str) { todo!() }
}

pub struct InMemoryCache {
    data: Arc<Mutex<std::collections::HashMap<String, String>>>,
}
impl InMemoryCache {
    pub fn new() -> Self {
        Self {
            data: Arc::new(Mutex::new(std::collections::HashMap::new())),
        }
    }
}
impl Cache for InMemoryCache {
    fn get(&self, key: &str) -> Option<String> {
        self.data.lock().unwrap().get(key).cloned()
    }
    fn set(&self, key: &str, value: &str) {
        self.data.lock().unwrap().insert(key.to_string(), value.to_string());
    }
}

pub struct PostgresDatabase {
    connection_string: String,
}
impl PostgresDatabase {
    pub fn new(connection_string: String) -> Self {
        Self { connection_string }
    }
}
impl Database for PostgresDatabase {
    fn query(&self, sql: &str) -> Result<Vec<String>, Box<dyn Error>> {
        // In real implementation, query database
        println!("[DB] Executing: {}", sql);
        Ok(vec!["result1".to_string(), "result2".to_string()])
    }
}

// ============== Business Logic ==============

pub struct UserService {
    db: Arc<dyn Database>,
    cache: Arc<dyn Cache>,
    logger: Arc<dyn Logger>,
}

impl UserService {
    pub fn new(
        db: Arc<dyn Database>,
        cache: Arc<dyn Cache>,
        logger: Arc<dyn Logger>,
    ) -> Self {
        Self { db, cache, logger }
    }

    pub fn get_user(&self, id: u64) -> Result<String, Box<dyn Error>> {
        let cache_key = format!("user:{}", id);

        // Try cache first
        if let Some(cached) = self.cache.get(&cache_key) {
            self.logger.log(&format!("Cache hit for user {}", id));
            return Ok(cached);
        }

        // Cache miss, query database
        self.logger.log(&format!("Cache miss, querying DB for user {}", id));
        let results = self.db.query(&format!("SELECT * FROM users WHERE id = {}", id))?;

        let user_data = results.get(0).cloned().unwrap_or_default();
        self.cache.set(&cache_key, &user_data);

        Ok(user_data)
    }
}

// ============== Factory Pattern ==============

pub struct ServiceFactory;

impl ServiceFactory {
    pub fn create_production(conn_str: String) -> UserService {
        UserService::new(
            Arc::new(PostgresDatabase::new(conn_str)),
            Arc::new(RedisCache),
            Arc::new(FileLogger::new("/var/log/app.log".to_string())),
        )
    }

    pub fn create_test() -> UserService {
        UserService::new(
            Arc::new(PostgresDatabase::new("test://db".to_string())),
            Arc::new(InMemoryCache::new()),
            Arc::new(StdoutLogger),
        )
    }
}

// ============== Main ==============

fn main() -> Result<(), Box<dyn Error>> {
    // Create service using factory
    let service = if std::env::var("ENV").unwrap_or_default() == "test" {
        ServiceFactory::create_test()
    } else {
        ServiceFactory::create_production("postgres://localhost/mydb".to_string())
    };

    // Use service
    let user = service.get_user(123)?;
    println!("User data: {}", user);

    Ok(())
}
```

---

## References

### Core Articles
1. **Dependency Injection in Rust** by Orion Kindel (Geek Culture) - https://medium.com/geekculture/dependency-injection-in-rust-3822bf689888
   - Basic trait-based DI
   - Calendar and Notifier trait examples
   - Mocking strategies

2. **Mastering Dependency Injection in Rust** by Pieter Engelbrecht (chesedo.me) - https://chesedo.me/blog/manual-dependency-injection-rust/
   - Builder pattern for complex dependencies
   - Factory pattern for multiple configurations
   - Dependency container implementation
   - Singleton, scoped, and transient lifetimes
   - Async dependencies

3. **Rust traits and dependency injection** by Julio Merino (jmmv.dev) - https://jmmv.dev/2022/04/rust-traits-and-dependency-injection.html
   - Newtype pattern for hiding traits
   - Visibility and encapsulation
   - Database logger example

4. **Rust Forum Discussion** - https://users.rust-lang.org/t/how-do-you-implement-dependency-injection-in-rust/213/4
   - Communicator trait example
   - Trait objects in return types
   - Boxed trait objects

### Further Reading
- The Rust Book: Traits - https://doc.rust-lang.org/book/ch10-02-traits.html
- Rust by Example: Trait Objects - https://doc.rust-lang.org/rust-by-example/trait.html
- SOLID Principles in Rust - https://www.youtube.com/watch?v=9v_p3l5qZbM

### Crates for DI
- **mockall** - Mocking framework: https://github.com/asomers/mockall
- **coi** - Dependency injection container: https://github.com/ZinoKader/coi
- **shaku** - Compile-time DI framework: https://github.com/Zenithar/shaku

---

## Summary

This guide provides a comprehensive approach to dependency injection and dependency inversion in Rust:

### Key Takeaways
1. ✅ **Use traits as abstractions** for DI in Rust
2. ✅ **Inject dependencies** via constructor parameters, not create them internally
3. ✅ **Use `Arc<dyn Trait>`** for runtime polymorphism and DI
4. ✅ **Use generics `<T: Trait>`** for performance-critical code
5. ✅ **Choose patterns wisely** based on complexity and needs
6. ✅ **Mock dependencies** for unit tests using simple structs or mockall
7. ✅ **Avoid common pitfalls** like over-engineering or violating DIP
8. ✅ **Verify with checklist** before completing tasks

### Pattern Decision Matrix
- **Simple (3-5 deps, all required)** → Direct Constructor Injection
- **Complex (optional deps, validation)** → Builder Pattern
- **Multiple configs** → Factory Pattern
- **Hide internal traits** → Newtype Pattern
- **Very large projects** → DI Container

**The Golden Rule:** Depend on abstractions (`dyn Trait`), not concretions. This leads to loosely coupled, testable, and maintainable software in Rust.
