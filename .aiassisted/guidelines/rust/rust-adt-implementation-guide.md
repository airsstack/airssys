# Rust ADT (Algebraic Data Type) Implementation Guide

## Overview

This document provides practical guidelines for implementing Algebraic Data Types (ADTs) in Rust. It focuses on implementation patterns, common pitfalls, and best practices for the AirS Stack ecosystem.

**Assumed Knowledge:** Basic understanding of ADTs (sum types, product types) and Rust enums/structs.

**Related Documents:**
- [Microsoft Rust Guidelines](./microsoft-rust-guidelines.md) - Follow M-COMMON-TRAITS, M-PUBLIC-DEBUG, M-UPSTREAM-GUIDELINES
- [Dependency Injection Guide](./rust-dependency-injection-dip-guide.md) - For trait abstractions over ADTs

---

## Table of Contents

1. [ADT Fundamentals in Rust](#adt-fundamentals-in-rust)
2. [Enum ADTs (Sum Types)](#enum-adts-sum-types)
3. [Struct ADTs (Product Types)](#struct-adts-product-types)
4. [Newtype Pattern](#newtype-pattern)
5. [State Machine ADTs](#state-machine-adts)
6. [Error ADTs](#error-adts)
7. [Validation and Invariants](#validation-and-invariants)
8. [Serialization and Deserialization](#serialization-and-deserialization)
9. [Testing ADTs](#testing-adts)
10. [Anti-Patterns](#anti-patterns)

---

## ADT Fundamentals in Rust

### Product Types (AND types)

A product type combines multiple values together. In Rust, these are `struct`s.

```rust
// Product type: Username AND Email
pub struct User {
    pub username: String,
    pub email: Email,
}
```

### Sum Types (OR types)

A sum type represents a choice between alternatives. In Rust, these are `enum`s.

```rust
// Sum type: Credit OR Debit OR Crypto
pub enum PaymentMethod {
    Credit,
    Debit,
    Crypto,
}
```

### Nested ADTs

Complex domain models combine both:

```rust
// Order is a product of: id AND items AND status
pub struct Order {
    pub id: OrderId,           // Newtype (product)
    pub items: Vec<LineItem>,  // Product
    pub status: OrderStatus,   // Sum type
}

// OrderStatus is a sum type with associated data
pub enum OrderStatus {
    Pending,
    Processing { estimated_completion: DateTime<Utc> },
    Shipped { tracking_number: String },
    Delivered { signed_by: String },
    Cancelled { reason: String },
}
```

---

## Enum ADTs (Sum Types)

### Basic Enum ADT Pattern

**Use Case:** Representing a closed set of alternatives

```rust
use serde::{Deserialize, Serialize};

/// Payment method for transactions.
///
/// Represents the type of payment accepted by the system.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PaymentMethod {
    /// Credit card payment
    Credit,
    /// Debit card payment
    Debit,
    /// Cryptocurrency payment
    Crypto,
    /// Bank transfer
    BankTransfer,
    /// Cash on delivery
    Cash,
}
```

**Mandatory Traits:**
- `Debug` - Required per M-PUBLIC-DEBUG
- `Clone` - Required per C-COMMON-TRAITS (if Copy-able)
- `Copy` - If enum has no heap data (recommended for simple enums)
- `PartialEq, Eq` - For comparisons
- `Hash` - If used in collections

**Optional Traits:**
- `Serialize, Deserialize` - For persistence
- `Display` - For user-facing strings
- `Default` - If there's a sensible default

### Enum with Data (Tagged Unions)

**Use Case:** When variants carry associated data

```rust
use std::path::PathBuf;

/// File operation result types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileOperationError {
    NotFound(PathBuf),
    PermissionDenied(PathBuf),
    InvalidFormat {
        path: PathBuf,
        expected: String,
        found: String,
    },
    SizeLimitExceeded {
        path: PathBuf,
        size: u64,
        limit: u64,
    },
}
```

**Pattern Matching:**

```rust
match error {
    FileOperationError::NotFound(path) => {
        eprintln!("File not found: {}", path.display());
    }
    FileOperationError::InvalidFormat { path, expected, found } => {
        eprintln!("Invalid format in {}: expected {}, found {}", 
                  path.display(), expected, found);
    }
    _ => eprintln!("Other error"),
}
```

### Non-Exhaustive Enums

**Use Case:** Public enums that may grow in future versions

```rust
/// Order status in the fulfillment pipeline.
///
/// Future versions may add additional statuses.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OrderStatus {
    Pending,
    Confirmed,
    Processing,
    Shipped,
    Delivered,
    Cancelled,
    Refunded,
}
```

**Benefits:**
- Allows adding variants without breaking API
- Forces downstream users to use `_ =>` in match expressions

**Pattern Matching with Non-Exhaustive:**

```rust
// Must have catch-all due to #[non_exhaustive]
match status {
    OrderStatus::Pending => println!("Pending"),
    OrderStatus::Confirmed => println!("Confirmed"),
    _ => println!("Other status"),
}
```

### Implementing Display for Enums

**Pattern 1: Simple String Mapping**

```rust
impl std::fmt::Display for PaymentMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Credit => write!(f, "credit"),
            Self::Debit => write!(f, "debit"),
            Self::Crypto => write!(f, "crypto"),
            Self::BankTransfer => write!(f, "bank_transfer"),
            Self::Cash => write!(f, "cash"),
        }
    }
}
```

**Pattern 2: Reusing Serde Serialization**

```rust
impl std::fmt::Display for PaymentMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Leverage serde's rename_all = "lowercase"
        write!(f, "{}", serde_json::to_string(self).unwrap().trim_matches('"'))
    }
}
```

### Default Implementations

**Use Case:** When there's a sensible default variant

```rust
impl Default for PaymentMethod {
    fn default() -> Self {
        Self::Credit // Most common payment method
    }
}

impl Default for OrderStatus {
    fn default() -> Self {
        Self::Pending // New orders start as pending
    }
}
```

**Avoid:** Default implementations for error types or states where no variant is truly "default"

---

## Struct ADTs (Product Types)

### Basic Struct ADT Pattern

**Use Case:** Grouping related data with public fields

```rust
/// A shipping address for order delivery.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Address {
    /// Street address line 1
    pub street1: String,
    /// Street address line 2 (optional)
    pub street2: Option<String>,
    /// City name
    pub city: String,
    /// State or province code
    pub state: String,
    /// Postal/ZIP code
    pub postal_code: String,
    /// Country code (ISO 3166-1 alpha-2)
    pub country: CountryCode,
}
```

**Mandatory Traits:**
- `Debug` - Required per M-PUBLIC-DEBUG
- `Clone` - Required per C-COMMON-TRAITS

**Optional Traits:**
- `PartialEq, Eq` - For comparisons
- `Serialize, Deserialize` - For persistence
- `Default` - If sensible defaults exist

### Struct with Invariants

**Use Case:** When struct fields must satisfy constraints

```rust
use chrono::{DateTime, Utc};

/// A monetary amount with currency.
///
/// # Invariants
/// - `amount` must be non-negative
/// - `amount` must have at most 2 decimal places (for most currencies)
/// - `currency` must be a valid ISO 4217 code
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Money {
    /// Amount in minor units (e.g., cents for USD)
    amount: i64,
    /// ISO 4217 currency code
    currency: CurrencyCode,
}

impl Money {
    /// Creates a new Money instance.
    ///
    /// # Errors
    /// 
    /// Returns error if amount is negative.
    pub fn new(amount: i64, currency: CurrencyCode) -> Result<Self, MoneyError> {
        if amount < 0 {
            return Err(MoneyError::NegativeAmount);
        }
        Ok(Self { amount, currency })
    }
    
    /// Returns the amount in minor units.
    pub fn amount(&self) -> i64 {
        self.amount
    }
    
    /// Returns the currency code.
    pub fn currency(&self) -> &CurrencyCode {
        &self.currency
    }
}
```

**Builder Pattern for Invariants:**

```rust
pub struct OrderBuilder {
    customer_id: Option<CustomerId>,
    items: Vec<LineItem>,
    shipping_address: Option<Address>,
    billing_address: Option<Address>,
}

impl OrderBuilder {
    pub fn new() -> Self {
        Self {
            customer_id: None,
            items: Vec::new(),
            shipping_address: None,
            billing_address: None,
        }
    }

    pub fn customer_id(mut self, id: CustomerId) -> Self {
        self.customer_id = Some(id);
        self
    }
    
    pub fn add_item(mut self, item: LineItem) -> Self {
        self.items.push(item);
        self
    }
    
    pub fn shipping_address(mut self, address: Address) -> Self {
        self.shipping_address = Some(address);
        self
    }

    pub fn build(self) -> Result<Order, OrderBuildError> {
        let customer_id = self.customer_id
            .ok_or(OrderBuildError::MissingCustomerId)?;
        
        if self.items.is_empty() {
            return Err(OrderBuildError::NoItems);
        }
        
        let shipping_address = self.shipping_address
            .ok_or(OrderBuildError::MissingShippingAddress)?;

        Ok(Order {
            id: OrderId::generate(),
            customer_id,
            items: self.items,
            shipping_address,
            billing_address: self.billing_address,
            status: OrderStatus::Pending,
            created_at: Utc::now(),
        })
    }
}
```

### Struct with Methods

**Use Case:** Encapsulating behavior with data

```rust
impl Order {
    /// Creates a new order with the given parameters.
    pub fn new(customer_id: CustomerId, items: Vec<LineItem>) -> Result<Self, OrderError> {
        if items.is_empty() {
            return Err(OrderError::NoItems);
        }
        
        Ok(Self {
            id: OrderId::generate(),
            customer_id,
            items,
            status: OrderStatus::Pending,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }
    
    /// Adds an item to the order.
    ///
    /// # Errors
    ///
    /// Returns error if order is already shipped or delivered.
    pub fn add_item(&mut self, item: LineItem) -> Result<(), OrderError> {
        if !self.can_modify() {
            return Err(OrderError::CannotModifyShippedOrder);
        }
        
        self.items.push(item);
        self.updated_at = Utc::now();
        Ok(())
    }
    
    /// Calculates the total price of all items.
    pub fn total_price(&self) -> Money {
        self.items.iter()
            .map(|item| item.price())
            .fold(Money::zero(CurrencyCode::USD), |acc, price| acc + price)
    }
    
    fn can_modify(&self) -> bool {
        matches!(self.status, OrderStatus::Pending | OrderStatus::Confirmed)
    }
}
```

**Guidelines:**
- Methods that modify state should update relevant timestamps
- Methods that maintain invariants should be the only way to modify fields (use private fields)
- Prefer methods over direct field access when invariants exist

---

## Newtype Pattern

**Use Case:** Type safety for primitive types, domain-specific validation

### Basic Newtype

```rust
use std::fmt::{self, Display, Formatter};

/// A validated email address.
/// 
/// # Examples
/// 
/// ```
/// use mylib::Email;
/// 
/// let email = Email::parse("user@example.com")?;
/// assert_eq!(email.as_str(), "user@example.com");
/// assert_eq!(email.domain(), "example.com");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Email(String);
```

**Mandatory Traits for Newtypes:**
- `Debug` - Required per M-PUBLIC-DEBUG
- `Clone` - Required per C-COMMON-TRAITS
- `PartialEq, Eq` - For comparisons
- `Hash` - If used in collections

### Newtype with Validation

```rust
impl Email {
    /// Validates and creates an Email from a string.
    /// 
    /// # Errors
    /// 
    /// Returns error if the email format is invalid.
    pub fn parse(s: &str) -> Result<Self, EmailParseError> {
        // Basic validation - in real code use a proper email validation library
        if !s.contains('@') {
            return Err(EmailParseError::MissingAtSymbol);
        }
        
        let parts: Vec<&str> = s.split('@').collect();
        if parts.len() != 2 {
            return Err(EmailParseError::InvalidFormat);
        }
        
        let local = parts[0];
        let domain = parts[1];
        
        if local.is_empty() {
            return Err(EmailParseError::EmptyLocalPart);
        }
        
        if domain.is_empty() || !domain.contains('.') {
            return Err(EmailParseError::InvalidDomain);
        }
        
        Ok(Self(s.to_lowercase()))
    }
    
    /// Creates an Email without validation (for trusted sources).
    /// 
    /// # Safety
    /// 
    /// Caller must ensure the string is a valid email address.
    pub fn new_unchecked(s: String) -> Self {
        Self(s)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EmailParseError {
    MissingAtSymbol,
    InvalidFormat,
    EmptyLocalPart,
    InvalidDomain,
}
```

### Newtype Accessors

```rust
impl Email {
    /// Returns the email address as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
    
    /// Returns the local part (before @).
    pub fn local_part(&self) -> &str {
        self.0.split('@').next().unwrap_or("")
    }
    
    /// Returns the domain part (after @).
    pub fn domain(&self) -> &str {
        self.0.split('@').nth(1).unwrap_or("")
    }
    
    /// Consumes self and returns the inner String.
    pub fn into_string(self) -> String {
        self.0
    }
}
```

### Newtype Conversions

```rust
impl Display for Email {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for Email {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

// Avoid implementing From<String> - use parse() instead
// to enforce validation
```

**Guidelines:**
- Use `new()` for validated construction
- Use `parse()` for fallible construction from strings
- Implement `AsRef<T>` for zero-cost conversions
- Implement `Display` for user-facing string representation
- **Do not** implement `From` traits that bypass validation

---

## State Machine ADTs

**Use Case:** Modeling state transitions with compile-time or runtime guarantees

### Runtime State Machine

```rust
/// State machine for order fulfillment transitions.
/// 
/// Valid transitions:
/// - Pending -> Confirmed, Cancelled
/// - Confirmed -> Processing, Cancelled
/// - Processing -> Shipped, Cancelled
/// - Shipped -> Delivered
/// - Delivered -> Refunded
#[derive(Debug, Clone, Default)]
pub struct OrderStateMachine;

impl OrderStateMachine {
    /// Creates a new state machine instance.
    pub fn new() -> Self {
        Self
    }
    
    /// Checks if a transition from one state to another is valid.
    pub fn can_transition(&self, from: OrderStatus, to: OrderStatus) -> bool {
        use OrderStatus::*;
        
        matches!(
            (from, to),
            (Pending, Confirmed) |
            (Pending, Cancelled) |
            (Confirmed, Processing) |
            (Confirmed, Cancelled) |
            (Processing, Shipped) |
            (Processing, Cancelled) |
            (Shipped, Delivered) |
            (Delivered, Refunded)
        )
    }
    
    /// Returns all valid transitions from the given state.
    pub fn valid_transitions(&self, from: OrderStatus) -> Vec<OrderStatus> {
        use OrderStatus::*;
        
        match from {
            Pending => vec![Confirmed, Cancelled],
            Confirmed => vec![Processing, Cancelled],
            Processing => vec![Shipped, Cancelled],
            Shipped => vec![Delivered],
            Delivered => vec![Refunded],
            Cancelled | Refunded => vec![],
        }
    }
    
    /// Attempts a state transition, returning error if invalid.
    pub fn transition(
        &self,
        current: OrderStatus,
        target: OrderStatus,
    ) -> Result<OrderStatus, TransitionError> {
        if !self.can_transition(current, target) {
            return Err(TransitionError::InvalidTransition {
                from: current,
                to: target,
            });
        }
        Ok(target)
    }
}
```

### Typestate Pattern (Compile-Time State Machine)

**Use Case:** When invalid states should be impossible to represent

```rust
// Marker types for states
pub struct Open;
pub struct Authenticated;
pub struct Closed;

/// Type-safe database connection that tracks state at compile time
pub struct Connection<State> {
    handle: DatabaseHandle,
    _state: std::marker::PhantomData<State>,
}

impl Connection<Open> {
    pub fn open(config: &Config) -> Result<Self, ConnectionError> {
        let handle = DatabaseHandle::connect(config)?;
        Ok(Self {
            handle,
            _state: std::marker::PhantomData,
        })
    }
    
    // Only open connections can authenticate
    pub fn authenticate(self, credentials: &Credentials) -> Result<Connection<Authenticated>, AuthError> {
        self.handle.authenticate(credentials)?;
        Ok(Connection {
            handle: self.handle,
            _state: std::marker::PhantomData,
        })
    }
}

impl Connection<Authenticated> {
    // Only authenticated connections can execute queries
    pub fn query(&self, sql: &str) -> Result<QueryResult, QueryError> {
        self.handle.execute(sql)
    }
    
    // Only authenticated connections can be closed
    pub fn close(self) -> Connection<Closed> {
        self.handle.disconnect();
        Connection {
            handle: self.handle,
            _state: std::marker::PhantomData,
        }
    }
}

impl Connection<Closed> {
    // Closed connections cannot perform any operations
    // Only drop is available
}
```

**Use Cases for Typestate:**
- File handles (open/closed)
- Database connections (connected/disconnected)
- Builders (incomplete/complete)
- Protocol states (handshake/authenticated/terminated)

**Trade-offs:**
- ✅ Invalid states impossible at compile time
- ✅ No runtime checking overhead
- ❌ More complex API
- ❌ Cannot serialize state directly
- ❌ More boilerplate code

---

## Error ADTs

### Error Enum Pattern

```rust
use thiserror::Error;

/// Errors related to payment processing operations.
#[derive(Error, Debug)]
pub enum PaymentError {
    #[error("payment method not found: {0}")]
    MethodNotFound(String),
    
    #[error("insufficient funds: required {required}, available {available}")]
    InsufficientFunds {
        required: Money,
        available: Money,
    },
    
    #[error("payment gateway error: {0}")]
    GatewayError(String),
    
    #[error("invalid card number")]
    InvalidCardNumber(#[from] CardValidationError),
    
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("network error: {0}")]
    Network(#[from] NetworkError),
}
```

**Guidelines:**
- Use `thiserror::Error` for error types (per M-ERRORS-CANONICAL-STRUCTS)
- Provide helpful error messages with context
- Use `#[from]` for automatic conversion from underlying errors
- Implement `Display` via `#[error("...")]` macro

### Error with Rich Context

```rust
#[derive(Error, Debug)]
pub enum OrderError {
    #[error("order not found: {0}")]
    NotFound(OrderId),
    
    #[error("cannot modify order {order_id} in status {status}")]
    InvalidStatusForOperation {
        order_id: OrderId,
        status: OrderStatus,
    },
    
    #[error("order {order_id} total {total} exceeds customer limit {limit}")]
    ExceedsCustomerLimit {
        order_id: OrderId,
        total: Money,
        limit: Money,
    },
}
```

---

## Validation and Invariants

### Validation Report Pattern

**Use Case:** Collecting multiple validation errors (permissive validation)

```rust
/// Result of form validation - collects all errors and warnings.
#[derive(Debug, Clone, Default)]
pub struct ValidationReport<T> {
    /// All errors found
    pub errors: Vec<ValidationError>,
    /// All warnings found
    pub warnings: Vec<String>,
    /// The validated value (if validation passed)
    pub value: Option<T>,
}

impl<T> ValidationReport<T> {
    /// Creates an empty report.
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Returns true if no errors were found.
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }
    
    /// Adds an error to the report.
    pub fn add_error(&mut self, error: ValidationError) {
        self.errors.push(error);
    }
    
    /// Adds a warning to the report.
    pub fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }
    
    /// Sets the validated value.
    pub fn with_value(mut self, value: T) -> Self {
        self.value = Some(value);
        self
    }
    
    /// Converts to Result, consuming self.
    pub fn into_result(self) -> Result<T, Vec<ValidationError>> {
        if self.is_valid() {
            Ok(self.value.expect("value must be set if valid"))
        } else {
            Err(self.errors)
        }
    }
}
```

### Validation Trait Pattern

```rust
/// Context for validation operations.
pub struct ValidationContext<'a> {
    /// Additional context data
    pub metadata: &'a HashMap<String, String>,
}

/// Trait for validation implementations.
pub trait Validator<T>: Send + Sync {
    /// Returns the name of this validator.
    fn name(&self) -> &'static str;
    
    /// Performs validation and returns a report.
    /// 
    /// This is permissive - it collects all errors rather than
    /// failing on the first one.
    fn validate(&self, value: &T, context: &ValidationContext) -> ValidationReport<()>;
}
```

---

## Serialization and Deserialization

### Basic Serde Integration

```rust
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Invoice {
    pub id: InvoiceId,
    pub customer_id: CustomerId,
    pub line_items: Vec<LineItem>,
    pub subtotal: Money,
    pub tax: Money,
    pub total: Money,
    pub issued_at: DateTime<Utc>,
    pub due_at: DateTime<Utc>,
}
```

### Custom Field Names

```rust
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShippingMethod {
    Standard,
    Express,
    Overnight,
    InternationalPriority,
}
// Serializes as: "standard", "express", "overnight", "international_priority"
```

### Default Values

```rust
#[derive(Serialize, Deserialize)]
pub struct OrderConfig {
    pub max_items: usize,
    
    #[serde(default)]
    pub allow_backorder: bool, // Uses false if missing
    
    #[serde(default = "default_shipping_method")]
    pub shipping_method: ShippingMethod,
    
    #[serde(default)]
    pub notes: Option<String>, // Uses None if missing
}

fn default_shipping_method() -> ShippingMethod {
    ShippingMethod::Standard
}
```

### Skipping Fields

```rust
#[derive(Serialize, Deserialize)]
pub struct Config {
    pub name: String,
    
    #[serde(skip)]
    pub runtime_cache: HashMap<String, String>, // Never serialized
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub optional_field: Option<String>, // Omitted if None
}
```

### Custom Serialization for Newtypes

```rust
use serde::{Deserialize, Deserializer, Serialize, Serializer};

impl Serialize for Email {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for Email {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Email::parse(&s).map_err(serde::de::Error::custom)
    }
}
```

---

## Testing ADTs

### Testing Enum Variants

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_payment_method_display() {
        assert_eq!(PaymentMethod::Credit.to_string(), "credit");
        assert_eq!(PaymentMethod::Crypto.to_string(), "crypto");
    }

    #[test]
    fn test_payment_method_default() {
        assert_eq!(PaymentMethod::default(), PaymentMethod::Credit);
    }

    #[test]
    fn test_payment_method_serde_roundtrip() {
        let method = PaymentMethod::BankTransfer;
        let json = serde_json::to_string(&method).unwrap();
        assert_eq!(json, "\"bank_transfer\"");
        
        let parsed: PaymentMethod = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, method);
    }
}
```

### Testing State Machines

```rust
#[test]
fn test_valid_order_transitions() {
    let sm = OrderStateMachine::new();
    
    assert!(sm.can_transition(OrderStatus::Pending, OrderStatus::Confirmed));
    assert!(sm.can_transition(OrderStatus::Processing, OrderStatus::Shipped));
    assert!(sm.can_transition(OrderStatus::Shipped, OrderStatus::Delivered));
}

#[test]
fn test_invalid_order_transitions() {
    let sm = OrderStateMachine::new();
    
    assert!(!sm.can_transition(OrderStatus::Pending, OrderStatus::Delivered));
    assert!(!sm.can_transition(OrderStatus::Delivered, OrderStatus::Processing));
    assert!(!sm.can_transition(OrderStatus::Cancelled, OrderStatus::Shipped));
}

#[test]
fn test_transition_result() {
    let sm = OrderStateMachine::new();
    
    let result = sm.transition(OrderStatus::Pending, OrderStatus::Confirmed);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), OrderStatus::Confirmed);
    
    let result = sm.transition(OrderStatus::Pending, OrderStatus::Delivered);
    assert!(result.is_err());
}
```

### Testing Newtypes

```rust
#[test]
### Testing Newtypes

```rust
#[test]
fn test_email_parse() {
    let result = Email::parse("user@example.com");
    assert!(result.is_ok());
    
    let email = result.unwrap();
    assert_eq!(email.as_str(), "user@example.com");
    assert_eq!(email.local_part(), "user");
    assert_eq!(email.domain(), "example.com");
}

#[test]
fn test_email_validation() {
    let result = Email::parse("invalid");
    assert!(matches!(result, Err(EmailParseError::MissingAtSymbol)));
    
    let result = Email::parse("@example.com");
    assert!(matches!(result, Err(EmailParseError::EmptyLocalPart)));
    
    let result = Email::parse("user@");
    assert!(matches!(result, Err(EmailParseError::InvalidDomain)));
}

#[test]
fn test_email_case_normalization() {
    let email1 = Email::parse("User@Example.COM").unwrap();
    let email2 = Email::parse("user@example.com").unwrap();
    assert_eq!(email1, email2);
}
```

### Property-Based Testing

```rust
#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_email_roundtrip(local in "[a-z]{1,20}", domain in "[a-z]{1,10}") {
            let email_str = format!("{}@{}.com", local, domain);
            let email = Email::parse(&email_str).unwrap();
            assert_eq!(email.as_str(), email_str);
        }
        
        #[test]
        fn test_money_non_negative(amount in 0i64..i64::MAX, currency in "[A-Z]{3}") {
            let result = Money::new(amount, CurrencyCode::from_str(&currency).unwrap());
            assert!(result.is_ok());
        }
### ❌ Primitive Obsession

**Bad:**
```rust
// Using raw strings for IDs - no type safety
pub struct Order {
    pub id: String,
    pub customer_id: String,
}

fn get_order(id: String) -> Option<Order> { /* ... */ }

// Easy to mix up arguments
get_order(order.customer_id); // Compiles but wrong!
```

**Good:**
```rust
pub struct Order {
    pub id: OrderId,
    pub customer_id: CustomerId,
}

fn get_order(id: OrderId) -> Option<Order> { /* ... */ }

get_order(order.customer_id); // Compile error - type mismatch
```

### ❌ Stringly-Typed Enums

**Bad:**
```rust
pub struct Config {
    pub mode: String, // "production" | "development" | "test"
}

fn process(config: &Config) {
    match config.mode.as_str() {
        "production" => { /* ... */ }
        "development" => { /* ... */ }
        _ => { /* typo or new mode? */ }
    }
}
```

**Good:**
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Production,
    Development,
    Test,
}

pub struct Config {
    pub mode: Mode,
}

fn process(config: &Config) {
    match config.mode {
        Mode::Production => { /* ... */ }
        Mode::Development => { /* ... */ }
        Mode::Test => { /* ... */ }
    } // Compiler ensures exhaustiveness
}
```

### ❌ Boolean Blindness

**Bad:**
```rust
pub struct ShippingOption {
    pub name: String,
    pub is_express: bool, // What does false mean?
}
```

**Good:**
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShippingSpeed {
    Standard,
    Express,
    Overnight,
}

pub struct ShippingOption {
    pub name: String,
    pub speed: ShippingSpeed,
}
```

### ❌ Unchecked Invariants

**Bad:**
```rust
pub struct Email(String); // Must be valid email format

impl Email {
    pub fn new(s: String) -> Self {
        Self(s) // No validation - invariant can be violated
    }
}
```

**Good:**
```rust
impl Email {
    pub fn parse(s: &str) -> Result<Self, EmailParseError> {
        // Validate format
        if !s.contains('@') {
            return Err(EmailParseError::MissingAtSymbol);
        }
        // More validation...
        Ok(Self(s.to_lowercase()))
    }
}
```

### ❌ Exposing Internal Representation

**Bad:**
```rust
pub struct Email(pub String); // Public inner field

// Users can bypass validation
let invalid_email = Email("not-an-email".to_string());
```

**Good:**
```rust
pub struct Email(String); // Private inner field

impl Email {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
```

### ❌ Overusing Enums with Data

**Bad:**
```rust
pub enum Config {
    Database {
        host: String,
        port: u16,
        username: String,
        password: String,
    },
    Cache {
        host: String,
        port: u16,
        ttl: Duration,
    },
    Logger {
        level: String,
        path: PathBuf,
    },
}

// Hard to compose, can only have one variant
```

**Good:**
```rust
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
}

pub struct CacheConfig {
    pub host: String,
    pub port: u16,
    pub ttl: Duration,
}

pub struct LoggerConfig {
    pub level: LogLevel,
    pub path: PathBuf,
}

pub struct Config {
    pub database: DatabaseConfig,
    pub cache: CacheConfig,
    pub logger: LoggerConfig,
}
```

---

## Quick Reference Checklist

When implementing ADTs, ensure:

- [ ] All public types implement `Debug` (M-PUBLIC-DEBUG)
- [ ] All types implement `Clone` where appropriate (C-COMMON-TRAITS)
- [ ] Enums implement `Copy` if they contain no heap data
- [ ] Newtypes use private inner fields (C-NEWTYPE)
- [ ] Error types use `thiserror::Error` (M-ERRORS-CANONICAL-STRUCTS)
- [ ] Enums have `#[non_exhaustive]` if they may grow
- [ ] Validation errors are collected, not fail-fast (permissive validation)
- [ ] Serde integration uses `#[serde(default)]` for optional fields
- [ ] State machines validate transitions before applying
- [ ] Unit tests cover all enum variants and edge cases
- [ ] Documentation includes examples for complex types
- [ ] Avoid primitive obsession - use strong types
- [ ] Avoid boolean blindness - use enums for choices

---

## Further Reading

- [Rust API Guidelines - C-COMMON-TRAITS](https://rust-lang.github.io/api-guidelines/interoperability.html#c-common-traits)
- [Rust API Guidelines - C-NEWTYPE](https://rust-lang.github.io/api-guidelines/type-safety.html#c-newtype)
- [Rust Design Patterns - Newtype](https://rust-unofficial.github.io/patterns/patterns/behavioural/newtype.html)
- [Rust Design Patterns - Typestate](https://rust-unofficial.github.io/patterns/patterns/behavioural/typestate.html)
- [Microsoft Rust Guidelines](./microsoft-rust-guidelines.md)

---

**Document Version:** 1.0  
**Last Updated:** 2026-01-25  
**Maintainer:** AirS Stack Team
