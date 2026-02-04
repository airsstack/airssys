# Rust Dependency Management Implementation Guide

> **Version:** 1.0  
> **Date:** 2026-01-25  
> **Complements:** `.aiassisted/guidelines/architecture/dependency-management.md`

---

## Overview

### Purpose

This guide provides **Rust-specific implementation patterns** for Dependency Injection (DI) and Dependency Inversion Principle (DIP). It focuses on practical, production-ready code examples using Rust's type system, ownership model, and trait system.

### Assumed Knowledge

- Basic Rust syntax and ownership rules
- Understanding of traits and trait objects
- Familiarity with `Arc`, `Rc`, and smart pointers
- Basic async/await concepts

### Related Documents

- **Architecture Guide**: `.aiassisted/guidelines/architecture/dependency-management.md` - Language-agnostic DI/DIP concepts
- **Rust Guidelines**: `.aiassisted/guidelines/rust/microsoft-rust-guidelines.md` - Base Rust coding standards
- **Builder Pattern**: `.aiassisted/guidelines/rust/rust-builder-pattern-guide.md` - Builder pattern for complex dependencies
- **Factory Pattern**: `.aiassisted/guidelines/rust/rust-factory-pattern-guide.md` - Factory pattern for creating instances

---

## Table of Contents

1. [When to Use Dependency Management](#1-when-to-use-dependency-management)
2. [Core Concepts in Rust](#2-core-concepts-in-rust)
3. [Pattern 1: Constructor Injection](#3-pattern-1-constructor-injection)
4. [Pattern 2: Builder-Based Dependency Injection](#4-pattern-2-builder-based-dependency-injection)
5. [Pattern 3: Factory-Based Dependency Injection](#5-pattern-3-factory-based-dependency-injection)
6. [Pattern 4: Trait Objects vs Generics](#6-pattern-4-trait-objects-vs-generics)
7. [Pattern 5: Async Dependency Injection](#7-pattern-5-async-dependency-injection)
8. [Pattern 6: Scoped Dependencies and Lifetimes](#8-pattern-6-scoped-dependencies-and-lifetimes)
9. [Testing with Dependency Injection](#9-testing-with-dependency-injection)
10. [Common Patterns](#10-common-patterns)
11. [Anti-Patterns](#11-anti-patterns)
12. [Conclusion](#12-conclusion)

---

## 1. When to Use Dependency Management

### ✅ Use Dependency Injection When:

- **Testing Requirements**: Need to mock external dependencies (databases, APIs, file systems)
- **Multiple Implementations**: Same interface with different implementations (production vs test, different databases)
- **Plugin Architecture**: Components should be swappable at runtime or compile time
- **Loose Coupling**: High-level modules shouldn't depend on low-level implementation details
- **Configuration-Based Behavior**: Different behaviors based on environment (dev, staging, production)
- **Complex Dependencies**: Components have multiple dependencies that need coordination

### ❌ Skip Dependency Injection When:

- **Simple Utilities**: Pure functions or stateless utilities with no external dependencies
- **Performance-Critical Paths**: Zero-cost abstractions needed, use generics instead of trait objects
- **Single Implementation**: No realistic alternative implementations exist or will exist
- **Owned Data Structures**: Simple structs that own their data with no I/O or side effects
- **Prototypes**: Quick proof-of-concept code that won't be maintained

### Examples

#### ✅ Good Use Case: HTTP Client Service

```rust
// Multiple implementations: real HTTP, mock for tests, fake for local dev
pub trait HttpClient: Send + Sync {
    fn get(&self, url: &str) -> Result<String, Error>;
}

pub struct UserService {
    http_client: Arc<dyn HttpClient>,
}

impl UserService {
    pub fn new(http_client: Arc<dyn HttpClient>) -> Self {
        Self { http_client }
    }
}
```

#### ❌ Bad Use Case: Simple Calculator

```rust
// NO NEED for DI - pure functions, no external dependencies
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

pub fn multiply(a: i32, b: i32) -> i32 {
    a * b
}
```

---

## 2. Core Concepts in Rust

### 2.1 Dependency Inversion Principle (DIP)

**Definition:**
1. High-level modules should not depend on low-level modules. Both should depend on abstractions (traits).
2. Abstractions should not depend on details. Details should depend on abstractions.

**In Rust:**
- **Abstractions = Traits**
- **Details = Concrete Implementations**

#### Without DIP (Tight Coupling)

```rust
use std::fs::File;
use std::io::Write;

// Low-level implementation
pub struct FileLogger {
    file: File,
}

impl FileLogger {
    pub fn new(path: &str) -> Result<Self, std::io::Error> {
        Ok(Self {
            file: File::create(path)?,
        })
    }

    pub fn log(&mut self, message: &str) -> Result<(), std::io::Error> {
        writeln!(self.file, "{}", message)
    }
}

// High-level module depends on concrete implementation
pub struct OrderService {
    logger: FileLogger,  // ❌ Tight coupling to FileLogger
}

impl OrderService {
    pub fn new() -> Result<Self, std::io::Error> {
        Ok(Self {
            logger: FileLogger::new("/var/log/orders.log")?,  // ❌ Creates dependency internally
        })
    }

    pub fn create_order(&mut self, order_id: u64) -> Result<(), std::io::Error> {
        // Business logic
        self.logger.log(&format!("Order {} created", order_id))?;
        Ok(())
    }
}
```

**Problems:**
- Cannot test `OrderService` without creating actual log files
- Cannot use different loggers (stdout, database, remote)
- Violates DIP: high-level depends on low-level implementation

#### With DIP (Loose Coupling)

```rust
use std::sync::Arc;

// Abstraction (trait)
pub trait Logger: Send + Sync {
    fn log(&self, message: &str) -> Result<(), LogError>;
}

#[derive(Debug)]
pub enum LogError {
    IoError(String),
}

// Low-level implementation 1: File logger
pub struct FileLogger {
    path: String,
}

impl FileLogger {
    pub fn new(path: String) -> Self {
        Self { path }
    }
}

impl Logger for FileLogger {
    fn log(&self, message: &str) -> Result<(), LogError> {
        // File logging implementation
        println!("[FILE:{}] {}", self.path, message);
        Ok(())
    }
}

// Low-level implementation 2: Stdout logger
pub struct StdoutLogger;

impl Logger for StdoutLogger {
    fn log(&self, message: &str) -> Result<(), LogError> {
        println!("[STDOUT] {}", message);
        Ok(())
    }
}

// High-level module depends on abstraction
pub struct OrderService {
    logger: Arc<dyn Logger>,  // ✅ Depends on trait, not implementation
}

impl OrderService {
    pub fn new(logger: Arc<dyn Logger>) -> Self {  // ✅ Dependency injection
        Self { logger }
    }

    pub fn create_order(&self, order_id: u64) -> Result<(), LogError> {
        // Business logic
        self.logger.log(&format!("Order {} created", order_id))?;
        Ok(())
    }
}

// Usage: Production
fn main() {
    let logger = Arc::new(FileLogger::new("/var/log/orders.log".to_string()));
    let service = OrderService::new(logger);
    service.create_order(123).unwrap();
}

// Usage: Testing
#[cfg(test)]
mod tests {
    use super::*;

    struct MockLogger {
        messages: std::sync::Mutex<Vec<String>>,
    }

    impl MockLogger {
        fn new() -> Self {
            Self {
                messages: std::sync::Mutex::new(Vec::new()),
            }
        }

        fn get_messages(&self) -> Vec<String> {
            self.messages.lock().unwrap().clone()
        }
    }

    impl Logger for MockLogger {
        fn log(&self, message: &str) -> Result<(), LogError> {
            self.messages.lock().unwrap().push(message.to_string());
            Ok(())
        }
    }

    #[test]
    fn test_create_order_logs_message() {
        let mock_logger = Arc::new(MockLogger::new());
        let service = OrderService::new(mock_logger.clone());

        service.create_order(123).unwrap();

        let messages = mock_logger.get_messages();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0], "Order 123 created");
    }
}
```

**Benefits:**
- ✅ `OrderService` works with any `Logger` implementation
- ✅ Easy to test with mock logger
- ✅ Can swap implementations at runtime
- ✅ Follows DIP: both depend on `Logger` trait

### 2.2 Smart Pointers for Dependency Injection

Rust's ownership system requires careful handling of dependencies. Choose the right smart pointer based on your needs:

#### `Arc<T>` - Atomic Reference Counting (Thread-Safe)

**Use when:**
- Dependency shared across multiple threads
- Service needs to be cloned and passed to async tasks
- Multi-threaded application with shared state

```rust
use std::sync::Arc;

pub struct ApiService {
    http_client: Arc<dyn HttpClient>,  // ✅ Thread-safe sharing
    database: Arc<dyn Database>,
}

impl ApiService {
    pub fn new(
        http_client: Arc<dyn HttpClient>,
        database: Arc<dyn Database>,
    ) -> Self {
        Self { http_client, database }
    }
}

// Can clone and move to different threads
let service = Arc::new(ApiService::new(client, db));
let service_clone = Arc::clone(&service);
tokio::spawn(async move {
    service_clone.handle_request().await;
});
```

#### `Rc<T>` - Reference Counting (Single-Threaded)

**Use when:**
- Single-threaded application only
- Shared ownership without thread safety overhead
- GUI applications or single-threaded event loops

```rust
use std::rc::Rc;

pub struct AppState {
    config: Rc<dyn Config>,  // ✅ Single-threaded sharing
    cache: Rc<dyn Cache>,
}

impl AppState {
    pub fn new(config: Rc<dyn Config>, cache: Rc<dyn Cache>) -> Self {
        Self { config, cache }
    }
}
```

⚠️ **Warning:** `Rc<T>` is not `Send` or `Sync` - cannot be used across threads!

#### `Box<T>` - Heap Allocation (Single Ownership)

**Use when:**
- Single owner, no sharing needed
- Want to reduce stack size
- Need trait object with single ownership

```rust
pub struct Worker {
    task: Box<dyn Task>,  // ✅ Single ownership
}

impl Worker {
    pub fn new(task: Box<dyn Task>) -> Self {
        Self { task }
    }

    pub fn execute(self) {
        self.task.run();  // Consumes self
    }
}
```

#### Comparison Table

| Smart Pointer | Thread-Safe | Clone Cost | Use Case |
|--------------|-------------|------------|----------|
| `Arc<T>` | ✅ Yes (`Send + Sync`) | Atomic increment | Multi-threaded apps, async code |
| `Rc<T>` | ❌ No | Non-atomic increment | Single-threaded apps |
| `Box<T>` | ⚠️ Depends on `T` | Cannot clone | Single ownership, no sharing |

---

## 3. Pattern 1: Constructor Injection

### Overview

**Best For:** Simple dependencies, all required, 2-5 dependencies

Constructor injection is the most straightforward DI pattern in Rust. Dependencies are passed as parameters to the `new` or constructor method.

### Basic Constructor Injection

```rust
use std::sync::Arc;

// Trait definitions
pub trait Database: Send + Sync {
    fn query(&self, sql: &str) -> Result<Vec<String>, DbError>;
    fn execute(&self, sql: &str) -> Result<u64, DbError>;
}

pub trait Cache: Send + Sync {
    fn get(&self, key: &str) -> Option<String>;
    fn set(&self, key: &str, value: String);
}

pub trait Logger: Send + Sync {
    fn info(&self, message: &str);
    fn error(&self, message: &str);
}

#[derive(Debug)]
pub enum DbError {
    ConnectionError(String),
    QueryError(String),
}

#[derive(Debug)]
pub enum ServiceError {
    DatabaseError(DbError),
    NotFound,
}

// Service with constructor injection
pub struct UserService {
    database: Arc<dyn Database>,
    cache: Arc<dyn Cache>,
    logger: Arc<dyn Logger>,
}

impl UserService {
    // Constructor injection: all dependencies passed as parameters
    pub fn new(
        database: Arc<dyn Database>,
        cache: Arc<dyn Cache>,
        logger: Arc<dyn Logger>,
    ) -> Self {
        Self {
            database,
            cache,
            logger,
        }
    }

    pub fn get_user(&self, user_id: u64) -> Result<User, ServiceError> {
        let cache_key = format!("user:{}", user_id);

        // Try cache first
        if let Some(cached) = self.cache.get(&cache_key) {
            self.logger.info(&format!("Cache hit for user {}", user_id));
            return Ok(serde_json::from_str(&cached).unwrap());
        }

        // Cache miss - query database
        self.logger.info(&format!("Cache miss for user {}", user_id));
        let results = self.database
            .query(&format!("SELECT * FROM users WHERE id = {}", user_id))
            .map_err(ServiceError::DatabaseError)?;

        if results.is_empty() {
            return Err(ServiceError::NotFound);
        }

        let user: User = serde_json::from_str(&results[0]).unwrap();
        self.cache.set(&cache_key, serde_json::to_string(&user).unwrap());

        Ok(user)
    }

    pub fn create_user(&self, name: &str, email: &str) -> Result<User, ServiceError> {
        self.logger.info(&format!("Creating user: {}", email));

        self.database
            .execute(&format!(
                "INSERT INTO users (name, email) VALUES ('{}', '{}')",
                name, email
            ))
            .map_err(ServiceError::DatabaseError)?;

        Ok(User {
            id: 1,
            name: name.to_string(),
            email: email.to_string(),
        })
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct User {
    pub id: u64,
    pub name: String,
    pub email: String,
}
```

### Concrete Implementations

```rust
use std::collections::HashMap;
use std::sync::Mutex;

// PostgreSQL database implementation
pub struct PostgresDatabase {
    connection_string: String,
}

impl PostgresDatabase {
    pub fn new(connection_string: String) -> Self {
        Self { connection_string }
    }
}

impl Database for PostgresDatabase {
    fn query(&self, sql: &str) -> Result<Vec<String>, DbError> {
        // Actual PostgreSQL query implementation
        println!("[PostgreSQL] Query: {}", sql);
        Ok(vec![r#"{"id":1,"name":"Alice","email":"alice@example.com"}"#.to_string()])
    }

    fn execute(&self, sql: &str) -> Result<u64, DbError> {
        println!("[PostgreSQL] Execute: {}", sql);
        Ok(1)
    }
}

// Redis cache implementation
pub struct RedisCache {
    store: Mutex<HashMap<String, String>>,
}

impl RedisCache {
    pub fn new() -> Self {
        Self {
            store: Mutex::new(HashMap::new()),
        }
    }
}

impl Cache for RedisCache {
    fn get(&self, key: &str) -> Option<String> {
        self.store.lock().unwrap().get(key).cloned()
    }

    fn set(&self, key: &str, value: String) {
        self.store.lock().unwrap().insert(key.to_string(), value);
    }
}

// Console logger implementation
pub struct ConsoleLogger;

impl Logger for ConsoleLogger {
    fn info(&self, message: &str) {
        println!("[INFO] {}", message);
    }

    fn error(&self, message: &str) {
        eprintln!("[ERROR] {}", message);
    }
}
```

### Usage

```rust
fn main() {
    // Create dependencies
    let database = Arc::new(PostgresDatabase::new(
        "postgres://localhost/mydb".to_string(),
    ));
    let cache = Arc::new(RedisCache::new());
    let logger = Arc::new(ConsoleLogger);

    // Inject dependencies
    let service = UserService::new(database, cache, logger);

    // Use service
    match service.get_user(1) {
        Ok(user) => println!("Found user: {:?}", user),
        Err(e) => eprintln!("Error: {:?}", e),
    }
}
```

### Benefits

- ✅ **Explicit Dependencies**: Clear what the service needs
- ✅ **Immutable**: Dependencies cannot be changed after construction
- ✅ **Type-Safe**: Compiler ensures all dependencies provided
- ✅ **Easy to Test**: Inject mocks instead of real implementations
- ✅ **No Magic**: Simple, straightforward pattern

### Drawbacks

- ⚠️ **Long Constructors**: With many dependencies (5+), constructor becomes unwieldy
- ⚠️ **No Defaults**: Cannot provide default implementations
- ⚠️ **No Validation**: Cannot validate dependencies before construction (use Builder pattern)

---

## 4. Pattern 2: Builder-Based Dependency Injection

### Overview

**Best For:** Complex dependencies, optional dependencies, validation needed, 5+ dependencies

The Builder pattern is ideal when you have many dependencies, some optional, or need validation before construction.

### Builder with Optional Dependencies

```rust
use std::sync::Arc;

// Service with many dependencies
pub struct ApplicationService {
    database: Arc<dyn Database>,
    cache: Option<Arc<dyn Cache>>,
    logger: Arc<dyn Logger>,
    metrics: Option<Arc<dyn Metrics>>,
    config: Config,
}

impl ApplicationService {
    pub fn builder() -> ApplicationServiceBuilder {
        ApplicationServiceBuilder::new()
    }

    pub fn process(&self, data: &str) -> Result<(), ProcessError> {
        self.logger.info("Processing data");

        if let Some(cache) = &self.cache {
            if let Some(cached) = cache.get(data) {
                self.logger.info("Cache hit");
                return Ok(());
            }
        }

        // Process data
        self.database.execute(&format!("INSERT INTO data VALUES ('{}')", data))
            .map_err(|_| ProcessError::DatabaseError)?;

        if let Some(metrics) = &self.metrics {
            metrics.increment("data_processed");
        }

        Ok(())
    }
}

#[derive(Debug)]
pub enum ProcessError {
    DatabaseError,
    ValidationError(String),
}

// Builder for ApplicationService
pub struct ApplicationServiceBuilder {
    database: Option<Arc<dyn Database>>,
    cache: Option<Arc<dyn Cache>>,
    logger: Option<Arc<dyn Logger>>,
    metrics: Option<Arc<dyn Metrics>>,
    config: Option<Config>,
}

impl ApplicationServiceBuilder {
    pub fn new() -> Self {
        Self {
            database: None,
            cache: None,
            logger: None,
            metrics: None,
            config: None,
        }
    }

    pub fn database(mut self, database: Arc<dyn Database>) -> Self {
        self.database = Some(database);
        self
    }

    pub fn cache(mut self, cache: Arc<dyn Cache>) -> Self {
        self.cache = Some(cache);
        self
    }

    pub fn logger(mut self, logger: Arc<dyn Logger>) -> Self {
        self.logger = Some(logger);
        self
    }

    pub fn metrics(mut self, metrics: Arc<dyn Metrics>) -> Self {
        self.metrics = Some(metrics);
        self
    }

    pub fn config(mut self, config: Config) -> Self {
        self.config = Some(config);
        self
    }

    pub fn build(self) -> Result<ApplicationService, BuildError> {
        // Validate required dependencies
        let database = self.database
            .ok_or(BuildError::MissingDependency("database"))?;
        
        let logger = self.logger
            .ok_or(BuildError::MissingDependency("logger"))?;
        
        let config = self.config
            .ok_or(BuildError::MissingDependency("config"))?;

        // Validate configuration
        if config.max_connections == 0 {
            return Err(BuildError::InvalidConfig("max_connections must be > 0"));
        }

        Ok(ApplicationService {
            database,
            cache: self.cache,  // Optional
            logger,
            metrics: self.metrics,  // Optional
            config,
        })
    }
}

#[derive(Debug)]
pub enum BuildError {
    MissingDependency(&'static str),
    InvalidConfig(&'static str),
}

#[derive(Debug, Clone)]
pub struct Config {
    pub max_connections: usize,
    pub timeout_ms: u64,
}

// Additional trait for metrics
pub trait Metrics: Send + Sync {
    fn increment(&self, metric: &str);
    fn record(&self, metric: &str, value: f64);
}
```

### Usage

```rust
fn main() -> Result<(), BuildError> {
    // Create dependencies
    let database = Arc::new(PostgresDatabase::new("postgres://localhost".to_string()));
    let cache = Arc::new(RedisCache::new());
    let logger = Arc::new(ConsoleLogger);
    let metrics = Arc::new(PrometheusMetrics::new());

    // Build service using builder pattern
    let service = ApplicationService::builder()
        .database(database)
        .cache(cache)  // Optional
        .logger(logger)
        .metrics(metrics)  // Optional
        .config(Config {
            max_connections: 10,
            timeout_ms: 5000,
        })
        .build()?;

    service.process("test data").unwrap();
    Ok(())
}

// Can also build minimal version without optional dependencies
fn minimal_setup() -> Result<(), BuildError> {
    let database = Arc::new(PostgresDatabase::new("postgres://localhost".to_string()));
    let logger = Arc::new(ConsoleLogger);

    let service = ApplicationService::builder()
        .database(database)
        .logger(logger)
        .config(Config {
            max_connections: 5,
            timeout_ms: 3000,
        })
        .build()?;  // No cache, no metrics - still works

    Ok(())
}
```

### Type-State Builder (Compile-Time Validation)

For even stronger guarantees, use type-state pattern to enforce required dependencies at compile time:

```rust
use std::marker::PhantomData;

// Type states
pub struct NeedDatabase;
pub struct NeedLogger;
pub struct Ready;

pub struct TypedApplicationServiceBuilder<State> {
    database: Option<Arc<dyn Database>>,
    logger: Option<Arc<dyn Logger>>,
    cache: Option<Arc<dyn Cache>>,
    _state: PhantomData<State>,
}

impl TypedApplicationServiceBuilder<NeedDatabase> {
    pub fn new() -> Self {
        Self {
            database: None,
            logger: None,
            cache: None,
            _state: PhantomData,
        }
    }

    pub fn database(
        self,
        database: Arc<dyn Database>,
    ) -> TypedApplicationServiceBuilder<NeedLogger> {
        TypedApplicationServiceBuilder {
            database: Some(database),
            logger: None,
            cache: None,
            _state: PhantomData,
        }
    }
}

impl TypedApplicationServiceBuilder<NeedLogger> {
    pub fn logger(
        self,
        logger: Arc<dyn Logger>,
    ) -> TypedApplicationServiceBuilder<Ready> {
        TypedApplicationServiceBuilder {
            database: self.database,
            logger: Some(logger),
            cache: self.cache,
            _state: PhantomData,
        }
    }

    pub fn cache(mut self, cache: Arc<dyn Cache>) -> Self {
        self.cache = Some(cache);
        self
    }
}

impl TypedApplicationServiceBuilder<Ready> {
    pub fn cache(mut self, cache: Arc<dyn Cache>) -> Self {
        self.cache = Some(cache);
        self
    }

    pub fn build(self) -> ApplicationService {
        ApplicationService {
            database: self.database.unwrap(),  // Safe: guaranteed by type state
            cache: self.cache,
            logger: self.logger.unwrap(),  // Safe: guaranteed by type state
            metrics: None,
            config: Config {
                max_connections: 10,
                timeout_ms: 5000,
            },
        }
    }
}

// Usage: Compile-time enforcement
fn typed_builder_usage() {
    let database = Arc::new(PostgresDatabase::new("postgres://localhost".to_string()));
    let logger = Arc::new(ConsoleLogger);
    let cache = Arc::new(RedisCache::new());

    // This compiles - all required dependencies provided
    let service = TypedApplicationServiceBuilder::new()
        .database(database)
        .logger(logger)
        .cache(cache)
        .build();

    // This won't compile - missing logger
    // let service = TypedApplicationServiceBuilder::new()
    //     .database(database)
    //     .build();  // ❌ Compile error: no method `build` on type `NeedLogger`
}
```

### Benefits

- ✅ **Optional Dependencies**: Can omit optional dependencies
- ✅ **Default Values**: Can provide defaults in builder
- ✅ **Validation**: Validate before construction
- ✅ **Readable**: Clear what's being configured
- ✅ **Flexible**: Easy to add new dependencies without breaking existing code
- ✅ **Type-State**: Compile-time guarantees with type-state pattern

### Drawbacks

- ⚠️ **More Code**: Requires builder struct and implementation
- ⚠️ **Complexity**: Type-state builders are complex for simple cases

---

## 5. Pattern 3: Factory-Based Dependency Injection

### Overview

**Best For:** Multiple configurations (production, test, dev), environment-specific dependencies

Factories encapsulate the complexity of creating objects with their dependencies. Useful when you need different configurations for different environments.

### Basic Factory Pattern

```rust
use std::sync::Arc;

// Factory for creating UserService with different configurations
pub struct UserServiceFactory;

impl UserServiceFactory {
    /// Create service for production environment
    pub fn create_production(db_url: &str) -> Result<UserService, FactoryError> {
        let database = Arc::new(PostgresDatabase::new(db_url.to_string()));
        let cache = Arc::new(RedisCache::new());
        let logger = Arc::new(FileLogger::new("/var/log/app.log")?);

        Ok(UserService::new(database, cache, logger))
    }

    /// Create service for testing environment
    pub fn create_test() -> UserService {
        let database = Arc::new(InMemoryDatabase::new());
        let cache = Arc::new(InMemoryCache::new());
        let logger = Arc::new(NoOpLogger);

        UserService::new(database, cache, logger)
    }

    /// Create service for local development
    pub fn create_development() -> Result<UserService, FactoryError> {
        let database = Arc::new(PostgresDatabase::new("postgres://localhost/dev".to_string()));
        let cache = Arc::new(InMemoryCache::new());  // Use in-memory cache for dev
        let logger = Arc::new(ConsoleLogger);

        Ok(UserService::new(database, cache, logger))
    }

    /// Create service with custom configuration
    pub fn create_custom(config: ServiceConfig) -> Result<UserService, FactoryError> {
        let database: Arc<dyn Database> = if config.use_postgres {
            Arc::new(PostgresDatabase::new(config.db_url))
        } else {
            Arc::new(InMemoryDatabase::new())
        };

        let cache: Arc<dyn Cache> = if config.use_redis {
            Arc::new(RedisCache::new())
        } else {
            Arc::new(InMemoryCache::new())
        };

        let logger: Arc<dyn Logger> = match config.log_target {
            LogTarget::File => Arc::new(FileLogger::new(&config.log_path)?),
            LogTarget::Console => Arc::new(ConsoleLogger),
            LogTarget::None => Arc::new(NoOpLogger),
        };

        Ok(UserService::new(database, cache, logger))
    }
}

#[derive(Debug)]
pub enum FactoryError {
    IoError(String),
    ConfigError(String),
}

pub struct ServiceConfig {
    pub use_postgres: bool,
    pub db_url: String,
    pub use_redis: bool,
    pub log_target: LogTarget,
    pub log_path: String,
}

pub enum LogTarget {
    File,
    Console,
    None,
}

// Test doubles for in-memory implementations
pub struct InMemoryDatabase {
    data: std::sync::Mutex<std::collections::HashMap<String, Vec<String>>>,
}

impl InMemoryDatabase {
    pub fn new() -> Self {
        Self {
            data: std::sync::Mutex::new(std::collections::HashMap::new()),
        }
    }
}

impl Database for InMemoryDatabase {
    fn query(&self, _sql: &str) -> Result<Vec<String>, DbError> {
        Ok(vec![])
    }

    fn execute(&self, _sql: &str) -> Result<u64, DbError> {
        Ok(1)
    }
}

pub struct InMemoryCache {
    store: std::sync::Mutex<std::collections::HashMap<String, String>>,
}

impl InMemoryCache {
    pub fn new() -> Self {
        Self {
            store: std::sync::Mutex::new(std::collections::HashMap::new()),
        }
    }
}

impl Cache for InMemoryCache {
    fn get(&self, key: &str) -> Option<String> {
        self.store.lock().unwrap().get(key).cloned()
    }

    fn set(&self, key: &str, value: String) {
        self.store.lock().unwrap().insert(key.to_string(), value);
    }
}

pub struct NoOpLogger;

impl Logger for NoOpLogger {
    fn info(&self, _message: &str) {}
    fn error(&self, _message: &str) {}
}

pub struct FileLogger {
    path: String,
}

impl FileLogger {
    pub fn new(path: &str) -> Result<Self, FactoryError> {
        // Validate path exists
        Ok(Self {
            path: path.to_string(),
        })
    }
}

impl Logger for FileLogger {
    fn info(&self, message: &str) {
        println!("[FILE:{}] INFO: {}", self.path, message);
    }

    fn error(&self, message: &str) {
        eprintln!("[FILE:{}] ERROR: {}", self.path, message);
    }
}
```

### Usage

```rust
fn main() -> Result<(), FactoryError> {
    // Production
    let prod_service = UserServiceFactory::create_production("postgres://prod.example.com/db")?;

    // Testing
    let test_service = UserServiceFactory::create_test();

    // Development
    let dev_service = UserServiceFactory::create_development()?;

    // Custom
    let custom_service = UserServiceFactory::create_custom(ServiceConfig {
        use_postgres: true,
        db_url: "postgres://custom.example.com/db".to_string(),
        use_redis: false,
        log_target: LogTarget::Console,
        log_path: String::new(),
    })?;

    Ok(())
}

// Environment-based factory selection
fn create_service_for_environment(env: &str) -> Result<UserService, FactoryError> {
    match env {
        "production" => UserServiceFactory::create_production("postgres://prod/db"),
        "test" => Ok(UserServiceFactory::create_test()),
        "development" => UserServiceFactory::create_development(),
        _ => Err(FactoryError::ConfigError(format!("Unknown environment: {}", env))),
    }
}
```

### Abstract Factory for Related Dependencies

```rust
// Abstract factory for creating related dependencies
pub trait DependencyFactory: Send + Sync {
    fn create_database(&self) -> Arc<dyn Database>;
    fn create_cache(&self) -> Arc<dyn Cache>;
    fn create_logger(&self) -> Arc<dyn Logger>;
}

pub struct ProductionFactory {
    db_url: String,
}

impl ProductionFactory {
    pub fn new(db_url: String) -> Self {
        Self { db_url }
    }
}

impl DependencyFactory for ProductionFactory {
    fn create_database(&self) -> Arc<dyn Database> {
        Arc::new(PostgresDatabase::new(self.db_url.clone()))
    }

    fn create_cache(&self) -> Arc<dyn Cache> {
        Arc::new(RedisCache::new())
    }

    fn create_logger(&self) -> Arc<dyn Logger> {
        Arc::new(FileLogger::new("/var/log/app.log").unwrap())
    }
}

pub struct TestFactory;

impl DependencyFactory for TestFactory {
    fn create_database(&self) -> Arc<dyn Database> {
        Arc::new(InMemoryDatabase::new())
    }

    fn create_cache(&self) -> Arc<dyn Cache> {
        Arc::new(InMemoryCache::new())
    }

    fn create_logger(&self) -> Arc<dyn Logger> {
        Arc::new(NoOpLogger)
    }
}

// Use abstract factory
pub struct Application {
    factory: Arc<dyn DependencyFactory>,
}

impl Application {
    pub fn new(factory: Arc<dyn DependencyFactory>) -> Self {
        Self { factory }
    }

    pub fn create_user_service(&self) -> UserService {
        UserService::new(
            self.factory.create_database(),
            self.factory.create_cache(),
            self.factory.create_logger(),
        )
    }
}

// Usage
fn main() {
    let factory = Arc::new(ProductionFactory::new("postgres://localhost/db".to_string()));
    let app = Application::new(factory);
    let service = app.create_user_service();
}
```

### Benefits

- ✅ **Centralized Configuration**: All wiring in one place
- ✅ **Environment-Specific**: Easy to switch between environments
- ✅ **Encapsulation**: Hides complexity of creating dependencies
- ✅ **Consistency**: Same configuration used everywhere

### Drawbacks

- ⚠️ **Indirection**: Extra layer between usage and creation
- ⚠️ **Factory Explosion**: Can have many factory methods

---

## 6. Pattern 4: Trait Objects vs Generics

### Overview

Rust offers two approaches to polymorphism: trait objects (`dyn Trait`) and generics (`<T: Trait>`). Choose based on your needs.

### Trait Objects: `Arc<dyn Trait>` (Dynamic Dispatch)

**Use when:**
- Need runtime polymorphism
- Want to store different implementations in the same collection
- DI with swappable implementations
- Binary size is a concern

```rust
use std::sync::Arc;

// Trait object approach
pub struct ApiHandler {
    auth: Arc<dyn AuthService>,  // Runtime polymorphism
    storage: Arc<dyn StorageService>,
}

impl ApiHandler {
    pub fn new(
        auth: Arc<dyn AuthService>,
        storage: Arc<dyn StorageService>,
    ) -> Self {
        Self { auth, storage }
    }

    pub fn handle_request(&self, token: &str) -> Result<String, Error> {
        // Dynamic dispatch - decided at runtime
        if self.auth.validate(token)? {
            Ok(self.storage.read("data")?)
        } else {
            Err(Error::Unauthorized)
        }
    }
}

pub trait AuthService: Send + Sync {
    fn validate(&self, token: &str) -> Result<bool, Error>;
}

pub trait StorageService: Send + Sync {
    fn read(&self, key: &str) -> Result<String, Error>;
}

#[derive(Debug)]
pub enum Error {
    Unauthorized,
    NotFound,
}

// Can store different implementations
fn create_handlers() -> Vec<ApiHandler> {
    vec![
        ApiHandler::new(
            Arc::new(OAuth2Auth::new()),
            Arc::new(S3Storage::new()),
        ),
        ApiHandler::new(
            Arc::new(BasicAuth::new()),
            Arc::new(LocalStorage::new()),
        ),
    ]
}

struct OAuth2Auth;
impl OAuth2Auth {
    fn new() -> Self { Self }
}
impl AuthService for OAuth2Auth {
    fn validate(&self, _token: &str) -> Result<bool, Error> {
        Ok(true)
    }
}

struct BasicAuth;
impl BasicAuth {
    fn new() -> Self { Self }
}
impl AuthService for BasicAuth {
    fn validate(&self, _token: &str) -> Result<bool, Error> {
        Ok(true)
    }
}

struct S3Storage;
impl S3Storage {
    fn new() -> Self { Self }
}
impl StorageService for S3Storage {
    fn read(&self, _key: &str) -> Result<String, Error> {
        Ok("data from S3".to_string())
    }
}

struct LocalStorage;
impl LocalStorage {
    fn new() -> Self { Self }
}
impl StorageService for LocalStorage {
    fn read(&self, _key: &str) -> Result<String, Error> {
        Ok("data from local".to_string())
    }
}
```

### Generics: `<T: Trait>` (Static Dispatch)

**Use when:**
- Need maximum performance (zero-cost abstraction)
- Implementation type known at compile time
- Want monomorphization for optimization

```rust
use std::sync::Arc;

// Generic approach
pub struct ApiHandler<A, S>
where
    A: AuthService,
    S: StorageService,
{
    auth: Arc<A>,  // Static dispatch - known at compile time
    storage: Arc<S>,
}

impl<A, S> ApiHandler<A, S>
where
    A: AuthService,
    S: StorageService,
{
    pub fn new(auth: Arc<A>, storage: Arc<S>) -> Self {
        Self { auth, storage }
    }

    pub fn handle_request(&self, token: &str) -> Result<String, Error> {
        // Static dispatch - inlined at compile time
        if self.auth.validate(token)? {
            Ok(self.storage.read("data")?)
        } else {
            Err(Error::Unauthorized)
        }
    }
}

// Each combination creates a different type
fn main() {
    let handler1: ApiHandler<OAuth2Auth, S3Storage> = ApiHandler::new(
        Arc::new(OAuth2Auth::new()),
        Arc::new(S3Storage::new()),
    );

    let handler2: ApiHandler<BasicAuth, LocalStorage> = ApiHandler::new(
        Arc::new(BasicAuth::new()),
        Arc::new(LocalStorage::new()),
    );

    // Cannot store different generic types in same vec
    // let handlers = vec![handler1, handler2];  // ❌ Compile error: different types
}
```

### Comparison Table

| Aspect | `Arc<dyn Trait>` | `<T: Trait>` |
|--------|-----------------|--------------|
| **Dispatch** | Dynamic (vtable) | Static (monomorphization) |
| **Performance** | Small overhead (~3-5%) | Zero-cost |
| **Binary Size** | Smaller | Larger (code for each type) |
| **Collections** | Can store different impls | Cannot mix types |
| **DI Flexibility** | High (swap at runtime) | Low (fixed at compile time) |
| **Use Case** | Plugin systems, DI | Performance-critical code |

### Hybrid Approach

Combine both for flexibility and performance:

```rust
// Generic for hot path (performance-critical)
pub struct DataProcessor<S: StorageService> {
    storage: Arc<S>,  // Static dispatch for performance
    logger: Arc<dyn Logger>,  // Dynamic dispatch for flexibility
}

impl<S: StorageService> DataProcessor<S> {
    pub fn new(storage: Arc<S>, logger: Arc<dyn Logger>) -> Self {
        Self { storage, logger }
    }

    pub fn process(&self, key: &str) -> Result<(), Error> {
        self.logger.info("Starting processing");  // Dynamic dispatch (flexible)
        let data = self.storage.read(key)?;  // Static dispatch (fast)
        self.logger.info("Processing complete");
        Ok(())
    }
}
```

### When to Choose

```
Need to store different implementations in same collection?
├─ Yes → Use `Arc<dyn Trait>`
└─ No
    │
    ├─ Performance-critical hot path?
    │  ├─ Yes → Use `<T: Trait>`
    │  └─ No → Either works, prefer `Arc<dyn Trait>` for flexibility
    │
    └─ Need runtime configuration?
       ├─ Yes → Use `Arc<dyn Trait>`
       └─ No → Use `<T: Trait>`
```

---

## 7. Pattern 5: Async Dependency Injection

### Overview

**Best For:** Async services, I/O-bound operations, web servers

Async dependencies require special handling. Use `async_trait` for async trait methods and `Arc` for sharing across tasks.

### Async Traits with `async_trait`

```rust
use async_trait::async_trait;
use std::sync::Arc;
use tokio;

// Async trait definition
#[async_trait]
pub trait AsyncDatabase: Send + Sync {
    async fn query(&self, sql: &str) -> Result<Vec<String>, DbError>;
    async fn execute(&self, sql: &str) -> Result<u64, DbError>;
}

#[async_trait]
pub trait AsyncCache: Send + Sync {
    async fn get(&self, key: &str) -> Result<Option<String>, CacheError>;
    async fn set(&self, key: &str, value: String) -> Result<(), CacheError>;
}

#[derive(Debug)]
pub enum DbError {
    ConnectionError(String),
    QueryError(String),
}

#[derive(Debug)]
pub enum CacheError {
    ConnectionError(String),
}

// Async service with injected dependencies
pub struct AsyncUserService {
    database: Arc<dyn AsyncDatabase>,
    cache: Arc<dyn AsyncCache>,
}

impl AsyncUserService {
    pub fn new(
        database: Arc<dyn AsyncDatabase>,
        cache: Arc<dyn AsyncCache>,
    ) -> Self {
        Self { database, cache }
    }

    pub async fn get_user(&self, user_id: u64) -> Result<User, ServiceError> {
        let cache_key = format!("user:{}", user_id);

        // Try cache first (async operation)
        if let Some(cached) = self.cache.get(&cache_key).await.ok().flatten() {
            return Ok(serde_json::from_str(&cached).unwrap());
        }

        // Query database (async operation)
        let results = self.database
            .query(&format!("SELECT * FROM users WHERE id = {}", user_id))
            .await
            .map_err(|_| ServiceError::NotFound)?;

        if results.is_empty() {
            return Err(ServiceError::NotFound);
        }

        let user: User = serde_json::from_str(&results[0]).unwrap();

        // Update cache (async operation)
        let _ = self.cache
            .set(&cache_key, serde_json::to_string(&user).unwrap())
            .await;

        Ok(user)
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct User {
    pub id: u64,
    pub name: String,
    pub email: String,
}

#[derive(Debug)]
pub enum ServiceError {
    NotFound,
}

// Concrete async implementations
pub struct AsyncPostgresDatabase {
    connection_string: String,
}

impl AsyncPostgresDatabase {
    pub fn new(connection_string: String) -> Self {
        Self { connection_string }
    }
}

#[async_trait]
impl AsyncDatabase for AsyncPostgresDatabase {
    async fn query(&self, sql: &str) -> Result<Vec<String>, DbError> {
        // Simulate async database query
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        println!("[AsyncPostgres] Query: {}", sql);
        Ok(vec![r#"{"id":1,"name":"Alice","email":"alice@example.com"}"#.to_string()])
    }

    async fn execute(&self, sql: &str) -> Result<u64, DbError> {
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        println!("[AsyncPostgres] Execute: {}", sql);
        Ok(1)
    }
}

pub struct AsyncRedisCache;

impl AsyncRedisCache {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl AsyncCache for AsyncRedisCache {
    async fn get(&self, key: &str) -> Result<Option<String>, CacheError> {
        tokio::time::sleep(tokio::time::Duration::from_millis(5)).await;
        println!("[AsyncRedis] GET: {}", key);
        Ok(None)
    }

    async fn set(&self, key: &str, value: String) -> Result<(), CacheError> {
        tokio::time::sleep(tokio::time::Duration::from_millis(5)).await;
        println!("[AsyncRedis] SET: {} = {}", key, value);
        Ok(())
    }
}
```

### Usage with Tokio

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create async dependencies
    let database = Arc::new(AsyncPostgresDatabase::new(
        "postgres://localhost/mydb".to_string(),
    ));
    let cache = Arc::new(AsyncRedisCache::new());

    // Inject dependencies
    let service = AsyncUserService::new(database, cache);

    // Use async service
    let user = service.get_user(1).await?;
    println!("User: {:?}", user);

    Ok(())
}
```

### Sharing Async Services Across Tasks

```rust
use tokio::task::JoinHandle;

async fn spawn_multiple_handlers(
    service: Arc<AsyncUserService>,
) -> Vec<JoinHandle<Result<User, ServiceError>>> {
    let mut handles = Vec::new();

    for user_id in 1..=5 {
        let service_clone = Arc::clone(&service);
        let handle = tokio::spawn(async move {
            service_clone.get_user(user_id).await
        });
        handles.push(handle);
    }

    handles
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let database = Arc::new(AsyncPostgresDatabase::new("postgres://localhost/mydb".to_string()));
    let cache = Arc::new(AsyncRedisCache::new());
    let service = Arc::new(AsyncUserService::new(database, cache));

    // Spawn multiple concurrent tasks
    let handles = spawn_multiple_handlers(service).await;

    // Wait for all tasks to complete
    for handle in handles {
        match handle.await? {
            Ok(user) => println!("Got user: {:?}", user),
            Err(e) => eprintln!("Error: {:?}", e),
        }
    }

    Ok(())
}
```

### Async Factory Pattern

```rust
pub struct AsyncServiceFactory;

impl AsyncServiceFactory {
    pub async fn create_production(db_url: &str) -> Result<AsyncUserService, FactoryError> {
        // Async initialization
        let database = AsyncPostgresDatabase::new(db_url.to_string());
        // Could await connection initialization here
        // database.connect().await?;

        let cache = AsyncRedisCache::new();
        // cache.connect().await?;

        Ok(AsyncUserService::new(
            Arc::new(database),
            Arc::new(cache),
        ))
    }

    pub fn create_test() -> AsyncUserService {
        let database = Arc::new(InMemoryAsyncDatabase::new());
        let cache = Arc::new(InMemoryAsyncCache::new());

        AsyncUserService::new(database, cache)
    }
}

pub struct InMemoryAsyncDatabase;

impl InMemoryAsyncDatabase {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl AsyncDatabase for InMemoryAsyncDatabase {
    async fn query(&self, _sql: &str) -> Result<Vec<String>, DbError> {
        Ok(vec![])
    }

    async fn execute(&self, _sql: &str) -> Result<u64, DbError> {
        Ok(1)
    }
}

pub struct InMemoryAsyncCache;

impl InMemoryAsyncCache {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl AsyncCache for InMemoryAsyncCache {
    async fn get(&self, _key: &str) -> Result<Option<String>, CacheError> {
        Ok(None)
    }

    async fn set(&self, _key: &str, _value: String) -> Result<(), CacheError> {
        Ok(())
    }
}

#[derive(Debug)]
pub enum FactoryError {
    ConnectionError(String),
}
```

### Benefits

- ✅ **Concurrent Operations**: Multiple async tasks can share service
- ✅ **Non-Blocking**: I/O operations don't block threads
- ✅ **Scalable**: Handles many concurrent requests efficiently

### Drawbacks

- ⚠️ **Complexity**: Async code is harder to reason about
- ⚠️ **Debugging**: Stack traces can be confusing
- ⚠️ **Dependency**: Requires `async_trait` crate

---

## 8. Pattern 6: Scoped Dependencies and Lifetimes

### Overview

**Best For:** Dependencies with specific lifetimes, request-scoped services, temporary dependencies

Sometimes dependencies don't need to live for the entire application lifetime. Use lifetimes to express these relationships.

### Borrowed Dependencies

```rust
// Service that borrows dependencies (no Arc needed)
pub struct RequestHandler<'a> {
    database: &'a dyn Database,
    cache: &'a dyn Cache,
    logger: &'a dyn Logger,
}

impl<'a> RequestHandler<'a> {
    pub fn new(
        database: &'a dyn Database,
        cache: &'a dyn Cache,
        logger: &'a dyn Logger,
    ) -> Self {
        Self {
            database,
            cache,
            logger,
        }
    }

    pub fn handle_request(&self, user_id: u64) -> Result<String, ServiceError> {
        self.logger.info(&format!("Handling request for user {}", user_id));

        let cache_key = format!("user:{}", user_id);
        if let Some(cached) = self.cache.get(&cache_key) {
            return Ok(cached);
        }

        let results = self.database
            .query(&format!("SELECT * FROM users WHERE id = {}", user_id))
            .map_err(|_| ServiceError::NotFound)?;

        Ok(results.first().unwrap_or(&String::new()).clone())
    }
}

// Usage: Dependencies live on stack
fn handle_http_request(user_id: u64) -> Result<String, ServiceError> {
    let database = PostgresDatabase::new("postgres://localhost/db".to_string());
    let cache = InMemoryCache::new();
    let logger = ConsoleLogger;

    // Handler borrows dependencies
    let handler = RequestHandler::new(&database, &cache, &logger);
    handler.handle_request(user_id)
}  // Dependencies dropped here
```

### Request-Scoped Services

```rust
use std::sync::Arc;

// Application-scoped dependencies (long-lived)
pub struct AppState {
    database: Arc<dyn Database>,
    config: Arc<Config>,
}

impl AppState {
    pub fn new(database: Arc<dyn Database>, config: Arc<Config>) -> Self {
        Self { database, config }
    }
}

// Request-scoped service (short-lived)
pub struct RequestService<'a> {
    app_state: &'a AppState,
    request_id: String,
    logger: ConsoleLogger,  // Owned, created per request
}

impl<'a> RequestService<'a> {
    pub fn new(app_state: &'a AppState, request_id: String) -> Self {
        Self {
            app_state,
            request_id,
            logger: ConsoleLogger,
        }
    }

    pub fn process(&self, data: &str) -> Result<(), ServiceError> {
        self.logger.info(&format!("[{}] Processing: {}", self.request_id, data));

        self.app_state
            .database
            .execute(&format!("INSERT INTO requests VALUES ('{}')", data))
            .map_err(|_| ServiceError::NotFound)?;

        Ok(())
    }
}

// Usage: Web server pattern
async fn handle_web_request(
    app_state: Arc<AppState>,
    request_id: String,
) -> Result<String, ServiceError> {
    // Create request-scoped service
    let service = RequestService::new(&app_state, request_id);
    service.process("user data")?;
    Ok("Success".to_string())
}

#[tokio::main]
async fn main() {
    // Create application-scoped dependencies (live for entire app)
    let database = Arc::new(PostgresDatabase::new("postgres://localhost/db".to_string()));
    let config = Arc::new(Config {
        max_connections: 10,
        timeout_ms: 5000,
    });
    let app_state = Arc::new(AppState::new(database, config));

    // Simulate multiple requests
    let request1 = handle_web_request(Arc::clone(&app_state), "req-1".to_string());
    let request2 = handle_web_request(Arc::clone(&app_state), "req-2".to_string());

    let _ = tokio::join!(request1, request2);
}
```

### Builder with Lifetimes

```rust
pub struct ServiceBuilder<'a> {
    database: Option<&'a dyn Database>,
    cache: Option<&'a dyn Cache>,
    logger: Option<&'a dyn Logger>,
}

impl<'a> ServiceBuilder<'a> {
    pub fn new() -> Self {
        Self {
            database: None,
            cache: None,
            logger: None,
        }
    }

    pub fn database(mut self, database: &'a dyn Database) -> Self {
        self.database = Some(database);
        self
    }

    pub fn cache(mut self, cache: &'a dyn Cache) -> Self {
        self.cache = Some(cache);
        self
    }

    pub fn logger(mut self, logger: &'a dyn Logger) -> Self {
        self.logger = Some(logger);
        self
    }

    pub fn build(self) -> Result<RequestHandler<'a>, BuildError> {
        Ok(RequestHandler {
            database: self.database.ok_or(BuildError::MissingDependency("database"))?,
            cache: self.cache.ok_or(BuildError::MissingDependency("cache"))?,
            logger: self.logger.ok_or(BuildError::MissingDependency("logger"))?,
        })
    }
}

// Usage
fn create_handler() -> Result<(), BuildError> {
    let database = PostgresDatabase::new("postgres://localhost/db".to_string());
    let cache = InMemoryCache::new();
    let logger = ConsoleLogger;

    let handler = ServiceBuilder::new()
        .database(&database)
        .cache(&cache)
        .logger(&logger)
        .build()?;

    Ok(())
}
```

### Benefits

- ✅ **No Allocation**: Borrowed dependencies don't need `Arc`
- ✅ **Clear Lifetimes**: Compiler enforces dependency lifetime
- ✅ **Efficient**: No reference counting overhead
- ✅ **Request-Scoped**: Perfect for web servers and request handling

### Drawbacks

- ⚠️ **Lifetime Annotations**: More complex type signatures
- ⚠️ **Limited Flexibility**: Cannot move service across thread boundaries
- ⚠️ **Stack Bound**: Dependencies must outlive the service

---

## 9. Testing with Dependency Injection

### Overview

DI makes testing easy by allowing mock implementations to replace real dependencies.

### Strategy 1: Simple Mock Structs

**Best for:** Simple traits, basic verification

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    // Mock logger that records messages
    struct MockLogger {
        messages: Mutex<Vec<String>>,
    }

    impl MockLogger {
        fn new() -> Self {
            Self {
                messages: Mutex::new(Vec::new()),
            }
        }

        fn get_messages(&self) -> Vec<String> {
            self.messages.lock().unwrap().clone()
        }

        fn clear(&self) {
            self.messages.lock().unwrap().clear();
        }
    }

    impl Logger for MockLogger {
        fn info(&self, message: &str) {
            self.messages.lock().unwrap().push(format!("INFO: {}", message));
        }

        fn error(&self, message: &str) {
            self.messages.lock().unwrap().push(format!("ERROR: {}", message));
        }
    }

    // Mock database that tracks queries
    struct MockDatabase {
        queries: Mutex<Vec<String>>,
        query_results: Mutex<Vec<Vec<String>>>,
    }

    impl MockDatabase {
        fn new() -> Self {
            Self {
                queries: Mutex::new(Vec::new()),
                query_results: Mutex::new(Vec::new()),
            }
        }

        fn add_query_result(&self, result: Vec<String>) {
            self.query_results.lock().unwrap().push(result);
        }

        fn get_queries(&self) -> Vec<String> {
            self.queries.lock().unwrap().clone()
        }
    }

    impl Database for MockDatabase {
        fn query(&self, sql: &str) -> Result<Vec<String>, DbError> {
            self.queries.lock().unwrap().push(sql.to_string());
            let results = self.query_results.lock().unwrap();
            Ok(results.first().cloned().unwrap_or_default())
        }

        fn execute(&self, sql: &str) -> Result<u64, DbError> {
            self.queries.lock().unwrap().push(sql.to_string());
            Ok(1)
        }
    }

    // Mock cache
    struct MockCache {
        store: Mutex<std::collections::HashMap<String, String>>,
    }

    impl MockCache {
        fn new() -> Self {
            Self {
                store: Mutex::new(std::collections::HashMap::new()),
            }
        }
    }

    impl Cache for MockCache {
        fn get(&self, key: &str) -> Option<String> {
            self.store.lock().unwrap().get(key).cloned()
        }

        fn set(&self, key: &str, value: String) {
            self.store.lock().unwrap().insert(key.to_string(), value);
        }
    }

    #[test]
    fn test_get_user_cache_hit() {
        // Arrange
        let mock_db = Arc::new(MockDatabase::new());
        let mock_cache = Arc::new(MockCache::new());
        let mock_logger = Arc::new(MockLogger::new());

        // Pre-populate cache
        mock_cache.set("user:1", r#"{"id":1,"name":"Alice","email":"alice@example.com"}"#.to_string());

        let service = UserService::new(mock_db.clone(), mock_cache.clone(), mock_logger.clone());

        // Act
        let result = service.get_user(1);

        // Assert
        assert!(result.is_ok());
        let user = result.unwrap();
        assert_eq!(user.name, "Alice");

        // Verify logger was called
        let messages = mock_logger.get_messages();
        assert!(messages.iter().any(|m| m.contains("Cache hit")));

        // Verify database was NOT called (cache hit)
        let queries = mock_db.get_queries();
        assert_eq!(queries.len(), 0);
    }

    #[test]
    fn test_get_user_cache_miss() {
        // Arrange
        let mock_db = Arc::new(MockDatabase::new());
        mock_db.add_query_result(vec![
            r#"{"id":1,"name":"Bob","email":"bob@example.com"}"#.to_string(),
        ]);

        let mock_cache = Arc::new(MockCache::new());
        let mock_logger = Arc::new(MockLogger::new());

        let service = UserService::new(mock_db.clone(), mock_cache.clone(), mock_logger.clone());

        // Act
        let result = service.get_user(1);

        // Assert
        assert!(result.is_ok());
        let user = result.unwrap();
        assert_eq!(user.name, "Bob");

        // Verify logger
        let messages = mock_logger.get_messages();
        assert!(messages.iter().any(|m| m.contains("Cache miss")));

        // Verify database was called
        let queries = mock_db.get_queries();
        assert_eq!(queries.len(), 1);
        assert!(queries[0].contains("SELECT * FROM users"));

        // Verify cache was updated
        let cached = mock_cache.get("user:1");
        assert!(cached.is_some());
    }

    #[test]
    fn test_create_user() {
        // Arrange
        let mock_db = Arc::new(MockDatabase::new());
        let mock_cache = Arc::new(MockCache::new());
        let mock_logger = Arc::new(MockLogger::new());

        let service = UserService::new(mock_db.clone(), mock_cache.clone(), mock_logger.clone());

        // Act
        let result = service.create_user("Charlie", "charlie@example.com");

        // Assert
        assert!(result.is_ok());

        // Verify database insert was called
        let queries = mock_db.get_queries();
        assert_eq!(queries.len(), 1);
        assert!(queries[0].contains("INSERT INTO users"));
        assert!(queries[0].contains("Charlie"));

        // Verify logger
        let messages = mock_logger.get_messages();
        assert!(messages.iter().any(|m| m.contains("Creating user")));
    }
}
```

### Strategy 2: Using `mockall` Crate

**Best for:** Complex verification, parameter matching, call counting

Add to `Cargo.toml`:
```toml
[dev-dependencies]
mockall = "0.12"
```

```rust
#[cfg(test)]
mod mockall_tests {
    use super::*;
    use mockall::{automock, predicate::*};
    use std::sync::Arc;

    // Use #[automock] to generate mock
    #[automock]
    pub trait Database: Send + Sync {
        fn query(&self, sql: &str) -> Result<Vec<String>, DbError>;
        fn execute(&self, sql: &str) -> Result<u64, DbError>;
    }

    #[automock]
    pub trait Logger: Send + Sync {
        fn info(&self, message: &str);
        fn error(&self, message: &str);
    }

    #[test]
    fn test_with_mockall() {
        // Create mocks
        let mut mock_db = MockDatabase::new();
        let mut mock_logger = MockLogger::new();

        // Set expectations
        mock_db
            .expect_query()
            .with(eq("SELECT * FROM users WHERE id = 1"))
            .times(1)
            .returning(|_| Ok(vec![r#"{"id":1,"name":"Alice","email":"alice@example.com"}"#.to_string()]));

        mock_logger
            .expect_info()
            .with(eq("Cache miss for user 1"))
            .times(1)
            .returning(|_| ());

        mock_logger
            .expect_info()
            .with(always())  // Any other info calls
            .returning(|_| ());

        // Use mocks (need to work around the trait object issue)
        // Note: mockall works best with concrete types or careful trait design
    }

    #[test]
    fn test_execute_with_mockall() {
        let mut mock_db = MockDatabase::new();

        // Expect specific SQL with parameter matching
        mock_db
            .expect_execute()
            .withf(|sql| sql.contains("INSERT") && sql.contains("users"))
            .times(1)
            .returning(|_| Ok(1));

        // Test code that uses mock_db
    }
}
```

### Strategy 3: Test Doubles (In-Memory Implementations)

**Best for:** Integration tests, realistic behavior

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    use std::sync::Arc;

    // In-memory database for integration tests
    pub struct TestDatabase {
        users: Mutex<Vec<User>>,
    }

    impl TestDatabase {
        pub fn new_with_users(users: Vec<User>) -> Self {
            Self {
                users: Mutex::new(users),
            }
        }
    }

    impl Database for TestDatabase {
        fn query(&self, sql: &str) -> Result<Vec<String>, DbError> {
            if sql.contains("SELECT * FROM users WHERE id") {
                // Parse user ID from SQL (simplified)
                let users = self.users.lock().unwrap();
                Ok(users
                    .iter()
                    .map(|u| serde_json::to_string(u).unwrap())
                    .collect())
            } else {
                Ok(vec![])
            }
        }

        fn execute(&self, sql: &str) -> Result<u64, DbError> {
            if sql.contains("INSERT INTO users") {
                // Parse and add user (simplified)
                let mut users = self.users.lock().unwrap();
                users.push(User {
                    id: (users.len() + 1) as u64,
                    name: "Test User".to_string(),
                    email: "test@example.com".to_string(),
                });
                Ok(1)
            } else {
                Ok(0)
            }
        }
    }

    #[test]
    fn integration_test_user_service() {
        // Use realistic test doubles
        let test_db = Arc::new(TestDatabase::new_with_users(vec![
            User {
                id: 1,
                name: "Alice".to_string(),
                email: "alice@example.com".to_string(),
            },
        ]));

        let cache = Arc::new(InMemoryCache::new());
        let logger = Arc::new(ConsoleLogger);

        let service = UserService::new(test_db, cache, logger);

        // Test realistic flow
        let user = service.get_user(1).unwrap();
        assert_eq!(user.name, "Alice");

        // Second call should hit cache
        let user2 = service.get_user(1).unwrap();
        assert_eq!(user2.name, "Alice");
    }
}
```

### Testing Async Services

```rust
#[cfg(test)]
mod async_tests {
    use super::*;
    use async_trait::async_trait;
    use std::sync::Arc;
    use tokio;

    struct MockAsyncDatabase {
        responses: Mutex<Vec<Vec<String>>>,
    }

    impl MockAsyncDatabase {
        fn new() -> Self {
            Self {
                responses: Mutex::new(Vec::new()),
            }
        }

        fn add_response(&self, response: Vec<String>) {
            self.responses.lock().unwrap().push(response);
        }
    }

    #[async_trait]
    impl AsyncDatabase for MockAsyncDatabase {
        async fn query(&self, _sql: &str) -> Result<Vec<String>, DbError> {
            let mut responses = self.responses.lock().unwrap();
            Ok(responses.pop().unwrap_or_default())
        }

        async fn execute(&self, _sql: &str) -> Result<u64, DbError> {
            Ok(1)
        }
    }

    #[tokio::test]
    async fn test_async_service() {
        let mock_db = Arc::new(MockAsyncDatabase::new());
        mock_db.add_response(vec![
            r#"{"id":1,"name":"Alice","email":"alice@example.com"}"#.to_string(),
        ]);

        let cache = Arc::new(InMemoryAsyncCache::new());
        let service = AsyncUserService::new(mock_db, cache);

        let user = service.get_user(1).await.unwrap();
        assert_eq!(user.name, "Alice");
    }
}
```

---

## 10. Common Patterns

### Pattern: Application Composition Root

**Concept:** Wire all dependencies at the application entry point

```rust
use std::sync::Arc;

pub struct Application {
    user_service: Arc<UserService>,
    order_service: Arc<OrderService>,
}

impl Application {
    pub fn new() -> Result<Self, ApplicationError> {
        // Create shared dependencies
        let database = Arc::new(PostgresDatabase::new("postgres://localhost/db".to_string()));
        let cache = Arc::new(RedisCache::new());
        let logger = Arc::new(FileLogger::new("/var/log/app.log")?);

        // Create services with dependencies
        let user_service = Arc::new(UserService::new(
            Arc::clone(&database),
            Arc::clone(&cache),
            Arc::clone(&logger),
        ));

        let order_service = Arc::new(OrderService::new(
            Arc::clone(&database),
            Arc::clone(&logger),
        ));

        Ok(Self {
            user_service,
            order_service,
        })
    }

    pub fn user_service(&self) -> Arc<UserService> {
        Arc::clone(&self.user_service)
    }

    pub fn order_service(&self) -> Arc<OrderService> {
        Arc::clone(&self.order_service)
    }
}

#[derive(Debug)]
pub enum ApplicationError {
    IoError(String),
}

impl From<FactoryError> for ApplicationError {
    fn from(e: FactoryError) -> Self {
        match e {
            FactoryError::IoError(msg) => ApplicationError::IoError(msg),
            _ => ApplicationError::IoError("Unknown error".to_string()),
        }
    }
}

pub struct OrderService {
    database: Arc<dyn Database>,
    logger: Arc<dyn Logger>,
}

impl OrderService {
    pub fn new(database: Arc<dyn Database>, logger: Arc<dyn Logger>) -> Self {
        Self { database, logger }
    }
}

fn main() -> Result<(), ApplicationError> {
    let app = Application::new()?;

    // Use services
    let user_service = app.user_service();
    let _user = user_service.get_user(1);

    Ok(())
}
```

### Pattern: Configuration-Based Injection

**Concept:** Select dependencies based on configuration

```rust
use serde::Deserialize;
use std::sync::Arc;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub database_type: DatabaseType,
    pub database_url: String,
    pub cache_type: CacheType,
    pub log_level: LogLevel,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DatabaseType {
    Postgres,
    Mysql,
    InMemory,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CacheType {
    Redis,
    InMemory,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Debug,
    Info,
    Error,
}

pub struct ConfigurableApplication;

impl ConfigurableApplication {
    pub fn from_config(config: AppConfig) -> Result<Application, ApplicationError> {
        // Select database based on config
        let database: Arc<dyn Database> = match config.database_type {
            DatabaseType::Postgres => Arc::new(PostgresDatabase::new(config.database_url)),
            DatabaseType::Mysql => Arc::new(MysqlDatabase::new(config.database_url)),
            DatabaseType::InMemory => Arc::new(InMemoryDatabase::new()),
        };

        // Select cache based on config
        let cache: Arc<dyn Cache> = match config.cache_type {
            CacheType::Redis => Arc::new(RedisCache::new()),
            CacheType::InMemory => Arc::new(InMemoryCache::new()),
        };

        // Select logger based on config
        let logger: Arc<dyn Logger> = match config.log_level {
            LogLevel::Debug | LogLevel::Info => Arc::new(ConsoleLogger),
            LogLevel::Error => Arc::new(FileLogger::new("/var/log/errors.log")?),
        };

        // Create services
        let user_service = Arc::new(UserService::new(
            Arc::clone(&database),
            Arc::clone(&cache),
            Arc::clone(&logger),
        ));

        let order_service = Arc::new(OrderService::new(
            Arc::clone(&database),
            Arc::clone(&logger),
        ));

        Ok(Application {
            user_service,
            order_service,
        })
    }
}

pub struct MysqlDatabase {
    connection_string: String,
}

impl MysqlDatabase {
    pub fn new(connection_string: String) -> Self {
        Self { connection_string }
    }
}

impl Database for MysqlDatabase {
    fn query(&self, sql: &str) -> Result<Vec<String>, DbError> {
        println!("[MySQL] Query: {}", sql);
        Ok(vec![])
    }

    fn execute(&self, sql: &str) -> Result<u64, DbError> {
        println!("[MySQL] Execute: {}", sql);
        Ok(1)
    }
}

// Usage with config file
fn load_from_config() -> Result<(), Box<dyn std::error::Error>> {
    let config_str = r#"
        database_type = "postgres"
        database_url = "postgres://localhost/mydb"
        cache_type = "redis"
        log_level = "info"
    "#;

    let config: AppConfig = toml::from_str(config_str)?;
    let app = ConfigurableApplication::from_config(config)?;

    Ok(())
}
```

### Pattern: Middleware/Decorator with DI

**Concept:** Wrap services with additional behavior

```rust
use std::sync::Arc;
use std::time::Instant;

// Logging decorator
pub struct LoggingDatabase {
    inner: Arc<dyn Database>,
    logger: Arc<dyn Logger>,
}

impl LoggingDatabase {
    pub fn new(inner: Arc<dyn Database>, logger: Arc<dyn Logger>) -> Self {
        Self { inner, logger }
    }
}

impl Database for LoggingDatabase {
    fn query(&self, sql: &str) -> Result<Vec<String>, DbError> {
        self.logger.info(&format!("Executing query: {}", sql));
        let start = Instant::now();

        let result = self.inner.query(sql);

        let duration = start.elapsed();
        self.logger.info(&format!("Query completed in {:?}", duration));

        result
    }

    fn execute(&self, sql: &str) -> Result<u64, DbError> {
        self.logger.info(&format!("Executing command: {}", sql));
        let start = Instant::now();

        let result = self.inner.execute(sql);

        let duration = start.elapsed();
        self.logger.info(&format!("Command completed in {:?}", duration));

        result
    }
}

// Usage: Wrap database with logging
fn create_logged_service() -> UserService {
    let database = Arc::new(PostgresDatabase::new("postgres://localhost/db".to_string()));
    let logger = Arc::new(ConsoleLogger);

    // Wrap database with logging decorator
    let logged_database = Arc::new(LoggingDatabase::new(database, Arc::clone(&logger)));

    let cache = Arc::new(RedisCache::new());

    UserService::new(logged_database, cache, logger)
}
```

---

## 11. Anti-Patterns

### Anti-Pattern 1: Creating Dependencies Internally

**Problem:** Creates tight coupling, makes testing difficult

```rust
// ❌ BAD: Creates dependencies internally
pub struct UserService {
    database: PostgresDatabase,  // Concrete type
    cache: RedisCache,
}

impl UserService {
    pub fn new() -> Self {
        Self {
            database: PostgresDatabase::new("postgres://localhost/db".to_string()),  // ❌ Created internally
            cache: RedisCache::new(),  // ❌ Created internally
        }
    }
}

// ✅ GOOD: Dependencies injected
pub struct UserService {
    database: Arc<dyn Database>,  // Abstraction
    cache: Arc<dyn Cache>,
}

impl UserService {
    pub fn new(database: Arc<dyn Database>, cache: Arc<dyn Cache>) -> Self {
        Self { database, cache }  // ✅ Injected
    }
}
```

**Why it's bad:**
- Cannot test with mocks
- Cannot swap implementations
- Violates Dependency Inversion Principle
- Hard to configure

### Anti-Pattern 2: Using Concrete Types Instead of Traits

**Problem:** Defeats the purpose of DI

```rust
// ❌ BAD: Depends on concrete types
pub struct UserService {
    database: Arc<PostgresDatabase>,  // ❌ Concrete type
    cache: Arc<RedisCache>,  // ❌ Concrete type
}

// ✅ GOOD: Depends on abstractions
pub struct UserService {
    database: Arc<dyn Database>,  // ✅ Trait
    cache: Arc<dyn Cache>,  // ✅ Trait
}
```

**Why it's bad:**
- Cannot swap implementations
- Cannot mock for testing
- Tight coupling to specific implementations

### Anti-Pattern 3: Service Locator Pattern

**Problem:** Hidden dependencies, runtime errors

```rust
use std::sync::Mutex;
use std::collections::HashMap;

// ❌ BAD: Service Locator (anti-pattern)
pub struct ServiceLocator {
    services: Mutex<HashMap<String, Arc<dyn std::any::Any + Send + Sync>>>,
}

impl ServiceLocator {
    pub fn new() -> Self {
        Self {
            services: Mutex::new(HashMap::new()),
        }
    }

    pub fn register<T: std::any::Any + Send + Sync>(&self, name: &str, service: Arc<T>) {
        let any: Arc<dyn std::any::Any + Send + Sync> = service;
        self.services.lock().unwrap().insert(name.to_string(), any);
    }

    pub fn get<T: std::any::Any + Send + Sync>(&self, name: &str) -> Option<Arc<T>> {
        self.services
            .lock()
            .unwrap()
            .get(name)
            .and_then(|s| Arc::downcast(s.clone()).ok())
    }
}

// Usage
pub struct UserService {
    locator: Arc<ServiceLocator>,  // ❌ Hidden dependencies
}

impl UserService {
    pub fn new(locator: Arc<ServiceLocator>) -> Self {
        Self { locator }
    }

    pub fn get_user(&self, user_id: u64) -> Result<User, ServiceError> {
        // ❌ Dependencies not visible in constructor
        let database: Arc<dyn Database> = self.locator
            .get("database")
            .ok_or(ServiceError::NotFound)?;

        // Use database...
        todo!()
    }
}
```

**Why it's bad:**
- Dependencies hidden (not in constructor signature)
- Runtime errors if service not registered
- Hard to understand what service needs
- Hard to test
- Tight coupling to ServiceLocator

**Fix:** Use constructor injection instead

### Anti-Pattern 4: Passing DI Container Around

**Problem:** Too much access, unclear dependencies

```rust
// ❌ BAD: Passing entire container
pub struct DIContainer {
    database: Arc<dyn Database>,
    cache: Arc<dyn Cache>,
    logger: Arc<dyn Logger>,
    // ... many more
}

pub struct UserService {
    container: Arc<DIContainer>,  // ❌ Access to everything
}

impl UserService {
    pub fn new(container: Arc<DIContainer>) -> Self {
        Self { container }
    }

    pub fn get_user(&self, user_id: u64) -> Result<User, ServiceError> {
        // ❌ Unclear what dependencies are actually used
        self.container.database.query(&format!("SELECT * FROM users WHERE id = {}", user_id));
        todo!()
    }
}

// ✅ GOOD: Inject specific dependencies
pub struct UserService {
    database: Arc<dyn Database>,  // ✅ Clear dependencies
    cache: Arc<dyn Cache>,
}

impl UserService {
    pub fn new(database: Arc<dyn Database>, cache: Arc<dyn Cache>) -> Self {
        Self { database, cache }
    }
}
```

**Why it's bad:**
- Unclear what service actually needs
- Too much access (violates Principle of Least Privilege)
- Hard to test (need full container)
- Tight coupling to container

### Anti-Pattern 5: Over-Abstraction

**Problem:** Abstracting things that don't need abstraction

```rust
// ❌ BAD: Unnecessary abstraction
pub trait StringConcatenator: Send + Sync {
    fn concat(&self, a: &str, b: &str) -> String;
}

pub struct DefaultStringConcatenator;

impl StringConcatenator for DefaultStringConcatenator {
    fn concat(&self, a: &str, b: &str) -> String {
        format!("{}{}", a, b)  // ❌ No value added
    }
}

pub struct MessageBuilder {
    concatenator: Arc<dyn StringConcatenator>,  // ❌ Over-engineered
}

// ✅ GOOD: Use built-in functionality
pub struct MessageBuilder;

impl MessageBuilder {
    pub fn build(&self, prefix: &str, suffix: &str) -> String {
        format!("{}{}", prefix, suffix)  // ✅ Simple, direct
    }
}
```

**Why it's bad:**
- Unnecessary complexity
- No realistic alternative implementations
- Harder to maintain
- Over-engineering

**When to abstract:**
- Multiple realistic implementations exist
- Need to mock for testing
- External dependencies (I/O, network, etc.)

### Anti-Pattern 6: Temporal Coupling with Setters

**Problem:** Object can be in invalid state

```rust
// ❌ BAD: Setter injection creates temporal coupling
pub struct UserService {
    database: Option<Arc<dyn Database>>,
    cache: Option<Arc<dyn Cache>>,
}

impl UserService {
    pub fn new() -> Self {
        Self {
            database: None,  // ❌ Initially invalid
            cache: None,
        }
    }

    pub fn set_database(&mut self, database: Arc<dyn Database>) {
        self.database = Some(database);
    }

    pub fn set_cache(&mut self, cache: Arc<dyn Cache>) {
        self.cache = Some(cache);
    }

    pub fn get_user(&self, user_id: u64) -> Result<User, ServiceError> {
        // ❌ Must check if dependencies set
        let database = self.database.as_ref().ok_or(ServiceError::NotFound)?;
        // Use database...
        todo!()
    }
}

// Usage
let mut service = UserService::new();
service.set_database(db);  // Must remember to call
service.set_cache(cache);  // Must call in correct order
// If we forget to set_database, runtime error!

// ✅ GOOD: Constructor injection ensures validity
pub struct UserService {
    database: Arc<dyn Database>,
    cache: Arc<dyn Cache>,
}

impl UserService {
    pub fn new(database: Arc<dyn Database>, cache: Arc<dyn Cache>) -> Self {
        Self { database, cache }  // ✅ Always valid
    }
}

// Usage
let service = UserService::new(db, cache);  // ✅ Cannot forget dependencies
```

**Why it's bad:**
- Object can be in invalid state
- Order of setter calls matters (temporal coupling)
- Easy to forget to set required dependencies
- Runtime errors instead of compile-time errors

### Anti-Pattern 7: Mixing Concerns in Traits

**Problem:** Traits that do too much

```rust
// ❌ BAD: Fat trait with mixed concerns
pub trait DatabaseAndLogger: Send + Sync {
    fn query(&self, sql: &str) -> Result<Vec<String>, Error>;
    fn execute(&self, sql: &str) -> Result<u64, Error>;
    fn log_info(&self, message: &str);  // ❌ Mixed concern
    fn log_error(&self, message: &str);  // ❌ Mixed concern
}

// ✅ GOOD: Separate traits for separate concerns
pub trait Database: Send + Sync {
    fn query(&self, sql: &str) -> Result<Vec<String>, Error>;
    fn execute(&self, sql: &str) -> Result<u64, Error>;
}

pub trait Logger: Send + Sync {
    fn log_info(&self, message: &str);
    fn log_error(&self, message: &str);
}

pub struct UserService {
    database: Arc<dyn Database>,  // ✅ Single responsibility
    logger: Arc<dyn Logger>,  // ✅ Single responsibility
}
```

**Why it's bad:**
- Violates Single Responsibility Principle
- Hard to mock (must implement all methods)
- Implementations forced to handle unrelated concerns
- Reduces flexibility

### Anti-Pattern 8: Unnecessary `Arc` Clones

**Problem:** Over-using `Arc::clone`

```rust
// ❌ BAD: Excessive cloning
pub fn process_data(
    database: Arc<dyn Database>,
    cache: Arc<dyn Cache>,
    logger: Arc<dyn Logger>,
) -> Result<(), Error> {
    let db_clone1 = Arc::clone(&database);  // ❌ Unnecessary
    let db_clone2 = Arc::clone(&database);  // ❌ Unnecessary
    let db_clone3 = Arc::clone(&database);  // ❌ Unnecessary

    // Use clones...
    todo!()
}

// ✅ GOOD: Use references where possible
pub fn process_data(
    database: &dyn Database,  // ✅ Borrow instead
    cache: &dyn Cache,
    logger: &dyn Logger,
) -> Result<(), Error> {
    // Use directly, no cloning needed
    database.query("SELECT * FROM users")?;
    todo!()
}

// Only clone when moving to different thread or struct
pub fn spawn_processor(database: Arc<dyn Database>) {
    tokio::spawn(async move {
        // ✅ Clone needed to move into async task
        database.query("SELECT * FROM users").await;
    });
}
```

**Why it's bad:**
- Unnecessary reference counting overhead
- Clutters code
- May indicate poor ownership design

---

## 12. Conclusion

### Key Takeaways

1. **Use Traits for Abstractions**: Define traits for dependencies, not concrete types
2. **Constructor Injection First**: Prefer constructor injection for required dependencies
3. **Use `Arc<dyn Trait>`**: For runtime polymorphism and dependency injection
4. **Use Generics `<T: Trait>`**: For performance-critical code with compile-time types
5. **Builder Pattern**: For complex dependencies with optional parameters
6. **Factory Pattern**: For environment-specific configurations
7. **Test with Mocks**: Create mock implementations for testing
8. **Avoid Anti-Patterns**: Don't create dependencies internally, don't use service locator
9. **Composition Root**: Wire all dependencies at application entry point
10. **Keep It Simple**: Only abstract when you need multiple implementations

### Pattern Selection Guide

| Scenario | Recommended Pattern |
|----------|-------------------|
| 2-5 required dependencies | Constructor Injection |
| 5+ dependencies, some optional | Builder Pattern |
| Multiple configurations (prod, test, dev) | Factory Pattern |
| Performance-critical, known types | Generics `<T: Trait>` |
| Runtime polymorphism needed | Trait Objects `Arc<dyn Trait>` |
| Async services | `async_trait` with `Arc` |
| Request-scoped dependencies | Borrowed dependencies with lifetimes |
| Testing | Mock structs or `mockall` crate |

### Decision Tree

```
Do you need dependency injection?
├─ Yes
│  ├─ How many dependencies?
│  │  ├─ 2-5, all required → Constructor Injection
│  │  └─ 5+, some optional → Builder Pattern
│  │
│  ├─ Multiple environments?
│  │  └─ Yes → Factory Pattern
│  │
│  ├─ Performance-critical?
│  │  ├─ Yes → Generics <T: Trait>
│  │  └─ No → Trait Objects Arc<dyn Trait>
│  │
│  └─ Async code?
│     └─ Yes → async_trait + Arc
│
└─ No (simple utilities, pure functions)
   └─ Direct implementation, no DI needed
```

### Verification Checklist

Before completing dependency management implementation:

- [ ] All dependencies injected via constructor or builder (not created internally)
- [ ] Services depend on traits, not concrete types (`Arc<dyn Trait>`)
- [ ] Constructor injection used for required dependencies
- [ ] Builder pattern used if 5+ dependencies or validation needed
- [ ] Factory methods for environment-specific configurations
- [ ] Mock implementations created for all external dependencies
- [ ] Unit tests use mocks
- [ ] Integration tests use real implementations or test doubles
- [ ] No service locator pattern used
- [ ] No temporal coupling (dependencies set before use)
- [ ] Trait objects marked `Send + Sync` for thread safety
- [ ] All tests pass (`cargo test`)
- [ ] Code compiles without warnings (`cargo clippy`)

### Further Reading

#### Rust Resources
- **The Rust Book - Traits**: https://doc.rust-lang.org/book/ch10-02-traits.html
- **Rust by Example - Trait Objects**: https://doc.rust-lang.org/rust-by-example/trait/dyn.html
- **async-trait crate**: https://docs.rs/async-trait/

#### Design Patterns
- **Architecture Guide**: `.aiassisted/guidelines/architecture/dependency-management.md`
- **Builder Pattern**: `.aiassisted/guidelines/rust/rust-builder-pattern-guide.md`
- **Factory Pattern**: `.aiassisted/guidelines/rust/rust-factory-pattern-guide.md`

#### Crates
- **mockall** - Mocking framework: https://docs.rs/mockall/
- **tokio** - Async runtime: https://docs.rs/tokio/
- **serde** - Serialization: https://docs.rs/serde/

#### Articles
- Martin Fowler - Inversion of Control Containers
- Robert C. Martin - SOLID Principles
- Mark Seemann - Dependency Injection Principles

---

**End of Guide**

*Version 1.0 - 2026-01-25*
