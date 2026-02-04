# Rust Builder Pattern Implementation Guide

## Overview

This document provides practical guidelines for implementing the Builder Pattern in Rust. It focuses on Rust-specific idioms, type-safe patterns, and best practices for creating ergonomic builder APIs in the AirS Stack ecosystem.

**Assumed Knowledge:** Basic understanding of the Builder Pattern (see [Builder Pattern Architecture Guide](../../architecture/builder-pattern.md)) and intermediate Rust knowledge.

**Related Documents:**
- [Builder Pattern Architecture Guide](../../architecture/builder-pattern.md) - Language-agnostic pattern fundamentals
- [Microsoft Rust Guidelines](./microsoft-rust-guidelines.md) - Follow M-COMMON-TRAITS, M-PUBLIC-DEBUG, M-UPSTREAM-GUIDELINES
- [Rust ADT Implementation Guide](./rust-adt-implementation-guide.md) - For domain model patterns

---

## Table of Contents

1. [When to Use Builders in Rust](#when-to-use-builders-in-rust)
2. [Basic Builder Pattern](#basic-builder-pattern)
3. [Infallible vs Fallible Builders](#infallible-vs-fallible-builders)
4. [Type-State Builder Pattern](#type-state-builder-pattern)
5. [Builder with Into Conversions](#builder-with-into-conversions)
6. [Consuming Builders](#consuming-builders)
7. [Builder Derive Macros](#builder-derive-macros)
8. [Testing Builders](#testing-builders)
9. [Common Patterns](#common-patterns)
10. [Anti-Patterns](#anti-patterns)

---

## When to Use Builders in Rust

### ✅ Use Builders When:

1. **Many Optional Fields** (4+ parameters)
   ```rust
   // ❌ BAD: Confusing constructor
   let config = Config::new(
       Some("localhost"),
       Some(8080),
       None,
       Some(30),
       None,
       Some(true),
   );
   
   // ✅ GOOD: Self-documenting builder
   let config = Config::builder()
       .host("localhost")
       .port(8080)
       .timeout(30)
       .enable_ssl()
       .build();
   ```

2. **Complex Validation**
   ```rust
   // Validate relationships between fields before construction
   let user = User::builder()
       .username("john_doe")
       .email("john@example.com")
       .age(17)
       .build()?; // Error: age must be 18+ for email validation
   ```

3. **Immutable Objects**
   ```rust
   // Builder creates immutable structs with private fields
   pub struct DatabaseConnection {
       connection_string: String,
       pool_size: usize,
   }
   
   impl DatabaseConnection {
       pub fn builder() -> DatabaseConnectionBuilder {
           DatabaseConnectionBuilder::default()
       }
       // No setters - immutable after construction
   }
   ```

4. **Different Representations**
   ```rust
   // Same builder, different outputs
   let dev_config = Config::builder()
       .environment(Environment::Development)
       .build();
   
   let prod_config = Config::builder()
       .environment(Environment::Production)
       .build();
   ```

### ❌ Avoid Builders When:

1. **Simple Structs** (≤3 required fields, no optional fields)
   ```rust
   // ❌ Over-engineering
   let point = Point::builder().x(10).y(20).build();
   
   // ✅ Direct construction is clearer
   let point = Point { x: 10, y: 20 };
   ```

2. **All Fields Required with No Defaults**
   ```rust
   // ❌ Builder adds no value
   let rect = Rectangle::builder()
       .width(100)
       .height(200)
       .build();
   
   // ✅ Use constructor or struct literal
   let rect = Rectangle::new(100, 200);
   let rect = Rectangle { width: 100, height: 200 };
   ```

3. **Performance-Critical Hot Paths**
   - Builder adds allocation overhead
   - Use builders for configuration, not tight loops

---

## Basic Builder Pattern

### Pattern 1: Owned Builder (Most Common)

**Use Case:** Standard builder for types with optional fields and validation.

```rust
use std::fmt;

/// HTTP request configuration.
#[derive(Debug, Clone)]
pub struct HttpRequest {
    method: String,
    url: String,
    headers: Vec<(String, String)>,
    body: Option<String>,
    timeout_ms: u64,
}

impl HttpRequest {
    /// Creates a new builder for `HttpRequest`.
    pub fn builder(url: impl Into<String>) -> HttpRequestBuilder {
        HttpRequestBuilder::new(url)
    }
    
    pub fn method(&self) -> &str {
        &self.method
    }
    
    pub fn url(&self) -> &str {
        &self.url
    }
    
    pub fn headers(&self) -> &[(String, String)] {
        &self.headers
    }
    
    pub fn body(&self) -> Option<&str> {
        self.body.as_deref()
    }
    
    pub fn timeout_ms(&self) -> u64 {
        self.timeout_ms
    }
}

/// Builder for [`HttpRequest`].
#[derive(Debug, Clone)]
pub struct HttpRequestBuilder {
    method: String,
    url: String,
    headers: Vec<(String, String)>,
    body: Option<String>,
    timeout_ms: u64,
}

impl HttpRequestBuilder {
    /// Creates a new builder with required URL.
    fn new(url: impl Into<String>) -> Self {
        Self {
            method: "GET".to_string(), // Default
            url: url.into(),
            headers: Vec::new(),
            body: None,
            timeout_ms: 30_000, // Default: 30 seconds
        }
    }
    
    /// Sets the HTTP method (default: "GET").
    pub fn method(mut self, method: impl Into<String>) -> Self {
        self.method = method.into();
        self
    }
    
    /// Adds a header to the request.
    pub fn header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.push((key.into(), value.into()));
        self
    }
    
    /// Sets the request body.
    pub fn body(mut self, body: impl Into<String>) -> Self {
        self.body = Some(body.into());
        self
    }
    
    /// Sets the timeout in milliseconds (default: 30000).
    pub fn timeout_ms(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = timeout_ms;
        self
    }
    
    /// Builds the [`HttpRequest`].
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration is invalid.
    pub fn build(self) -> Result<HttpRequest, BuildError> {
        // Validate URL
        if self.url.is_empty() {
            return Err(BuildError::EmptyUrl);
        }
        
        // Validate method
        let method = self.method.to_uppercase();
        if !["GET", "POST", "PUT", "DELETE", "PATCH", "HEAD", "OPTIONS"]
            .contains(&method.as_str())
        {
            return Err(BuildError::InvalidMethod(self.method));
        }
        
        // Validate body (only allowed for certain methods)
        if self.body.is_some() && ["GET", "HEAD"].contains(&method.as_str()) {
            return Err(BuildError::BodyNotAllowed(method));
        }
        
        Ok(HttpRequest {
            method,
            url: self.url,
            headers: self.headers,
            body: self.body,
            timeout_ms: self.timeout_ms,
        })
    }
}

/// Error type for [`HttpRequestBuilder`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BuildError {
    EmptyUrl,
    InvalidMethod(String),
    BodyNotAllowed(String),
}

impl fmt::Display for BuildError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyUrl => write!(f, "URL cannot be empty"),
            Self::InvalidMethod(method) => write!(f, "invalid HTTP method: {}", method),
            Self::BodyNotAllowed(method) => {
                write!(f, "{} requests cannot have a body", method)
            }
        }
    }
}

impl std::error::Error for BuildError {}

// Usage example:
fn example() -> Result<(), BuildError> {
    let request = HttpRequest::builder("https://api.example.com/users")
        .method("POST")
        .header("Content-Type", "application/json")
        .header("Authorization", "Bearer token123")
        .body(r#"{"name": "John Doe"}"#)
        .timeout_ms(10_000)
        .build()?;
    
    println!("Request: {} {}", request.method(), request.url());
    Ok(())
}
```

**Key Points:**
- Builder takes ownership with `mut self` → enables method chaining
- Required fields in builder constructor (`url`)
- Optional fields have defaults
- Validation happens in `build()`
- Returns `Result<T, BuildError>` for fallible construction

---

## Infallible vs Fallible Builders

### Infallible Builder (No Validation)

**Use Case:** When all field combinations are valid.

```rust
/// Query parameters for an API request.
#[derive(Debug, Clone, Default)]
pub struct QueryParams {
    limit: Option<usize>,
    offset: Option<usize>,
    sort_by: Option<String>,
    order: Option<String>,
}

impl QueryParams {
    pub fn builder() -> QueryParamsBuilder {
        QueryParamsBuilder::default()
    }
}

/// Builder for [`QueryParams`].
#[derive(Debug, Clone, Default)]
pub struct QueryParamsBuilder {
    limit: Option<usize>,
    offset: Option<usize>,
    sort_by: Option<String>,
    order: Option<String>,
}

impl QueryParamsBuilder {
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }
    
    pub fn offset(mut self, offset: usize) -> Self {
        self.offset = Some(offset);
        self
    }
    
    pub fn sort_by(mut self, field: impl Into<String>) -> Self {
        self.sort_by = Some(field.into());
        self
    }
    
    pub fn order(mut self, order: impl Into<String>) -> Self {
        self.order = Some(order.into());
        self
    }
    
    /// Builds the [`QueryParams`].
    ///
    /// This is infallible - all field combinations are valid.
    pub fn build(self) -> QueryParams {
        QueryParams {
            limit: self.limit,
            offset: self.offset,
            sort_by: self.sort_by,
            order: self.order,
        }
    }
}

// Usage:
fn example() {
    let params = QueryParams::builder()
        .limit(10)
        .offset(20)
        .sort_by("created_at")
        .order("desc")
        .build(); // No Result - always succeeds
}
```

### Fallible Builder (With Validation)

**Use Case:** When fields have constraints or relationships.

```rust
use thiserror::Error;

/// User registration data.
#[derive(Debug, Clone)]
pub struct UserRegistration {
    username: String,
    email: String,
    password: String,
    age: u8,
}

impl UserRegistration {
    pub fn builder() -> UserRegistrationBuilder {
        UserRegistrationBuilder::default()
    }
}

/// Builder for [`UserRegistration`].
#[derive(Debug, Clone, Default)]
pub struct UserRegistrationBuilder {
    username: Option<String>,
    email: Option<String>,
    password: Option<String>,
    age: Option<u8>,
}

impl UserRegistrationBuilder {
    pub fn username(mut self, username: impl Into<String>) -> Self {
        self.username = Some(username.into());
        self
    }
    
    pub fn email(mut self, email: impl Into<String>) -> Self {
        self.email = Some(email.into());
        self
    }
    
    pub fn password(mut self, password: impl Into<String>) -> Self {
        self.password = Some(password.into());
        self
    }
    
    pub fn age(mut self, age: u8) -> Self {
        self.age = Some(age);
        self
    }
    
    /// Builds the [`UserRegistration`].
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Any required field is missing
    /// - Username is invalid (length, characters)
    /// - Email is invalid format
    /// - Password is too weak
    /// - Age is below minimum
    pub fn build(self) -> Result<UserRegistration, ValidationError> {
        // Check required fields
        let username = self.username.ok_or(ValidationError::MissingUsername)?;
        let email = self.email.ok_or(ValidationError::MissingEmail)?;
        let password = self.password.ok_or(ValidationError::MissingPassword)?;
        let age = self.age.ok_or(ValidationError::MissingAge)?;
        
        // Validate username
        if username.len() < 3 {
            return Err(ValidationError::UsernameTooShort);
        }
        if username.len() > 20 {
            return Err(ValidationError::UsernameTooLong);
        }
        if !username.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err(ValidationError::UsernameInvalidChars);
        }
        
        // Validate email
        if !email.contains('@') {
            return Err(ValidationError::InvalidEmail);
        }
        
        // Validate password
        if password.len() < 8 {
            return Err(ValidationError::PasswordTooShort);
        }
        if !password.chars().any(|c| c.is_uppercase()) {
            return Err(ValidationError::PasswordNeedsUppercase);
        }
        if !password.chars().any(|c| c.is_numeric()) {
            return Err(ValidationError::PasswordNeedsNumber);
        }
        
        // Validate age
        if age < 18 {
            return Err(ValidationError::AgeTooYoung);
        }
        
        Ok(UserRegistration {
            username,
            email,
            password,
            age,
        })
    }
}

/// Validation errors for [`UserRegistrationBuilder`].
#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum ValidationError {
    #[error("username is required")]
    MissingUsername,
    #[error("email is required")]
    MissingEmail,
    #[error("password is required")]
    MissingPassword,
    #[error("age is required")]
    MissingAge,
    #[error("username must be at least 3 characters")]
    UsernameTooShort,
    #[error("username must be at most 20 characters")]
    UsernameTooLong,
    #[error("username can only contain letters, numbers, and underscores")]
    UsernameInvalidChars,
    #[error("invalid email format")]
    InvalidEmail,
    #[error("password must be at least 8 characters")]
    PasswordTooShort,
    #[error("password must contain at least one uppercase letter")]
    PasswordNeedsUppercase,
    #[error("password must contain at least one number")]
    PasswordNeedsNumber,
    #[error("must be at least 18 years old")]
    AgeTooYoung,
}

// Usage:
fn example() -> Result<(), ValidationError> {
    let registration = UserRegistration::builder()
        .username("john_doe")
        .email("john@example.com")
        .password("SecurePass123")
        .age(25)
        .build()?;
    
    Ok(())
}
```

---

## Type-State Builder Pattern

**Use Case:** Enforce construction order at compile time using the type system.

### Basic Type-State Builder

```rust
use std::marker::PhantomData;

/// Database connection configuration.
#[derive(Debug)]
pub struct DatabaseConnection {
    host: String,
    port: u16,
    database: String,
    username: String,
    password: String,
}

impl DatabaseConnection {
    /// Starts building a connection configuration.
    pub fn builder() -> DatabaseConnectionBuilder<NoHost> {
        DatabaseConnectionBuilder {
            host: None,
            port: 5432, // Default PostgreSQL port
            database: None,
            username: None,
            password: None,
            _state: PhantomData,
        }
    }
}

// Type-state markers
pub struct NoHost;
pub struct HasHost;
pub struct NoDatabase;
pub struct HasDatabase;
pub struct NoCredentials;
pub struct HasCredentials;

/// Type-state builder for [`DatabaseConnection`].
pub struct DatabaseConnectionBuilder<State> {
    host: Option<String>,
    port: u16,
    database: Option<String>,
    username: Option<String>,
    password: Option<String>,
    _state: PhantomData<State>,
}

impl DatabaseConnectionBuilder<NoHost> {
    /// Sets the database host (required).
    pub fn host(self, host: impl Into<String>) -> DatabaseConnectionBuilder<HasHost> {
        DatabaseConnectionBuilder {
            host: Some(host.into()),
            port: self.port,
            database: self.database,
            username: self.username,
            password: self.password,
            _state: PhantomData,
        }
    }
}

impl DatabaseConnectionBuilder<HasHost> {
    /// Sets the database port (optional, defaults to 5432).
    pub fn port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }
    
    /// Sets the database name (required).
    pub fn database(self, database: impl Into<String>) -> DatabaseConnectionBuilder<HasDatabase> {
        DatabaseConnectionBuilder {
            host: self.host,
            port: self.port,
            database: Some(database.into()),
            username: self.username,
            password: self.password,
            _state: PhantomData,
        }
    }
}

impl DatabaseConnectionBuilder<HasDatabase> {
    /// Sets the authentication credentials (required).
    pub fn credentials(
        self,
        username: impl Into<String>,
        password: impl Into<String>,
    ) -> DatabaseConnectionBuilder<HasCredentials> {
        DatabaseConnectionBuilder {
            host: self.host,
            port: self.port,
            database: self.database,
            username: Some(username.into()),
            password: Some(password.into()),
            _state: PhantomData,
        }
    }
}

impl DatabaseConnectionBuilder<HasCredentials> {
    /// Builds the database connection configuration.
    ///
    /// This method is only available after all required fields are set.
    pub fn build(self) -> DatabaseConnection {
        DatabaseConnection {
            host: self.host.unwrap(), // Safe: type system guarantees these exist
            port: self.port,
            database: self.database.unwrap(),
            username: self.username.unwrap(),
            password: self.password.unwrap(),
        }
    }
}

// Usage:
fn example() {
    let connection = DatabaseConnection::builder()
        .host("localhost")
        .port(5432)
        .database("myapp")
        .credentials("admin", "secret")
        .build();
    
    // ❌ This won't compile - type system enforces order:
    // let connection = DatabaseConnection::builder()
    //     .database("myapp")  // ERROR: can't set database before host
    //     .build();           // ERROR: can't build without all required fields
}
```

**Advantages:**
- Compile-time enforcement of required fields
- Impossible to create invalid objects
- Clear API - IDE autocomplete guides you

**Disadvantages:**
- More boilerplate code
- Inflexible order (sometimes undesirable)
- Complex type signatures

### Simplified Type-State (Single State Parameter)

```rust
/// Simplified type-state builder with single state parameter.
pub struct ConfigBuilder<State = Incomplete> {
    host: Option<String>,
    port: Option<u16>,
    _state: PhantomData<State>,
}

pub struct Incomplete;
pub struct Complete;

impl ConfigBuilder<Incomplete> {
    pub fn new() -> Self {
        Self {
            host: None,
            port: None,
            _state: PhantomData,
        }
    }
    
    pub fn host(mut self, host: impl Into<String>) -> Self {
        self.host = Some(host.into());
        self
    }
    
    pub fn port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }
    
    /// Transitions to `Complete` state if all required fields are set.
    pub fn complete(self) -> Result<ConfigBuilder<Complete>, &'static str> {
        if self.host.is_none() {
            return Err("host is required");
        }
        if self.port.is_none() {
            return Err("port is required");
        }
        
        Ok(ConfigBuilder {
            host: self.host,
            port: self.port,
            _state: PhantomData,
        })
    }
}

impl ConfigBuilder<Complete> {
    pub fn build(self) -> Config {
        Config {
            host: self.host.unwrap(),
            port: self.port.unwrap(),
        }
    }
}

#[derive(Debug)]
pub struct Config {
    host: String,
    port: u16,
}
```

---

## Builder with Into Conversions

**Use Case:** Accept any type that can be converted into the target type.

```rust
use std::path::{Path, PathBuf};

/// File configuration.
#[derive(Debug, Clone)]
pub struct FileConfig {
    path: PathBuf,
    encoding: String,
    buffer_size: usize,
}

impl FileConfig {
    pub fn builder(path: impl Into<PathBuf>) -> FileConfigBuilder {
        FileConfigBuilder {
            path: path.into(),
            encoding: "utf-8".to_string(),
            buffer_size: 8192,
        }
    }
}

/// Builder for [`FileConfig`].
#[derive(Debug, Clone)]
pub struct FileConfigBuilder {
    path: PathBuf,
    encoding: String,
    buffer_size: usize,
}

impl FileConfigBuilder {
    /// Sets the file encoding.
    ///
    /// Accepts any type that can be converted into a `String`:
    /// - `&str`
    /// - `String`
    /// - `Cow<str>`
    pub fn encoding(mut self, encoding: impl Into<String>) -> Self {
        self.encoding = encoding.into();
        self
    }
    
    /// Sets the buffer size in bytes.
    pub fn buffer_size(mut self, size: usize) -> Self {
        self.buffer_size = size;
        self
    }
    
    pub fn build(self) -> FileConfig {
        FileConfig {
            path: self.path,
            encoding: self.encoding,
            buffer_size: self.buffer_size,
        }
    }
}

// Usage - accepts multiple types:
fn example() {
    // From &str
    let config1 = FileConfig::builder("/tmp/file.txt")
        .encoding("utf-8")
        .build();
    
    // From String
    let path = String::from("/tmp/file.txt");
    let config2 = FileConfig::builder(path)
        .encoding("latin1".to_string())
        .build();
    
    // From PathBuf
    let path = PathBuf::from("/tmp/file.txt");
    let config3 = FileConfig::builder(path)
        .build();
    
    // From &Path
    let path = Path::new("/tmp/file.txt");
    let config4 = FileConfig::builder(path)
        .build();
}
```

**Key Points:**
- Use `impl Into<T>` for ergonomic APIs
- Caller can pass `&str`, `String`, `Cow<str>`, etc.
- Call `.into()` once in the builder method

---

## Consuming Builders

### Pattern 1: Consuming Builder (Move Semantics)

Most common pattern - builder is consumed on each method call.

```rust
/// Consuming builder - takes `self` by value.
pub struct OrderBuilder {
    items: Vec<String>,
    total: f64,
}

impl OrderBuilder {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            total: 0.0,
        }
    }
    
    /// Adds an item to the order.
    ///
    /// Takes `self` by value, consumes the builder.
    pub fn add_item(mut self, item: String, price: f64) -> Self {
        self.items.push(item);
        self.total += price;
        self // Return self for chaining
    }
    
    pub fn build(self) -> Order {
        Order {
            items: self.items,
            total: self.total,
        }
    }
}

#[derive(Debug)]
pub struct Order {
    items: Vec<String>,
    total: f64,
}

// Usage:
fn example() {
    let order = OrderBuilder::new()
        .add_item("Widget".to_string(), 9.99)
        .add_item("Gadget".to_string(), 19.99)
        .build();
    
    // ❌ Can't reuse builder after build():
    // let order2 = builder.build(); // ERROR: builder was moved
}
```

### Pattern 2: Non-Consuming Builder (Borrowing)

**Use Case:** When you need to reuse the builder or inspect it mid-construction.

```rust
/// Non-consuming builder - takes `&mut self`.
pub struct QueryBuilder {
    select: Vec<String>,
    from: Option<String>,
    where_clauses: Vec<String>,
}

impl QueryBuilder {
    pub fn new() -> Self {
        Self {
            select: Vec::new(),
            from: None,
            where_clauses: Vec::new(),
        }
    }
    
    /// Adds a field to SELECT clause.
    ///
    /// Takes `&mut self`, allows reuse.
    pub fn select(&mut self, field: &str) -> &mut Self {
        self.select.push(field.to_string());
        self
    }
    
    pub fn from(&mut self, table: &str) -> &mut Self {
        self.from = Some(table.to_string());
        self
    }
    
    pub fn where_clause(&mut self, clause: &str) -> &mut Self {
        self.where_clauses.push(clause.to_string());
        self
    }
    
    /// Builds the SQL query string.
    pub fn build(&self) -> String {
        let select = self.select.join(", ");
        let from = self.from.as_deref().unwrap_or("");
        let where_part = if self.where_clauses.is_empty() {
            String::new()
        } else {
            format!(" WHERE {}", self.where_clauses.join(" AND "))
        };
        
        format!("SELECT {} FROM {}{}", select, from, where_part)
    }
    
    /// Resets the builder for reuse.
    pub fn reset(&mut self) {
        self.select.clear();
        self.from = None;
        self.where_clauses.clear();
    }
}

// Usage:
fn example() {
    let mut builder = QueryBuilder::new();
    
    // Build first query
    let query1 = builder
        .select("id")
        .select("name")
        .from("users")
        .where_clause("active = true")
        .build();
    
    println!("Query 1: {}", query1);
    
    // Reuse builder for second query
    builder.reset();
    let query2 = builder
        .select("*")
        .from("orders")
        .build();
    
    println!("Query 2: {}", query2);
}
```

**Trade-offs:**

| Consuming (`self`) | Borrowing (`&mut self`) |
|-------------------|------------------------|
| ✅ Move semantics prevent misuse | ✅ Can reuse builder |
| ✅ Clear ownership | ✅ Can inspect mid-build |
| ✅ More idiomatic Rust | ❌ Mutable reference complexity |
| ❌ Cannot reuse builder | ❌ Easier to misuse |

**Recommendation:** Prefer consuming builders unless you have a specific need for reuse.

---

## Builder Derive Macros

### Using `derive_builder` Crate

For simple cases, use the [`derive_builder`](https://docs.rs/derive_builder/) crate to auto-generate builders.

**Add to `Cargo.toml`:**
```toml
[dependencies]
derive_builder = "0.20"
```

**Example:**

```rust
use derive_builder::Builder;

/// User configuration with auto-generated builder.
#[derive(Debug, Clone, Builder)]
#[builder(setter(into))]
pub struct User {
    /// Username (required).
    #[builder(setter(into))]
    username: String,
    
    /// Email address (required).
    #[builder(setter(into))]
    email: String,
    
    /// First name (optional).
    #[builder(setter(into, strip_option), default)]
    first_name: Option<String>,
    
    /// Last name (optional).
    #[builder(setter(into, strip_option), default)]
    last_name: Option<String>,
    
    /// Age (optional).
    #[builder(default)]
    age: Option<u8>,
    
    /// Account is active (default: true).
    #[builder(default = "true")]
    active: bool,
}

// Usage:
fn example() -> Result<(), derive_builder::UninitializedFieldError> {
    let user = UserBuilder::default()
        .username("john_doe")
        .email("john@example.com")
        .first_name("John")
        .last_name("Doe")
        .age(30)
        .build()?;
    
    println!("{:?}", user);
    Ok(())
}
```

**Key Attributes:**
- `#[builder(setter(into))]` - Accept types via `Into` conversion
- `#[builder(setter(strip_option))]` - For `Option<T>` fields, setter accepts `T`
- `#[builder(default)]` - Use `Default::default()` if not set
- `#[builder(default = "expr")]` - Custom default value

### Custom Validation with `derive_builder`

```rust
use derive_builder::Builder;

#[derive(Debug, Builder)]
#[builder(build_fn(validate = "Self::validate"))]
pub struct DatabaseConfig {
    host: String,
    port: u16,
    username: String,
    password: String,
}

impl DatabaseConfigBuilder {
    /// Custom validation before build.
    fn validate(&self) -> Result<(), String> {
        if let Some(port) = self.port {
            if port == 0 {
                return Err("port cannot be 0".to_string());
            }
        }
        
        if let Some(password) = &self.password {
            if password.len() < 8 {
                return Err("password must be at least 8 characters".to_string());
            }
        }
        
        Ok(())
    }
}
```

**When to Use `derive_builder`:**
- ✅ Simple structs with mostly optional fields
- ✅ Minimal custom validation
- ✅ Prototyping or internal APIs

**When to Write Manual Builders:**
- ❌ Complex validation logic
- ❌ Type-state patterns
- ❌ Custom builder methods beyond simple setters
- ❌ Public APIs where you need full control

---

## Testing Builders

### Unit Tests for Builders

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_http_request_builder_default_values() {
        let request = HttpRequest::builder("https://example.com")
            .build()
            .expect("should build with defaults");
        
        assert_eq!(request.method(), "GET");
        assert_eq!(request.url(), "https://example.com");
        assert_eq!(request.timeout_ms(), 30_000);
        assert!(request.headers().is_empty());
        assert!(request.body().is_none());
    }
    
    #[test]
    fn test_http_request_builder_custom_values() {
        let request = HttpRequest::builder("https://api.example.com")
            .method("POST")
            .header("Content-Type", "application/json")
            .body(r#"{"key": "value"}"#)
            .timeout_ms(5_000)
            .build()
            .expect("should build");
        
        assert_eq!(request.method(), "POST");
        assert_eq!(request.body(), Some(r#"{"key": "value"}"#));
        assert_eq!(request.timeout_ms(), 5_000);
    }
    
    #[test]
    fn test_http_request_builder_empty_url() {
        let result = HttpRequest::builder("")
            .build();
        
        assert_eq!(result, Err(BuildError::EmptyUrl));
    }
    
    #[test]
    fn test_http_request_builder_invalid_method() {
        let result = HttpRequest::builder("https://example.com")
            .method("INVALID")
            .build();
        
        assert_eq!(result, Err(BuildError::InvalidMethod("INVALID".to_string())));
    }
    
    #[test]
    fn test_http_request_builder_get_with_body() {
        let result = HttpRequest::builder("https://example.com")
            .method("GET")
            .body("some body")
            .build();
        
        assert_eq!(result, Err(BuildError::BodyNotAllowed("GET".to_string())));
    }
    
    #[test]
    fn test_user_registration_builder_valid() {
        let result = UserRegistration::builder()
            .username("john_doe")
            .email("john@example.com")
            .password("SecurePass123")
            .age(25)
            .build();
        
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_user_registration_builder_missing_field() {
        let result = UserRegistration::builder()
            .username("john_doe")
            .email("john@example.com")
            .password("SecurePass123")
            // Missing age
            .build();
        
        assert_eq!(result, Err(ValidationError::MissingAge));
    }
    
    #[test]
    fn test_user_registration_builder_username_too_short() {
        let result = UserRegistration::builder()
            .username("ab") // Only 2 characters
            .email("john@example.com")
            .password("SecurePass123")
            .age(25)
            .build();
        
        assert_eq!(result, Err(ValidationError::UsernameTooShort));
    }
    
    #[test]
    fn test_user_registration_builder_password_weak() {
        let result = UserRegistration::builder()
            .username("john_doe")
            .email("john@example.com")
            .password("weak") // No uppercase, no number, too short
            .age(25)
            .build();
        
        assert_eq!(result, Err(ValidationError::PasswordTooShort));
    }
}
```

### Property-Based Testing with `proptest`

```rust
#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;
    
    proptest! {
        #[test]
        fn test_valid_usernames_always_build(
            username in "[a-zA-Z0-9_]{3,20}",
            age in 18u8..100u8,
        ) {
            let result = UserRegistration::builder()
                .username(username)
                .email("test@example.com")
                .password("ValidPass123")
                .age(age)
                .build();
            
            prop_assert!(result.is_ok());
        }
        
        #[test]
        fn test_short_usernames_always_fail(
            username in "[a-zA-Z]{1,2}",
        ) {
            let result = UserRegistration::builder()
                .username(username)
                .email("test@example.com")
                .password("ValidPass123")
                .age(25)
                .build();
            
            prop_assert_eq!(result, Err(ValidationError::UsernameTooShort));
        }
        
        #[test]
        fn test_timeout_always_preserved(
            timeout in 1u64..=60_000u64,
        ) {
            let request = HttpRequest::builder("https://example.com")
                .timeout_ms(timeout)
                .build()
                .unwrap();
            
            prop_assert_eq!(request.timeout_ms(), timeout);
        }
    }
}
```

### Builder Test Utilities

```rust
#[cfg(test)]
pub mod test_utils {
    use super::*;
    
    /// Creates a valid test user registration.
    pub fn valid_user_registration() -> UserRegistrationBuilder {
        UserRegistration::builder()
            .username("test_user")
            .email("test@example.com")
            .password("TestPass123")
            .age(25)
    }
    
    /// Creates a valid test HTTP request.
    pub fn valid_http_request() -> HttpRequestBuilder {
        HttpRequest::builder("https://api.example.com/test")
            .method("GET")
            .timeout_ms(30_000)
    }
}

#[cfg(test)]
mod integration_tests {
    use super::test_utils::*;
    
    #[test]
    fn test_user_registration_integration() {
        let user = valid_user_registration()
            .username("custom_user")
            .build()
            .expect("should build valid user");
        
        // Use user in integration test...
    }
}
```

---

## Common Patterns

### Pattern 1: Builder with Presets

**Use Case:** Provide common configurations as starting points.

```rust
impl ServerConfigBuilder {
    /// Creates a development configuration preset.
    pub fn development() -> Self {
        Self::new()
            .host("localhost")
            .port(3000)
            .enable_hot_reload(true)
            .log_level("debug")
    }
    
    /// Creates a production configuration preset.
    pub fn production() -> Self {
        Self::new()
            .host("0.0.0.0")
            .port(8080)
            .enable_hot_reload(false)
            .log_level("info")
            .enable_compression(true)
    }
}

// Usage:
let config = ServerConfigBuilder::production()
    .port(9000) // Override preset value
    .build()?;
```

### Pattern 2: Builder with Method Aliases

**Use Case:** Provide ergonomic aliases for common operations.

```rust
impl HttpRequestBuilder {
    /// Alias for `method("GET")`.
    pub fn get(self) -> Self {
        self.method("GET")
    }
    
    /// Alias for `method("POST")`.
    pub fn post(self) -> Self {
        self.method("POST")
    }
    
    /// Alias for `method("PUT")`.
    pub fn put(self) -> Self {
        self.method("PUT")
    }
    
    /// Alias for `method("DELETE")`.
    pub fn delete(self) -> Self {
        self.method("DELETE")
    }
}

// Usage:
let request = HttpRequest::builder("https://api.example.com/users")
    .post() // More readable than .method("POST")
    .body(r#"{"name": "John"}"#)
    .build()?;
```

### Pattern 3: Builder with Conditional Logic

**Use Case:** Apply settings conditionally based on environment.

```rust
impl ConfigBuilder {
    /// Applies settings based on environment.
    pub fn from_env(mut self) -> Self {
        if let Ok(host) = std::env::var("APP_HOST") {
            self = self.host(host);
        }
        if let Ok(port) = std::env::var("APP_PORT") {
            if let Ok(port) = port.parse() {
                self = self.port(port);
            }
        }
        self
    }
    
    /// Applies settings if condition is true.
    pub fn when(mut self, condition: bool, f: impl FnOnce(Self) -> Self) -> Self {
        if condition {
            f(self)
        } else {
            self
        }
    }
}

// Usage:
let config = ConfigBuilder::new()
    .from_env()
    .when(cfg!(debug_assertions), |b| {
        b.enable_debug_logging()
    })
    .build();
```

### Pattern 4: Builder with Collection Methods

**Use Case:** Ergonomic methods for adding multiple items.

```rust
impl OrderBuilder {
    /// Adds a single item.
    pub fn add_item(mut self, item: LineItem) -> Self {
        self.items.push(item);
        self
    }
    
    /// Adds multiple items.
    pub fn add_items(mut self, items: impl IntoIterator<Item = LineItem>) -> Self {
        self.items.extend(items);
        self
    }
    
    /// Adds items from iterator.
    pub fn extend_items(mut self, items: impl IntoIterator<Item = LineItem>) -> Self {
        self.items.extend(items);
        self
    }
}

// Usage:
let order = OrderBuilder::new()
    .add_item(item1)
    .add_items(vec![item2, item3, item4])
    .build();
```

### Pattern 5: Builder with Try Methods

**Use Case:** Fallible operations during building.

```rust
impl ConfigBuilder {
    /// Tries to parse and set port from string.
    pub fn try_port(mut self, port_str: &str) -> Result<Self, ParseIntError> {
        let port = port_str.parse::<u16>()?;
        self.port = Some(port);
        Ok(self)
    }
    
    /// Tries to read config from file.
    pub fn try_from_file(mut self, path: impl AsRef<Path>) -> std::io::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        // Parse content and update builder
        Ok(self)
    }
}

// Usage:
let config = ConfigBuilder::new()
    .host("localhost")
    .try_port("8080")?
    .build();
```

---

## Anti-Patterns

### ❌ Anti-Pattern 1: Builder for Simple Structs

```rust
// ❌ BAD: Over-engineering a simple type
#[derive(Debug)]
pub struct Point {
    x: i32,
    y: i32,
}

pub struct PointBuilder {
    x: Option<i32>,
    y: Option<i32>,
}

impl PointBuilder {
    pub fn new() -> Self {
        Self { x: None, y: None }
    }
    
    pub fn x(mut self, x: i32) -> Self {
        self.x = Some(x);
        self
    }
    
    pub fn y(mut self, y: i32) -> Self {
        self.y = Some(y);
        self
    }
    
    pub fn build(self) -> Result<Point, &'static str> {
        Ok(Point {
            x: self.x.ok_or("x is required")?,
            y: self.y.ok_or("y is required")?,
        })
    }
}

// ✅ GOOD: Simple struct with direct construction
#[derive(Debug)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

// Usage: much clearer
let point = Point::new(10, 20);
let point = Point { x: 10, y: 20 };
```

### ❌ Anti-Pattern 2: Mixing Mutability with Builders

```rust
// ❌ BAD: Product has setters, defeating immutability
pub struct User {
    username: String,
    email: String,
}

impl User {
    pub fn builder() -> UserBuilder {
        UserBuilder::default()
    }
    
    // Setter defeats builder pattern benefits
    pub fn set_email(&mut self, email: String) {
        self.email = email;
    }
}

// ✅ GOOD: Immutable product
pub struct User {
    username: String,
    email: String,
}

impl User {
    pub fn builder() -> UserBuilder {
        UserBuilder::default()
    }
    
    // Only getters, no setters
    pub fn email(&self) -> &str {
        &self.email
    }
}
```

### ❌ Anti-Pattern 3: No Validation in build()

```rust
// ❌ BAD: No validation, can create invalid objects
impl EmailBuilder {
    pub fn build(self) -> Email {
        Email {
            address: self.address.unwrap_or_default(),
        }
    }
}

// ✅ GOOD: Validate before construction
impl EmailBuilder {
    pub fn build(self) -> Result<Email, ValidationError> {
        let address = self.address.ok_or(ValidationError::MissingAddress)?;
        
        if !address.contains('@') {
            return Err(ValidationError::InvalidEmailFormat);
        }
        
        Ok(Email { address })
    }
}
```

### ❌ Anti-Pattern 4: Exposing Builder Internal State

```rust
// ❌ BAD: Public fields allow bypassing builder validation
pub struct ConfigBuilder {
    pub host: Option<String>,  // Public!
    pub port: Option<u16>,     // Public!
}

impl ConfigBuilder {
    pub fn build(self) -> Result<Config, Error> {
        // Validation can be bypassed by setting fields directly
        // ...
    }
}

// Usage: bypasses validation
let mut builder = ConfigBuilder::new();
builder.host = Some("".to_string()); // Invalid, but allowed!
let config = builder.build()?;

// ✅ GOOD: Private fields enforce using builder methods
pub struct ConfigBuilder {
    host: Option<String>,  // Private
    port: Option<u16>,     // Private
}

impl ConfigBuilder {
    pub fn host(mut self, host: impl Into<String>) -> Self {
        self.host = Some(host.into());
        self
    }
    
    pub fn build(self) -> Result<Config, Error> {
        // Validation cannot be bypassed
        // ...
    }
}
```

### ❌ Anti-Pattern 5: Forgetting to Return `self`

```rust
// ❌ BAD: Cannot chain methods
impl ConfigBuilder {
    pub fn host(mut self, host: String) {
        self.host = Some(host);
        // No return!
    }
}

// Cannot chain:
let builder = ConfigBuilder::new();
builder.host("localhost".to_string()); // Returns ()
// builder.port(8080); // ERROR: builder was moved

// ✅ GOOD: Return self for chaining
impl ConfigBuilder {
    pub fn host(mut self, host: String) -> Self {
        self.host = Some(host);
        self // Return for chaining
    }
    
    pub fn port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }
}

// Can chain:
let config = ConfigBuilder::new()
    .host("localhost".to_string())
    .port(8080)
    .build();
```

### ❌ Anti-Pattern 6: Not Using `Into<T>` for Conversions

```rust
// ❌ BAD: Forces caller to convert to String
impl ConfigBuilder {
    pub fn host(mut self, host: String) -> Self {
        self.host = Some(host);
        self
    }
}

// Caller must convert:
let config = ConfigBuilder::new()
    .host("localhost".to_string()) // Annoying!
    .build();

// ✅ GOOD: Accept Into<String> for ergonomics
impl ConfigBuilder {
    pub fn host(mut self, host: impl Into<String>) -> Self {
        self.host = Some(host.into());
        self
    }
}

// Caller can pass &str directly:
let config = ConfigBuilder::new()
    .host("localhost") // Ergonomic!
    .build();
```

### ❌ Anti-Pattern 7: Builder State Leakage

```rust
// ❌ BAD: build() doesn't consume builder, allows reuse
impl ConfigBuilder {
    pub fn build(&self) -> Config {
        Config {
            host: self.host.clone().unwrap(),
            port: self.port.unwrap(),
        }
    }
}

// Can be misused:
let builder = ConfigBuilder::new()
    .host("localhost")
    .port(8080);

let config1 = builder.build(); // Works
let config2 = builder.build(); // Also works - confusing!

// ✅ GOOD: build() consumes builder
impl ConfigBuilder {
    pub fn build(self) -> Config {
        Config {
            host: self.host.unwrap(),
            port: self.port.unwrap(),
        }
    }
}

// Clear ownership:
let builder = ConfigBuilder::new()
    .host("localhost")
    .port(8080);

let config = builder.build();
// let config2 = builder.build(); // ERROR: builder was moved
```

---

## Conclusion

### Key Takeaways

1. **When to Use Builders:**
   - 4+ optional parameters
   - Complex validation
   - Immutable objects
   - Different representations

2. **Prefer Consuming Builders:**
   - Take `self` by value
   - Return `Self` for chaining
   - Consume in `build()`

3. **Use Type-States for Strict APIs:**
   - Enforce required fields at compile time
   - Guide users with type system
   - Prevent invalid states

4. **Embrace `Into<T>` Conversions:**
   - Accept `impl Into<String>`, `impl Into<PathBuf>`, etc.
   - Makes APIs more ergonomic
   - Reduces boilerplate for callers

5. **Validate in `build()`:**
   - Return `Result<T, E>` for fallible builders
   - Collect all validation errors
   - Provide detailed error messages

6. **Consider `derive_builder` for Simple Cases:**
   - Saves boilerplate
   - Good for internal APIs
   - Manual builders for public/complex APIs

### Quick Reference Checklist

**Builder Implementation:**
- [ ] Required fields in constructor or type-state
- [ ] Optional fields with sensible defaults
- [ ] Methods take `self` or `&mut self` and return `Self`
- [ ] Use `impl Into<T>` for string/path parameters
- [ ] Validate in `build()` method
- [ ] Return `Result<Product, Error>` for fallible builders
- [ ] Document required vs optional fields
- [ ] Implement common traits (Debug, Clone if appropriate)
- [ ] Private fields in builder (encapsulation)
- [ ] Clear documentation with examples

**Product Type:**
- [ ] Private fields (immutability)
- [ ] Getters for field access
- [ ] No setters (enforce immutability)
- [ ] Implement common traits (Debug, Clone, etc.)

---

## Further Reading

### Rust Resources
- [Rust API Guidelines - C-BUILDER](https://rust-lang.github.io/api-guidelines/type-safety.html#c-builder)
- [The Rust Book - Design Patterns](https://rust-unofficial.github.io/patterns/patterns/creational/builder.html)
- [`derive_builder` crate documentation](https://docs.rs/derive_builder/)
- [`typed-builder` crate](https://docs.rs/typed-builder/) - Alternative with type-state support

### Related Patterns
- [Newtype Pattern](./rust-adt-implementation-guide.md#newtype-pattern)
- [Type-State Pattern](https://cliffle.com/blog/rust-typestate/)
- [Builder Pattern Architecture](../../architecture/builder-pattern.md)

---

**Document Version:** 1.0  
**Last Updated:** 2026-01-25  
**Status:** Active
