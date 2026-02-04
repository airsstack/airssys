# Rust Factory Pattern Implementation Guide

## Overview

This document provides practical guidelines for implementing the Factory Pattern in Rust. It focuses on Rust-specific idioms, trait-based designs, and zero-cost abstractions for creating ergonomic factory APIs in the AirS Stack ecosystem.

**Assumed Knowledge:** Basic understanding of the Factory Pattern (see [Factory Pattern Architecture Guide](../../architecture/factory-pattern.md)) and intermediate Rust knowledge including traits and enums.

**Related Documents:**
- [Factory Pattern Architecture Guide](../../architecture/factory-pattern.md) - Language-agnostic pattern fundamentals
- [Microsoft Rust Guidelines](./microsoft-rust-guidelines.md) - Follow M-COMMON-TRAITS, M-PUBLIC-DEBUG, M-UPSTREAM-GUIDELINES
- [Rust ADT Implementation Guide](./rust-adt-implementation-guide.md) - For product type patterns
- [Rust Builder Pattern Guide](./rust-builder-pattern-guide.md) - Combining factories with builders

---

## Table of Contents

1. [When to Use Factories in Rust](#when-to-use-factories-in-rust)
2. [Simple Factory Pattern](#simple-factory-pattern)
3. [Trait-Based Factory Method](#trait-based-factory-method)
4. [Enum-Based Factory](#enum-based-factory)
5. [Registry-Based Factory](#registry-based-factory)
6. [Abstract Factory Pattern](#abstract-factory-pattern)
7. [Factory with Type Parameters](#factory-with-type-parameters)
8. [Async Factories](#async-factories)
9. [Testing Factories](#testing-factories)
10. [Common Patterns](#common-patterns)
11. [Anti-Patterns](#anti-patterns)

---

## When to Use Factories in Rust

### ✅ Use Factories When:

1. **Multiple Product Types with Shared Interface**
   ```rust
   // ❌ BAD: Client code tightly coupled to concrete types
   fn process_data(db_type: &str) {
       let connection = match db_type {
           "postgres" => PostgresConnection::new("..."),
           "mysql" => MySqlConnection::new("..."),
           _ => panic!("Unknown database"),
       };
       // Use connection...
   }
   
   // ✅ GOOD: Factory abstracts creation
   fn process_data(db_type: &str) {
       let connection = ConnectionFactory::create(db_type)
           .expect("Failed to create connection");
       // Use connection through trait interface
   }
   ```

2. **Complex Initialization Logic**
   ```rust
   // Factory encapsulates complex setup
   pub struct HttpClientFactory;
   
   impl HttpClientFactory {
       pub fn create(config: &Config) -> Result<Box<dyn HttpClient>, Error> {
           let mut client = ReqwestClient::new();
           client.set_timeout(config.timeout);
           client.add_default_headers(&config.headers);
           client.configure_tls(&config.tls_config)?;
           client.setup_connection_pool(config.pool_size);
           Ok(Box::new(client))
       }
   }
   ```

3. **Runtime Type Selection**
   ```rust
   // Choose implementation based on runtime configuration
   let logger = LoggerFactory::from_env()?;
   ```

4. **Resource Pooling**
   ```rust
   // Factory manages connection pool
   let connection = pool_factory.acquire().await?;
   ```

### ❌ Avoid Factories When:

1. **Simple Types with No Variants**
   ```rust
   // ❌ Over-engineering
   struct PointFactory;
   impl PointFactory {
       fn create(x: i32, y: i32) -> Point {
           Point { x, y }
       }
   }
   
   // ✅ Direct construction is clearer
   let point = Point { x: 10, y: 20 };
   ```

2. **Single Implementation**
   ```rust
   // ❌ Unnecessary abstraction
   trait UserFactory {
       fn create_user(&self, name: String) -> User;
   }
   
   // ✅ Just use a constructor or associated function
   impl User {
       pub fn new(name: String) -> Self {
           Self { name }
       }
   }
   ```

---

## Simple Factory Pattern

### Pattern 1: Static Factory Functions

**Use Case:** Create different types based on a parameter with no state.

```rust
use std::fmt;
use thiserror::Error;

/// Transport interface - all transport types implement this.
pub trait Transport: fmt::Debug {
    fn deliver(&self, destination: &str) -> String;
    fn cost(&self) -> f64;
}

/// Truck transport implementation.
#[derive(Debug, Clone)]
pub struct Truck {
    capacity_tons: u32,
}

impl Transport for Truck {
    fn deliver(&self, destination: &str) -> String {
        format!("Delivering by truck to {} (capacity: {} tons)", 
                destination, self.capacity_tons)
    }
    
    fn cost(&self) -> f64 {
        50.0 * self.capacity_tons as f64
    }
}

/// Ship transport implementation.
#[derive(Debug, Clone)]
pub struct Ship {
    container_count: u32,
}

impl Transport for Ship {
    fn deliver(&self, destination: &str) -> String {
        format!("Delivering by ship to {} ({} containers)", 
                destination, self.container_count)
    }
    
    fn cost(&self) -> f64 {
        200.0 * self.container_count as f64
    }
}

/// Airplane transport implementation.
#[derive(Debug, Clone)]
pub struct Airplane {
    cargo_weight_kg: u32,
}

impl Transport for Airplane {
    fn deliver(&self, destination: &str) -> String {
        format!("Delivering by air to {} ({} kg cargo)", 
                destination, self.cargo_weight_kg)
    }
    
    fn cost(&self) -> f64 {
        0.5 * self.cargo_weight_kg as f64
    }
}

/// Transport types supported by the factory.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransportType {
    Truck,
    Ship,
    Airplane,
}

/// Error type for transport factory.
#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum TransportFactoryError {
    #[error("unknown transport type: {0}")]
    UnknownType(String),
}

/// Simple factory for creating transport instances.
pub struct TransportFactory;

impl TransportFactory {
    /// Creates a transport instance based on type.
    ///
    /// # Errors
    ///
    /// Returns an error if the transport type is not recognized.
    pub fn create(transport_type: TransportType) -> Box<dyn Transport> {
        match transport_type {
            TransportType::Truck => Box::new(Truck { capacity_tons: 20 }),
            TransportType::Ship => Box::new(Ship { container_count: 100 }),
            TransportType::Airplane => Box::new(Airplane { cargo_weight_kg: 10000 }),
        }
    }
    
    /// Creates a transport instance from a string identifier.
    ///
    /// # Errors
    ///
    /// Returns an error if the transport type string is not recognized.
    pub fn create_from_str(type_str: &str) -> Result<Box<dyn Transport>, TransportFactoryError> {
        match type_str.to_lowercase().as_str() {
            "truck" => Ok(Box::new(Truck { capacity_tons: 20 })),
            "ship" => Ok(Box::new(Ship { container_count: 100 })),
            "airplane" | "plane" => Ok(Box::new(Airplane { cargo_weight_kg: 10000 })),
            _ => Err(TransportFactoryError::UnknownType(type_str.to_string())),
        }
    }
    
    /// Creates a custom truck with specified capacity.
    pub fn create_truck(capacity_tons: u32) -> Box<dyn Transport> {
        Box::new(Truck { capacity_tons })
    }
    
    /// Creates a custom ship with specified container count.
    pub fn create_ship(container_count: u32) -> Box<dyn Transport> {
        Box::new(Ship { container_count })
    }
    
    /// Creates a custom airplane with specified cargo weight.
    pub fn create_airplane(cargo_weight_kg: u32) -> Box<dyn Transport> {
        Box::new(Airplane { cargo_weight_kg })
    }
}

// Usage example:
fn example() -> Result<(), TransportFactoryError> {
    // Using enum variant
    let transport = TransportFactory::create(TransportType::Truck);
    println!("{}", transport.deliver("New York"));
    
    // Using string identifier
    let transport = TransportFactory::create_from_str("ship")?;
    println!("Cost: ${}", transport.cost());
    
    // Custom configuration
    let truck = TransportFactory::create_truck(50);
    println!("{}", truck.deliver("Los Angeles"));
    
    Ok(())
}
```

**Key Points:**
- Factory is a zero-sized struct with static methods
- Returns `Box<dyn Trait>` for trait objects
- Provides both enum-based and string-based creation
- Custom factory methods for specific configurations
- Clear error handling with `Result`

### Pattern 2: Stateful Factory

**Use Case:** Factory needs configuration or dependencies.

```rust
use std::sync::Arc;

/// Logger interface.
pub trait Logger: Send + Sync {
    fn log(&self, level: LogLevel, message: &str);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

/// Console logger implementation.
struct ConsoleLogger;

impl Logger for ConsoleLogger {
    fn log(&self, level: LogLevel, message: &str) {
        println!("[{:?}] {}", level, message);
    }
}

/// File logger implementation.
struct FileLogger {
    path: String,
}

impl Logger for FileLogger {
    fn log(&self, level: LogLevel, message: &str) {
        // In real code, write to file
        eprintln!("[{:?}] {} (to file: {})", level, message, self.path);
    }
}

/// Configuration for the logger factory.
#[derive(Debug, Clone)]
pub struct LoggerConfig {
    pub environment: String,
    pub log_path: Option<String>,
    pub min_level: LogLevel,
}

/// Stateful factory for creating loggers.
pub struct LoggerFactory {
    config: LoggerConfig,
}

impl LoggerFactory {
    /// Creates a new logger factory with the given configuration.
    pub fn new(config: LoggerConfig) -> Self {
        Self { config }
    }
    
    /// Creates a logger based on the factory configuration.
    pub fn create_logger(&self) -> Arc<dyn Logger> {
        match self.config.environment.as_str() {
            "production" => {
                if let Some(path) = &self.config.log_path {
                    Arc::new(FileLogger {
                        path: path.clone(),
                    })
                } else {
                    Arc::new(ConsoleLogger)
                }
            }
            "development" | "test" => Arc::new(ConsoleLogger),
            _ => Arc::new(ConsoleLogger),
        }
    }
    
    /// Creates a logger from environment variables.
    pub fn from_env() -> Result<Self, std::env::VarError> {
        let environment = std::env::var("APP_ENV")
            .unwrap_or_else(|_| "development".to_string());
        let log_path = std::env::var("LOG_PATH").ok();
        
        Ok(Self::new(LoggerConfig {
            environment,
            log_path,
            min_level: LogLevel::Info,
        }))
    }
}

// Usage:
fn example() -> Result<(), std::env::VarError> {
    let config = LoggerConfig {
        environment: "production".to_string(),
        log_path: Some("/var/log/app.log".to_string()),
        min_level: LogLevel::Info,
    };
    
    let factory = LoggerFactory::new(config);
    let logger = factory.create_logger();
    
    logger.log(LogLevel::Info, "Application started");
    
    Ok(())
}
```

---

## Trait-Based Factory Method

**Use Case:** Extensible factory pattern where different implementations can define their own creation logic.

```rust
use std::error::Error;

/// Document interface.
pub trait Document: std::fmt::Debug {
    fn open(&self) -> Result<(), Box<dyn Error>>;
    fn save(&self) -> Result<(), Box<dyn Error>>;
    fn close(&self) -> Result<(), Box<dyn Error>>;
}

/// PDF document implementation.
#[derive(Debug)]
pub struct PdfDocument {
    path: String,
}

impl Document for PdfDocument {
    fn open(&self) -> Result<(), Box<dyn Error>> {
        println!("Opening PDF: {}", self.path);
        Ok(())
    }
    
    fn save(&self) -> Result<(), Box<dyn Error>> {
        println!("Saving PDF: {}", self.path);
        Ok(())
    }
    
    fn close(&self) -> Result<(), Box<dyn Error>> {
        println!("Closing PDF: {}", self.path);
        Ok(())
    }
}

/// Word document implementation.
#[derive(Debug)]
pub struct WordDocument {
    path: String,
}

impl Document for WordDocument {
    fn open(&self) -> Result<(), Box<dyn Error>> {
        println!("Opening Word document: {}", self.path);
        Ok(())
    }
    
    fn save(&self) -> Result<(), Box<dyn Error>> {
        println!("Saving Word document: {}", self.path);
        Ok(())
    }
    
    fn close(&self) -> Result<(), Box<dyn Error>> {
        println!("Closing Word document: {}", self.path);
        Ok(())
    }
}

/// Factory method trait - implementors define how to create documents.
pub trait DocumentCreator {
    /// Factory method - subclasses implement this to create specific document types.
    fn create_document(&self, path: &str) -> Box<dyn Document>;
    
    /// Template method that uses the factory method.
    fn open_document(&self, path: &str) -> Result<(), Box<dyn Error>> {
        let document = self.create_document(path);
        document.open()?;
        Ok(())
    }
}

/// PDF document creator.
pub struct PdfCreator;

impl DocumentCreator for PdfCreator {
    fn create_document(&self, path: &str) -> Box<dyn Document> {
        Box::new(PdfDocument {
            path: path.to_string(),
        })
    }
}

/// Word document creator.
pub struct WordCreator;

impl DocumentCreator for WordCreator {
    fn create_document(&self, path: &str) -> Box<dyn Document> {
        Box::new(WordDocument {
            path: path.to_string(),
        })
    }
}

/// Application that uses document creators.
pub struct Application {
    creator: Box<dyn DocumentCreator>,
}

impl Application {
    pub fn new(creator: Box<dyn DocumentCreator>) -> Self {
        Self { creator }
    }
    
    pub fn open_file(&self, path: &str) -> Result<(), Box<dyn Error>> {
        self.creator.open_document(path)
    }
}

// Usage:
fn example() -> Result<(), Box<dyn Error>> {
    // PDF application
    let pdf_app = Application::new(Box::new(PdfCreator));
    pdf_app.open_file("report.pdf")?;
    
    // Word application
    let word_app = Application::new(Box::new(WordCreator));
    word_app.open_file("letter.docx")?;
    
    Ok(())
}
```

**Key Points:**
- `DocumentCreator` trait defines factory method
- Each creator implements the factory method for specific product
- Template method `open_document` uses the factory method
- Allows runtime polymorphism with trait objects
- Easy to add new document types without modifying existing code

---

## Enum-Based Factory

**Use Case:** When all product types are known at compile time and you want exhaustive matching.

```rust
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Payment processor interface.
pub trait PaymentProcessor {
    fn process_payment(&self, amount: f64, details: &PaymentDetails) -> Result<String, PaymentError>;
    fn refund(&self, transaction_id: &str) -> Result<(), PaymentError>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentDetails {
    pub card_number: String,
    pub cvv: String,
    pub expiry: String,
}

#[derive(Debug, Error)]
pub enum PaymentError {
    #[error("payment declined")]
    Declined,
    #[error("invalid details: {0}")]
    InvalidDetails(String),
    #[error("network error: {0}")]
    NetworkError(String),
}

/// Stripe payment processor.
struct StripeProcessor;

impl PaymentProcessor for StripeProcessor {
    fn process_payment(&self, amount: f64, _details: &PaymentDetails) -> Result<String, PaymentError> {
        println!("Processing ${} via Stripe", amount);
        Ok(format!("stripe_tx_{}", uuid::Uuid::new_v4()))
    }
    
    fn refund(&self, transaction_id: &str) -> Result<(), PaymentError> {
        println!("Refunding Stripe transaction: {}", transaction_id);
        Ok(())
    }
}

/// PayPal payment processor.
struct PayPalProcessor;

impl PaymentProcessor for PayPalProcessor {
    fn process_payment(&self, amount: f64, _details: &PaymentDetails) -> Result<String, PaymentError> {
        println!("Processing ${} via PayPal", amount);
        Ok(format!("paypal_tx_{}", uuid::Uuid::new_v4()))
    }
    
    fn refund(&self, transaction_id: &str) -> Result<(), PaymentError> {
        println!("Refunding PayPal transaction: {}", transaction_id);
        Ok(())
    }
}

/// Square payment processor.
struct SquareProcessor;

impl PaymentProcessor for SquareProcessor {
    fn process_payment(&self, amount: f64, _details: &PaymentDetails) -> Result<String, PaymentError> {
        println!("Processing ${} via Square", amount);
        Ok(format!("square_tx_{}", uuid::Uuid::new_v4()))
    }
    
    fn refund(&self, transaction_id: &str) -> Result<(), PaymentError> {
        println!("Refunding Square transaction: {}", transaction_id);
        Ok(())
    }
}

/// Payment gateway enumeration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PaymentGateway {
    Stripe,
    PayPal,
    Square,
}

impl PaymentGateway {
    /// Factory method on the enum itself.
    pub fn create_processor(self) -> Box<dyn PaymentProcessor> {
        match self {
            Self::Stripe => Box::new(StripeProcessor),
            Self::PayPal => Box::new(PayPalProcessor),
            Self::Square => Box::new(SquareProcessor),
        }
    }
    
    /// Parses gateway from string.
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "stripe" => Some(Self::Stripe),
            "paypal" => Some(Self::PayPal),
            "square" => Some(Self::Square),
            _ => None,
        }
    }
}

// Alternative: Factory struct for more complex logic
pub struct PaymentProcessorFactory {
    api_keys: std::collections::HashMap<PaymentGateway, String>,
}

impl PaymentProcessorFactory {
    pub fn new() -> Self {
        Self {
            api_keys: std::collections::HashMap::new(),
        }
    }
    
    pub fn add_api_key(&mut self, gateway: PaymentGateway, key: String) {
        self.api_keys.insert(gateway, key);
    }
    
    pub fn create(&self, gateway: PaymentGateway) -> Result<Box<dyn PaymentProcessor>, String> {
        if !self.api_keys.contains_key(&gateway) {
            return Err(format!("No API key configured for {:?}", gateway));
        }
        
        Ok(gateway.create_processor())
    }
}

// Usage:
fn example() -> Result<(), PaymentError> {
    // Direct enum-based factory
    let processor = PaymentGateway::Stripe.create_processor();
    
    let details = PaymentDetails {
        card_number: "4242424242424242".to_string(),
        cvv: "123".to_string(),
        expiry: "12/25".to_string(),
    };
    
    let tx_id = processor.process_payment(99.99, &details)?;
    println!("Transaction ID: {}", tx_id);
    
    // Factory with configuration
    let mut factory = PaymentProcessorFactory::new();
    factory.add_api_key(PaymentGateway::Stripe, "sk_test_...".to_string());
    factory.add_api_key(PaymentGateway::PayPal, "paypal_key_...".to_string());
    
    let processor = factory.create(PaymentGateway::PayPal)
        .map_err(|e| PaymentError::InvalidDetails(e))?;
    
    Ok(())
}
```

**Key Points:**
- Enum variants represent all possible product types
- Exhaustive matching ensures all types are handled
- Factory method can be on the enum itself
- Combine with factory struct for configuration/validation
- Type-safe - compiler ensures completeness

---

## Registry-Based Factory

**Use Case:** Extensible factory where types can be registered at runtime (plugins, dynamic loading).

```rust
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Notification interface.
pub trait Notification: Send + Sync {
    fn send(&self, recipient: &str, message: &str) -> Result<(), String>;
}

/// Type alias for notification constructor.
type NotificationConstructor = Box<dyn Fn(&NotificationConfig) -> Box<dyn Notification> + Send + Sync>;

/// Configuration for notifications.
#[derive(Debug, Clone)]
pub struct NotificationConfig {
    pub api_key: Option<String>,
    pub endpoint: Option<String>,
    pub timeout_secs: u64,
}

/// Email notification implementation.
struct EmailNotification {
    smtp_server: String,
}

impl Notification for EmailNotification {
    fn send(&self, recipient: &str, message: &str) -> Result<(), String> {
        println!("Sending email to {} via {}: {}", recipient, self.smtp_server, message);
        Ok(())
    }
}

/// SMS notification implementation.
struct SmsNotification {
    twilio_key: String,
}

impl Notification for SmsNotification {
    fn send(&self, recipient: &str, message: &str) -> Result<(), String> {
        println!("Sending SMS to {} via Twilio ({}): {}", recipient, self.twilio_key, message);
        Ok(())
    }
}

/// Registry-based notification factory.
pub struct NotificationFactory {
    registry: Arc<RwLock<HashMap<String, NotificationConstructor>>>,
}

impl NotificationFactory {
    /// Creates a new factory with empty registry.
    pub fn new() -> Self {
        Self {
            registry: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Registers a notification type with its constructor.
    pub fn register<F>(&self, name: &str, constructor: F)
    where
        F: Fn(&NotificationConfig) -> Box<dyn Notification> + Send + Sync + 'static,
    {
        let mut registry = self.registry.write().unwrap();
        registry.insert(name.to_string(), Box::new(constructor));
    }
    
    /// Creates a notification instance by name.
    pub fn create(&self, name: &str, config: &NotificationConfig) -> Result<Box<dyn Notification>, String> {
        let registry = self.registry.read().unwrap();
        
        if let Some(constructor) = registry.get(name) {
            Ok(constructor(config))
        } else {
            Err(format!("Unknown notification type: {}", name))
        }
    }
    
    /// Lists all registered notification types.
    pub fn list_types(&self) -> Vec<String> {
        let registry = self.registry.read().unwrap();
        registry.keys().cloned().collect()
    }
}

impl Default for NotificationFactory {
    fn default() -> Self {
        let factory = Self::new();
        
        // Register built-in types
        factory.register("email", |config| {
            Box::new(EmailNotification {
                smtp_server: config.endpoint.clone().unwrap_or_else(|| "smtp.example.com".to_string()),
            })
        });
        
        factory.register("sms", |config| {
            Box::new(SmsNotification {
                twilio_key: config.api_key.clone().unwrap_or_default(),
            })
        });
        
        factory
    }
}

// Usage:
fn example() -> Result<(), String> {
    let factory = NotificationFactory::default();
    
    // List available types
    println!("Available notification types: {:?}", factory.list_types());
    
    // Create instances
    let config = NotificationConfig {
        api_key: Some("twilio_key_123".to_string()),
        endpoint: Some("smtp.gmail.com".to_string()),
        timeout_secs: 30,
    };
    
    let email = factory.create("email", &config)?;
    email.send("user@example.com", "Hello!")?;
    
    let sms = factory.create("sms", &config)?;
    sms.send("+1234567890", "Alert: System down")?;
    
    // Register custom notification type
    factory.register("slack", |config| {
        struct SlackNotification {
            webhook: String,
        }
        impl Notification for SlackNotification {
            fn send(&self, recipient: &str, message: &str) -> Result<(), String> {
                println!("Sending Slack message to {} via {}: {}", recipient, self.webhook, message);
                Ok(())
            }
        }
        Box::new(SlackNotification {
            webhook: config.endpoint.clone().unwrap_or_default(),
        })
    });
    
    let slack = factory.create("slack", &config)?;
    slack.send("#alerts", "Deployment complete")?;
    
    Ok(())
}
```

**Key Points:**
- Registry stored in `Arc<RwLock<HashMap<...>>>`
- Thread-safe registration and creation
- Constructor functions stored as closures
- Allows plugin-style extensibility
- Built-in types registered in `Default` implementation

---

## Abstract Factory Pattern

**Use Case:** Creating families of related objects (e.g., UI components for different platforms).

```rust
use std::fmt;

/// Button interface.
pub trait Button: fmt::Debug {
    fn render(&self) -> String;
    fn on_click(&self, handler: &str) -> String;
}

/// Checkbox interface.
pub trait Checkbox: fmt::Debug {
    fn render(&self) -> String;
    fn on_toggle(&self, handler: &str) -> String;
}

/// Text field interface.
pub trait TextField: fmt::Debug {
    fn render(&self) -> String;
    fn on_change(&self, handler: &str) -> String;
}

// Windows implementations
#[derive(Debug)]
struct WindowsButton;

impl Button for WindowsButton {
    fn render(&self) -> String {
        "Rendering Windows-style button".to_string()
    }
    
    fn on_click(&self, handler: &str) -> String {
        format!("Windows button clicked: {}", handler)
    }
}

#[derive(Debug)]
struct WindowsCheckbox;

impl Checkbox for WindowsCheckbox {
    fn render(&self) -> String {
        "Rendering Windows-style checkbox".to_string()
    }
    
    fn on_toggle(&self, handler: &str) -> String {
        format!("Windows checkbox toggled: {}", handler)
    }
}

#[derive(Debug)]
struct WindowsTextField;

impl TextField for WindowsTextField {
    fn render(&self) -> String {
        "Rendering Windows-style text field".to_string()
    }
    
    fn on_change(&self, handler: &str) -> String {
        format!("Windows text field changed: {}", handler)
    }
}

// macOS implementations
#[derive(Debug)]
struct MacButton;

impl Button for MacButton {
    fn render(&self) -> String {
        "Rendering macOS-style button".to_string()
    }
    
    fn on_click(&self, handler: &str) -> String {
        format!("macOS button clicked: {}", handler)
    }
}

#[derive(Debug)]
struct MacCheckbox;

impl Checkbox for MacCheckbox {
    fn render(&self) -> String {
        "Rendering macOS-style checkbox".to_string()
    }
    
    fn on_toggle(&self, handler: &str) -> String {
        format!("macOS checkbox toggled: {}", handler)
    }
}

#[derive(Debug)]
struct MacTextField;

impl TextField for MacTextField {
    fn render(&self) -> String {
        "Rendering macOS-style text field".to_string()
    }
    
    fn on_change(&self, handler: &str) -> String {
        format!("macOS text field changed: {}", handler)
    }
}

/// Abstract factory trait for creating UI component families.
pub trait UiFactory {
    fn create_button(&self) -> Box<dyn Button>;
    fn create_checkbox(&self) -> Box<dyn Checkbox>;
    fn create_text_field(&self) -> Box<dyn TextField>;
}

/// Windows UI factory.
pub struct WindowsUiFactory;

impl UiFactory for WindowsUiFactory {
    fn create_button(&self) -> Box<dyn Button> {
        Box::new(WindowsButton)
    }
    
    fn create_checkbox(&self) -> Box<dyn Checkbox> {
        Box::new(WindowsCheckbox)
    }
    
    fn create_text_field(&self) -> Box<dyn TextField> {
        Box::new(WindowsTextField)
    }
}

/// macOS UI factory.
pub struct MacUiFactory;

impl UiFactory for MacUiFactory {
    fn create_button(&self) -> Box<dyn Button> {
        Box::new(MacButton)
    }
    
    fn create_checkbox(&self) -> Box<dyn Checkbox> {
        Box::new(MacCheckbox)
    }
    
    fn create_text_field(&self) -> Box<dyn TextField> {
        Box::new(MacTextField)
    }
}

/// Application that uses UI components.
pub struct Application {
    factory: Box<dyn UiFactory>,
}

impl Application {
    pub fn new(factory: Box<dyn UiFactory>) -> Self {
        Self { factory }
    }
    
    pub fn render_form(&self) {
        let button = self.factory.create_button();
        let checkbox = self.factory.create_checkbox();
        let text_field = self.factory.create_text_field();
        
        println!("{}", button.render());
        println!("{}", checkbox.render());
        println!("{}", text_field.render());
    }
}

// Usage:
fn example() {
    // Determine platform at runtime
    let platform = std::env::consts::OS;
    
    let factory: Box<dyn UiFactory> = match platform {
        "windows" => Box::new(WindowsUiFactory),
        "macos" => Box::new(MacUiFactory),
        _ => Box::new(WindowsUiFactory), // Default
    };
    
    let app = Application::new(factory);
    app.render_form();
}
```

**Key Points:**
- Factory trait defines methods for creating related products
- Each concrete factory creates a family of consistent products
- Client code works with abstract interfaces
- Ensures product compatibility within a family
- Easy to add new platform support

---

## Factory with Type Parameters

**Use Case:** Generic factory using Rust's type system for compile-time polymorphism.

```rust
use std::marker::PhantomData;

/// Parser interface.
pub trait Parser {
    type Output;
    
    fn parse(&self, input: &str) -> Result<Self::Output, String>;
}

/// JSON parser.
pub struct JsonParser;

impl Parser for JsonParser {
    type Output = serde_json::Value;
    
    fn parse(&self, input: &str) -> Result<Self::Output, String> {
        serde_json::from_str(input).map_err(|e| e.to_string())
    }
}

/// TOML parser.
pub struct TomlParser;

impl Parser for TomlParser {
    type Output = toml::Value;
    
    fn parse(&self, input: &str) -> Result<Self::Output, String> {
        toml::from_str(input).map_err(|e| e.to_string())
    }
}

/// Generic parser factory using type parameters.
pub struct ParserFactory<P: Parser> {
    _phantom: PhantomData<P>,
}

impl<P: Parser + Default> ParserFactory<P> {
    pub fn create() -> P {
        P::default()
    }
}

impl<P: Parser> ParserFactory<P> {
    pub fn create_with(parser: P) -> P {
        parser
    }
}

impl Default for JsonParser {
    fn default() -> Self {
        Self
    }
}

impl Default for TomlParser {
    fn default() -> Self {
        Self
    }
}

// Alternative: Type-based factory without PhantomData
pub struct TypedParserFactory;

impl TypedParserFactory {
    pub fn json() -> JsonParser {
        JsonParser
    }
    
    pub fn toml() -> TomlParser {
        TomlParser
    }
}

// Usage:
fn example() -> Result<(), String> {
    // Using generic factory
    let json_parser = ParserFactory::<JsonParser>::create();
    let value = json_parser.parse(r#"{"key": "value"}"#)?;
    println!("JSON: {:?}", value);
    
    let toml_parser = ParserFactory::<TomlParser>::create();
    let value = toml_parser.parse("key = \"value\"")?;
    println!("TOML: {:?}", value);
    
    // Using typed factory
    let json_parser = TypedParserFactory::json();
    let value = json_parser.parse(r#"{"name": "Alice"}"#)?;
    
    Ok(())
}
```

**Key Points:**
- Uses type parameters for compile-time polymorphism
- No runtime overhead (zero-cost abstraction)
- Type safety - wrong parser type caught at compile time
- PhantomData when factory needs to carry type information
- Consider simple functions instead of factory struct

---

## Async Factories

**Use Case:** Creating objects that require async initialization.

```rust
use std::sync::Arc;
use tokio::time::{sleep, Duration};

/// Database connection interface.
#[async_trait::async_trait]
pub trait DatabaseConnection: Send + Sync {
    async fn query(&self, sql: &str) -> Result<Vec<String>, String>;
    async fn execute(&self, sql: &str) -> Result<u64, String>;
}

/// PostgreSQL connection.
struct PostgresConnection {
    host: String,
    port: u16,
}

#[async_trait::async_trait]
impl DatabaseConnection for PostgresConnection {
    async fn query(&self, sql: &str) -> Result<Vec<String>, String> {
        println!("Querying Postgres at {}:{}: {}", self.host, self.port, sql);
        sleep(Duration::from_millis(10)).await; // Simulate network delay
        Ok(vec!["result1".to_string(), "result2".to_string()])
    }
    
    async fn execute(&self, sql: &str) -> Result<u64, String> {
        println!("Executing on Postgres at {}:{}: {}", self.host, self.port, sql);
        sleep(Duration::from_millis(10)).await;
        Ok(1)
    }
}

/// MySQL connection.
struct MySqlConnection {
    host: String,
    port: u16,
}

#[async_trait::async_trait]
impl DatabaseConnection for MySqlConnection {
    async fn query(&self, sql: &str) -> Result<Vec<String>, String> {
        println!("Querying MySQL at {}:{}: {}", self.host, self.port, sql);
        sleep(Duration::from_millis(10)).await;
        Ok(vec!["mysql_result".to_string()])
    }
    
    async fn execute(&self, sql: &str) -> Result<u64, String> {
        println!("Executing on MySQL at {}:{}: {}", self.host, self.port, sql);
        sleep(Duration::from_millis(10)).await;
        Ok(1)
    }
}

/// Database type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DatabaseType {
    Postgres,
    MySql,
}

/// Configuration for database connections.
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub db_type: DatabaseType,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
}

/// Async factory for database connections.
pub struct DatabaseConnectionFactory;

impl DatabaseConnectionFactory {
    /// Creates and initializes a database connection asynchronously.
    pub async fn create(config: DatabaseConfig) -> Result<Arc<dyn DatabaseConnection>, String> {
        println!("Connecting to {:?} database...", config.db_type);
        
        // Simulate async connection establishment
        sleep(Duration::from_millis(100)).await;
        
        let connection: Arc<dyn DatabaseConnection> = match config.db_type {
            DatabaseType::Postgres => Arc::new(PostgresConnection {
                host: config.host,
                port: config.port,
            }),
            DatabaseType::MySql => Arc::new(MySqlConnection {
                host: config.host,
                port: config.port,
            }),
        };
        
        // Perform async initialization
        connection.query("SELECT 1").await?;
        
        println!("Connected successfully!");
        Ok(connection)
    }
    
    /// Creates a connection from environment variables.
    pub async fn from_env() -> Result<Arc<dyn DatabaseConnection>, String> {
        let db_type_str = std::env::var("DB_TYPE").unwrap_or_else(|_| "postgres".to_string());
        let db_type = match db_type_str.as_str() {
            "postgres" => DatabaseType::Postgres,
            "mysql" => DatabaseType::MySql,
            _ => return Err(format!("Unknown database type: {}", db_type_str)),
        };
        
        let config = DatabaseConfig {
            db_type,
            host: std::env::var("DB_HOST").unwrap_or_else(|_| "localhost".to_string()),
            port: std::env::var("DB_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(5432),
            username: std::env::var("DB_USER").unwrap_or_else(|_| "postgres".to_string()),
            password: std::env::var("DB_PASS").unwrap_or_default(),
        };
        
        Self::create(config).await
    }
}

// Usage:
#[tokio::main]
async fn example() -> Result<(), String> {
    let config = DatabaseConfig {
        db_type: DatabaseType::Postgres,
        host: "localhost".to_string(),
        port: 5432,
        username: "admin".to_string(),
        password: "secret".to_string(),
    };
    
    let connection = DatabaseConnectionFactory::create(config).await?;
    
    let results = connection.query("SELECT * FROM users").await?;
    println!("Query results: {:?}", results);
    
    Ok(())
}
```

**Key Points:**
- Use `async fn` for factory methods that require async initialization
- Return `Arc<dyn Trait>` for shared ownership across async tasks
- Use `#[async_trait]` for async trait methods
- Handle connection failures gracefully
- Consider connection pooling for production use

---

## Testing Factories

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_transport_factory_creates_correct_types() {
        let truck = TransportFactory::create(TransportType::Truck);
        assert!(truck.deliver("NYC").contains("truck"));
        
        let ship = TransportFactory::create(TransportType::Ship);
        assert!(ship.deliver("London").contains("ship"));
    }
    
    #[test]
    fn test_transport_factory_from_string() {
        let result = TransportFactory::create_from_str("truck");
        assert!(result.is_ok());
        
        let result = TransportFactory::create_from_str("invalid");
        assert!(result.is_err());
    }
    
    #[test]
    fn test_custom_truck_capacity() {
        let truck = TransportFactory::create_truck(100);
        assert_eq!(truck.cost(), 5000.0); // 100 tons * 50.0
    }
    
    #[test]
    fn test_payment_gateway_enum_factory() {
        let processor = PaymentGateway::Stripe.create_processor();
        // Processor created successfully
        
        let from_str = PaymentGateway::from_str("paypal");
        assert_eq!(from_str, Some(PaymentGateway::PayPal));
    }
    
    #[test]
    fn test_notification_registry() {
        let factory = NotificationFactory::default();
        
        // Check built-in types registered
        let types = factory.list_types();
        assert!(types.contains(&"email".to_string()));
        assert!(types.contains(&"sms".to_string()));
        
        // Create notification
        let config = NotificationConfig {
            api_key: Some("test_key".to_string()),
            endpoint: None,
            timeout_secs: 30,
        };
        
        let result = factory.create("email", &config);
        assert!(result.is_ok());
        
        let result = factory.create("unknown", &config);
        assert!(result.is_err());
    }
}
```

### Mock Factories for Testing

```rust
#[cfg(test)]
mod test_utils {
    use super::*;
    use std::sync::{Arc, Mutex};
    
    /// Mock transport for testing.
    #[derive(Debug, Clone)]
    pub struct MockTransport {
        pub deliveries: Arc<Mutex<Vec<String>>>,
    }
    
    impl Transport for MockTransport {
        fn deliver(&self, destination: &str) -> String {
            let mut deliveries = self.deliveries.lock().unwrap();
            deliveries.push(destination.to_string());
            format!("Mock delivery to {}", destination)
        }
        
        fn cost(&self) -> f64 {
            0.0 // Free in tests
        }
    }
    
    /// Mock transport factory.
    pub struct MockTransportFactory {
        pub deliveries: Arc<Mutex<Vec<String>>>,
    }
    
    impl MockTransportFactory {
        pub fn new() -> Self {
            Self {
                deliveries: Arc::new(Mutex::new(Vec::new())),
            }
        }
        
        pub fn create(&self) -> Box<dyn Transport> {
            Box::new(MockTransport {
                deliveries: Arc::clone(&self.deliveries),
            })
        }
        
        pub fn get_deliveries(&self) -> Vec<String> {
            self.deliveries.lock().unwrap().clone()
        }
    }
}

#[cfg(test)]
mod integration_tests {
    use super::test_utils::*;
    
    #[test]
    fn test_logistics_with_mock_factory() {
        let factory = MockTransportFactory::new();
        let transport = factory.create();
        
        transport.deliver("New York");
        transport.deliver("Los Angeles");
        
        let deliveries = factory.get_deliveries();
        assert_eq!(deliveries.len(), 2);
        assert_eq!(deliveries[0], "New York");
        assert_eq!(deliveries[1], "Los Angeles");
    }
}
```

### Async Factory Tests

```rust
#[cfg(test)]
mod async_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_database_factory_creates_connection() {
        let config = DatabaseConfig {
            db_type: DatabaseType::Postgres,
            host: "localhost".to_string(),
            port: 5432,
            username: "test".to_string(),
            password: "test".to_string(),
        };
        
        let result = DatabaseConnectionFactory::create(config).await;
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_database_factory_query() {
        let config = DatabaseConfig {
            db_type: DatabaseType::MySql,
            host: "localhost".to_string(),
            port: 3306,
            username: "test".to_string(),
            password: "test".to_string(),
        };
        
        let connection = DatabaseConnectionFactory::create(config).await.unwrap();
        let results = connection.query("SELECT 1").await.unwrap();
        assert!(!results.is_empty());
    }
}
```

---

## Common Patterns

### Pattern 1: Factory with Builder

**Combine factory pattern with builder for complex configuration.**

```rust
pub struct HttpClientFactory;

impl HttpClientFactory {
    pub fn builder() -> HttpClientBuilder {
        HttpClientBuilder::default()
    }
    
    pub fn create_default() -> Box<dyn HttpClient> {
        Self::builder().build()
    }
}

#[derive(Default)]
pub struct HttpClientBuilder {
    timeout: Option<Duration>,
    max_redirects: Option<usize>,
    user_agent: Option<String>,
}

impl HttpClientBuilder {
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }
    
    pub fn max_redirects(mut self, max: usize) -> Self {
        self.max_redirects = Some(max);
        self
    }
    
    pub fn user_agent(mut self, ua: impl Into<String>) -> Self {
        self.user_agent = Some(ua.into());
        self
    }
    
    pub fn build(self) -> Box<dyn HttpClient> {
        // Create and configure client
        todo!()
    }
}
```

### Pattern 2: Factory with Presets

**Provide common configurations as factory methods.**

```rust
impl LoggerFactory {
    pub fn production() -> Arc<dyn Logger> {
        Self::new(LoggerConfig {
            environment: "production".to_string(),
            log_path: Some("/var/log/app.log".to_string()),
            min_level: LogLevel::Info,
        }).create_logger()
    }
    
    pub fn development() -> Arc<dyn Logger> {
        Self::new(LoggerConfig {
            environment: "development".to_string(),
            log_path: None,
            min_level: LogLevel::Debug,
        }).create_logger()
    }
    
    pub fn test() -> Arc<dyn Logger> {
        Self::new(LoggerConfig {
            environment: "test".to_string(),
            log_path: None,
            min_level: LogLevel::Warn,
        }).create_logger()
    }
}
```

### Pattern 3: Lazy Factory

**Create products on-demand and cache them.**

```rust
use std::cell::RefCell;

pub struct LazyConnectionFactory {
    cache: RefCell<HashMap<String, Arc<dyn DatabaseConnection>>>,
}

impl LazyConnectionFactory {
    pub fn new() -> Self {
        Self {
            cache: RefCell::new(HashMap::new()),
        }
    }
    
    pub async fn get_connection(&self, db_name: &str) -> Result<Arc<dyn DatabaseConnection>, String> {
        // Check cache first
        if let Some(conn) = self.cache.borrow().get(db_name) {
            return Ok(Arc::clone(conn));
        }
        
        // Create new connection
        let config = self.load_config(db_name)?;
        let connection = DatabaseConnectionFactory::create(config).await?;
        
        // Cache for future use
        self.cache.borrow_mut().insert(db_name.to_string(), Arc::clone(&connection));
        
        Ok(connection)
    }
    
    fn load_config(&self, db_name: &str) -> Result<DatabaseConfig, String> {
        // Load configuration for database
        todo!()
    }
}
```

---

## Anti-Patterns

### ❌ Anti-Pattern 1: Factory for Simple Types

```rust
// ❌ BAD: Unnecessary factory
pub struct PointFactory;

impl PointFactory {
    pub fn create(x: i32, y: i32) -> Point {
        Point { x, y }
    }
}

// ✅ GOOD: Direct construction
#[derive(Debug, Clone, Copy)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

let point = Point { x: 10, y: 20 };
```

### ❌ Anti-Pattern 2: Returning Option Instead of Result

```rust
// ❌ BAD: Silent failure with Option
impl TransportFactory {
    pub fn create_from_str(s: &str) -> Option<Box<dyn Transport>> {
        match s {
            "truck" => Some(Box::new(Truck::default())),
            _ => None, // Why did it fail?
        }
    }
}

// ✅ GOOD: Explicit error with Result
impl TransportFactory {
    pub fn create_from_str(s: &str) -> Result<Box<dyn Transport>, TransportFactoryError> {
        match s {
            "truck" => Ok(Box::new(Truck::default())),
            _ => Err(TransportFactoryError::UnknownType(s.to_string())),
        }
    }
}
```

### ❌ Anti-Pattern 3: God Factory

```rust
// ❌ BAD: Factory creates everything
pub struct GodFactory;

impl GodFactory {
    pub fn create(type_name: &str) -> Box<dyn Any> {
        match type_name {
            "user" => Box::new(User::default()),
            "order" => Box::new(Order::default()),
            "product" => Box::new(Product::default()),
            "payment" => Box::new(Payment::default()),
            // ... 50 more types
            _ => panic!("Unknown type"),
        }
    }
}

// ✅ GOOD: Separate domain factories
pub struct UserFactory;
impl UserFactory {
    pub fn create_user() -> User { /* ... */ }
    pub fn create_admin() -> Admin { /* ... */ }
}

pub struct OrderFactory;
impl OrderFactory {
    pub fn create_order() -> Order { /* ... */ }
    pub fn create_quote() -> Quote { /* ... */ }
}
```

### ❌ Anti-Pattern 4: Stateful Factory with Side Effects

```rust
// ❌ BAD: Factory with mutable state
pub struct StatefulFactory {
    counter: AtomicUsize,
}

impl StatefulFactory {
    pub fn create(&self) -> Product {
        let id = self.counter.fetch_add(1, Ordering::SeqCst);
        Product::new(id) // Unpredictable behavior
    }
}

// ✅ GOOD: Stateless factory, ID generation elsewhere
pub struct ProductFactory;

impl ProductFactory {
    pub fn create(id: ProductId) -> Product {
        Product::new(id)
    }
}

// ID generation is separate concern
pub struct ProductIdGenerator {
    counter: AtomicUsize,
}

impl ProductIdGenerator {
    pub fn next(&self) -> ProductId {
        ProductId(self.counter.fetch_add(1, Ordering::SeqCst))
    }
}
```

### ❌ Anti-Pattern 5: Ignoring Rust's Type System

```rust
// ❌ BAD: Stringly-typed factory
pub fn create_processor(type_str: &str) -> Box<dyn Processor> {
    match type_str {
        "stripe" => Box::new(StripeProcessor),
        "paypal" => Box::new(PayPalProcessor),
        _ => panic!(),
    }
}

// ✅ GOOD: Use enum for type safety
pub enum ProcessorType {
    Stripe,
    PayPal,
}

pub fn create_processor(processor_type: ProcessorType) -> Box<dyn Processor> {
    match processor_type {
        ProcessorType::Stripe => Box::new(StripeProcessor),
        ProcessorType::PayPal => Box::new(PayPalProcessor),
    }
}
```

---

## Conclusion

### Key Takeaways

1. **When to Use Factories in Rust:**
   - Multiple product types with shared trait
   - Complex initialization logic
   - Runtime type selection
   - Resource pooling

2. **Rust-Specific Patterns:**
   - Use enums for closed product sets
   - Trait objects (`Box<dyn Trait>`) for polymorphism
   - `Arc<dyn Trait>` for shared ownership
   - Type parameters for zero-cost abstractions

3. **Factory Variants:**
   - **Simple Factory:** Static functions on struct
   - **Trait-Based:** Factory Method pattern with traits
   - **Enum-Based:** Type-safe product selection
   - **Registry-Based:** Runtime extensibility (plugins)
   - **Abstract Factory:** Product families

4. **Error Handling:**
   - Return `Result<T, E>` not `Option<T>`
   - Use custom error types with `thiserror`
   - Provide clear error messages

5. **Async Considerations:**
   - Use `async fn` for async initialization
   - `Arc<dyn Trait>` for shared async ownership
   - `#[async_trait]` for async trait methods

### Quick Reference Checklist

**Factory Implementation:**
- [ ] Return trait objects (`Box<dyn Trait>`) for polymorphism
- [ ] Use enums for closed product sets
- [ ] Return `Result<T, E>` for fallible creation
- [ ] Keep factories stateless when possible
- [ ] Document supported product types
- [ ] Provide clear error messages
- [ ] Consider `Arc` instead of `Box` for shared ownership
- [ ] Use type parameters for zero-cost abstractions
- [ ] Test factory with all product types

**Product Types:**
- [ ] Implement common traits (Debug, Clone, etc.)
- [ ] Design around trait interfaces
- [ ] Keep product types independent
- [ ] Document product-specific behavior

---

## Further Reading

### Rust Resources
- [Rust Design Patterns - Creational Patterns](https://rust-unofficial.github.io/patterns/patterns/creational/index.html)
- [Trait Objects](https://doc.rust-lang.org/book/ch17-02-trait-objects.html)
- [Advanced Traits](https://doc.rust-lang.org/book/ch19-03-advanced-traits.html)
- [`async-trait` crate](https://docs.rs/async-trait/)

### Related Patterns
- [Builder Pattern Guide](./rust-builder-pattern-guide.md)
- [ADT Implementation Guide](./rust-adt-implementation-guide.md)
- [Factory Pattern Architecture](../../architecture/builder-pattern.md)

---

**Document Version:** 1.0  
**Last Updated:** 2026-01-25  
**Status:** Active
