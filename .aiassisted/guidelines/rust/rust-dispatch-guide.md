# Rust Static and Dynamic Dispatch Implementation Guide

> **Comprehensive guide to dispatch mechanisms in Rust**

**Status:** Complete  
**Assumed Knowledge:** Intermediate Rust (traits, generics, ownership, lifetimes)  
**Related Documents:**
- [Rust ADT Implementation Guide](./rust-adt-implementation-guide.md)
- [Rust Factory Pattern Guide](./rust-factory-pattern-guide.md)
- [Rust Dependency Management Guide](./rust-dependency-management-guide.md)

---

## Table of Contents

1. [Overview](#1-overview)
2. [When to Use Static vs Dynamic Dispatch](#2-when-to-use-static-vs-dynamic-dispatch)
3. [Core Concepts](#3-core-concepts)
4. [Pattern 1: Static Dispatch with Monomorphization](#4-pattern-1-static-dispatch-with-monomorphization)
5. [Pattern 2: Dynamic Dispatch with Trait Objects](#5-pattern-2-dynamic-dispatch-with-trait-objects)
6. [Pattern 3: Hybrid Dispatch Strategies](#6-pattern-3-hybrid-dispatch-strategies)
7. [Pattern 4: Async Trait Dispatch](#7-pattern-4-async-trait-dispatch)
8. [Pattern 5: Object Safety and Sized Bounds](#8-pattern-5-object-safety-and-sized-bounds)
9. [Performance Characteristics](#9-performance-characteristics)
10. [Testing Strategies](#10-testing-strategies)
11. [Anti-Patterns](#11-anti-patterns)
12. [Conclusion](#12-conclusion)

---

## 1. Overview

### What is Dispatch?

**Dispatch** is the mechanism by which the compiler or runtime determines which specific function or method implementation to call. In Rust, dispatch is closely tied to how polymorphism is implemented through traits and generics.

### Two Forms of Dispatch

1. **Static Dispatch**: Function calls resolved at compile time
2. **Dynamic Dispatch**: Function calls resolved at runtime through vtables

### Key Distinctions

```rust
// Static dispatch - monomorphization
fn process<T: Processor>(item: T) {
    item.process(); // Resolved at compile time
}

// Dynamic dispatch - trait objects
fn process_dyn(item: &dyn Processor) {
    item.process(); // Resolved at runtime via vtable
}
```

### Why This Matters

- **Performance**: Static dispatch enables inlining and aggressive optimization
- **Code Size**: Dynamic dispatch reduces binary bloat from monomorphization
- **Flexibility**: Dynamic dispatch allows heterogeneous collections
- **API Design**: Choice affects trait design, object safety, and ergonomics

### Document Structure

This guide covers:
- Core dispatch mechanisms and their trade-offs
- Implementation patterns for both approaches
- Async trait dispatch challenges and solutions
- Performance implications and optimization strategies
- Object safety rules and workarounds
- Testing and validation techniques
- Common pitfalls and anti-patterns

---

## 2. When to Use Static vs Dynamic Dispatch

### Use Static Dispatch When ✅

**Scenario 1: Performance-Critical Code**
```rust
// Hot path in computational kernel
fn transform<T: Transform>(data: &mut [T]) {
    for item in data {
        item.transform(); // Zero-cost abstraction
    }
}
```
✅ **Why**: Eliminates indirect calls, enables inlining, allows LLVM optimizations

**Scenario 2: Known Types at Compile Time**
```rust
// Generic function with concrete types known at call site
fn serialize<T: Serialize>(value: T) -> Vec<u8> {
    // Compiler generates specialized version for each T
    value.to_bytes()
}
```
✅ **Why**: No need for runtime flexibility, optimization opportunities

**Scenario 3: Library APIs**
```rust
// Public API that benefits from monomorphization
pub fn process_items<I, T>(items: I) 
where
    I: IntoIterator<Item = T>,
    T: Processable,
{
    for item in items {
        item.process();
    }
}
```
✅ **Why**: Allows callers to use any iterator type, zero runtime cost

**Scenario 4: Zero-Cost Abstractions**
```rust
// Iterator chains - all static dispatch
let result: Vec<_> = data
    .iter()
    .filter(|x| x.is_valid())
    .map(|x| x.transform())
    .collect();
```
✅ **Why**: Entire chain optimized away to tight loop

**Scenario 5: Embedded Systems**
```rust
// No_std environment with strict memory constraints
#![no_std]

trait Sensor {
    fn read(&self) -> u16;
}

fn poll_sensors<S: Sensor>(sensors: &[S]) {
    // Static dispatch, no heap allocation
}
```
✅ **Why**: No dynamic allocation, predictable binary size

### Use Dynamic Dispatch When ✅

**Scenario 1: Heterogeneous Collections**
```rust
// Collection of different types implementing same trait
struct EventBus {
    handlers: Vec<Box<dyn EventHandler>>,
}

impl EventBus {
    fn dispatch(&self, event: &Event) {
        for handler in &self.handlers {
            handler.handle(event);
        }
    }
}
```
✅ **Why**: Cannot know all types at compile time

**Scenario 2: Plugin Systems**
```rust
// Dynamically loaded plugins
trait Plugin {
    fn execute(&self, context: &Context) -> Result<()>;
}

struct PluginManager {
    plugins: Vec<Box<dyn Plugin>>,
}

impl PluginManager {
    fn load_plugin(&mut self, plugin: Box<dyn Plugin>) {
        self.plugins.push(plugin);
    }
}
```
✅ **Why**: Plugins loaded at runtime, types unknown at compile time

**Scenario 3: Reducing Binary Size**
```rust
// Large trait with many implementors
trait ComplexProcessor {
    fn step1(&self);
    fn step2(&self);
    fn step3(&self);
    // ... 20 more methods
}

// Dynamic dispatch prevents 20+ monomorphized copies
fn process_all(processors: &[Box<dyn ComplexProcessor>]) {
    for p in processors {
        p.step1();
        p.step2();
        // ...
    }
}
```
✅ **Why**: Avoids code bloat from monomorphization

**Scenario 4: API Stability**
```rust
// Public API using trait objects for ABI stability
pub struct Service {
    backend: Box<dyn Backend>,
}

impl Service {
    pub fn new(backend: Box<dyn Backend>) -> Self {
        Self { backend }
    }
}
```
✅ **Why**: Trait object layout stable across versions

**Scenario 5: Recursive Types**
```rust
// Tree structure with heterogeneous nodes
trait Node {
    fn evaluate(&self) -> i32;
}

struct BranchNode {
    children: Vec<Box<dyn Node>>,
}

impl Node for BranchNode {
    fn evaluate(&self) -> i32 {
        self.children.iter().map(|c| c.evaluate()).sum()
    }
}
```
✅ **Why**: Enables recursive structures with different types

### Avoid Static Dispatch When ❌

❌ **Binary size constraints with many types**
```rust
// BAD: Monomorphizes 100+ versions
fn process<T: Trait>(item: T) { /* large function */ }

// Called with 100 different types
```

❌ **Need runtime polymorphism**
```rust
// BAD: Cannot store different types
let items: Vec<T> = vec![Type1, Type2]; // ERROR: incompatible types
```

❌ **Type unknown until runtime**
```rust
// BAD: Type determined by configuration file
match config.processor_type {
    "A" => process::<TypeA>(), // Cannot express this pattern
    "B" => process::<TypeB>(),
}
```

### Avoid Dynamic Dispatch When ❌

❌ **Performance-critical tight loops**
```rust
// BAD: Virtual call overhead in hot path
for item in large_dataset {
    processor.process(item); // Indirect call every iteration
}
```

❌ **Async trait methods (without workarounds)**
```rust
// BAD: Not object-safe by default
trait AsyncProcessor {
    async fn process(&self); // ERROR: method `process` is not object-safe
}
```

❌ **Need generic associated types**
```rust
// BAD: Not object-safe
trait Container {
    type Item<'a> where Self: 'a;
    fn get<'a>(&'a self) -> Self::Item<'a>;
}

// Cannot create &dyn Container
```

### Decision Matrix

| Factor | Static Dispatch | Dynamic Dispatch |
|--------|----------------|------------------|
| **Performance** | ⭐⭐⭐⭐⭐ Fast | ⭐⭐⭐ Slight overhead |
| **Binary Size** | ⭐⭐ Can bloat | ⭐⭐⭐⭐ Smaller |
| **Compile Time** | ⭐⭐ Slower | ⭐⭐⭐⭐ Faster |
| **Flexibility** | ⭐⭐⭐ Compile-time only | ⭐⭐⭐⭐⭐ Runtime |
| **Optimization** | ⭐⭐⭐⭐⭐ Aggressive | ⭐⭐⭐ Limited |
| **Heterogeneous Collections** | ❌ No | ✅ Yes |
| **Plugin Systems** | ❌ No | ✅ Yes |

---

## 3. Core Concepts

### 3.1 Monomorphization

**What It Is**: The process of generating specialized versions of generic functions for each concrete type used.

```rust
// Generic function
fn max<T: PartialOrd>(a: T, b: T) -> T {
    if a > b { a } else { b }
}

// Usage
let x = max(10, 20);       // Generates max_i32
let y = max(3.14, 2.71);   // Generates max_f64
let z = max('a', 'z');     // Generates max_char

// Compiler generates (conceptually):
fn max_i32(a: i32, b: i32) -> i32 {
    if a > b { a } else { b }
}

fn max_f64(a: f64, b: f64) -> f64 {
    if a > b { a } else { b }
}

fn max_char(a: char, b: char) -> char {
    if a > b { a } else { b }
}
```

**Implications**:
1. Each type gets optimized version
2. Binary contains multiple copies
3. Compiler can inline and specialize
4. Compile time increases

### 3.2 Trait Objects and Fat Pointers

**Fat Pointer Structure**: A trait object consists of two pointers:

```rust
use std::mem;

trait Drawable {
    fn draw(&self);
}

struct Circle { radius: f64 }
impl Drawable for Circle {
    fn draw(&self) { println!("Circle: {}", self.radius); }
}

fn main() {
    let circle = Circle { radius: 5.0 };
    
    // Regular reference: 8 bytes (on 64-bit)
    let r: &Circle = &circle;
    println!("Reference size: {}", mem::size_of_val(&r)); // 8
    
    // Trait object: 16 bytes (on 64-bit)
    let d: &dyn Drawable = &circle;
    println!("Trait object size: {}", mem::size_of_val(&d)); // 16
    
    // d contains:
    // 1. Pointer to data (Circle instance)
    // 2. Pointer to vtable (method implementations)
}
```

**Vtable Structure**: Virtual method table containing:

```rust
// Conceptual vtable structure
struct DrawableVtable {
    destructor: unsafe fn(*mut ()),
    size: usize,
    align: usize,
    draw: unsafe fn(*const ()),
}

// For Circle
static CIRCLE_DRAWABLE_VTABLE: DrawableVtable = DrawableVtable {
    destructor: /* compiler magic */,
    size: mem::size_of::<Circle>(),
    align: mem::align_of::<Circle>(),
    draw: call_circle_draw as unsafe fn(*const ()),
};

unsafe fn call_circle_draw(ptr: *const ()) {
    let circle = &*(ptr as *const Circle);
    circle.draw();
}
```

### 3.3 Object Safety Rules

**Object-Safe Trait**: Can be used as trait object (`dyn Trait`)

**Requirements**:
1. All methods must have `Self: Sized` bound OR:
   - No `Self` in return position (except in `&self`, `&mut self`, `Box<Self>`, etc.)
   - No generic type parameters
   - First parameter must be `&self`, `&mut self`, `self`, `Box<Self>`, `Rc<Self>`, `Arc<Self>`, or `Pin<P>` where `P` is one of the above

2. Trait must not require `Self: Sized`
3. No associated constants
4. No associated types with generic parameters (GATs)

```rust
// Object-safe trait
trait ObjectSafe {
    fn method(&self) -> i32;
    fn method_mut(&mut self);
    fn method_consume(self: Box<Self>);
}

// NOT object-safe traits
trait NotObjectSafe1 {
    fn returns_self(&self) -> Self; // ERROR: Self in return position
}

trait NotObjectSafe2 {
    fn generic_method<T>(&self, value: T); // ERROR: generic method
}

trait NotObjectSafe3: Sized { // ERROR: requires Sized
    fn method(&self);
}

trait NotObjectSafe4 {
    const VALUE: i32; // ERROR: associated constant
}
```

### 3.4 Smart Pointers and Trait Objects

**Common Patterns**:

```rust
trait Service {
    fn execute(&self) -> Result<String, Error>;
}

// Reference - borrowed trait object
fn use_reference(service: &dyn Service) {
    // Most common, no allocation
    service.execute();
}

// Box - owned trait object
fn use_box(service: Box<dyn Service>) {
    // Heap allocation, single ownership
    service.execute();
}

// Rc - shared ownership, single-threaded
use std::rc::Rc;
fn use_rc(service: Rc<dyn Service>) {
    // Reference counted, not Send
    service.execute();
}

// Arc - shared ownership, thread-safe
use std::sync::Arc;
fn use_arc(service: Arc<dyn Service>) {
    // Atomic reference counted, Send + Sync
    service.execute();
}

// Pin - for self-referential types
use std::pin::Pin;
fn use_pin(service: Pin<Box<dyn Service>>) {
    // Cannot move, useful for async
    service.execute();
}
```

### 3.5 Type Erasure

**Concept**: Hiding concrete type behind trait interface

```rust
trait Storage {
    fn store(&mut self, data: Vec<u8>);
    fn retrieve(&self) -> Vec<u8>;
}

struct MemoryStorage {
    buffer: Vec<u8>,
}

impl Storage for MemoryStorage {
    fn store(&mut self, data: Vec<u8>) {
        self.buffer = data;
    }
    
    fn retrieve(&self) -> Vec<u8> {
        self.buffer.clone()
    }
}

struct FileStorage {
    path: PathBuf,
}

impl Storage for FileStorage {
    fn store(&mut self, data: Vec<u8>) {
        std::fs::write(&self.path, data).unwrap();
    }
    
    fn retrieve(&self) -> Vec<u8> {
        std::fs::read(&self.path).unwrap()
    }
}

// Type-erased container
struct DataStore {
    backend: Box<dyn Storage>,
}

impl DataStore {
    fn new(backend: Box<dyn Storage>) -> Self {
        Self { backend }
    }
    
    fn save(&mut self, data: Vec<u8>) {
        // Concrete type hidden, dispatch via vtable
        self.backend.store(data);
    }
}

fn main() {
    // Type erased at construction
    let mut store = DataStore::new(Box::new(MemoryStorage { buffer: vec![] }));
    store.save(vec![1, 2, 3]);
    
    // Can swap backend at runtime
    let mut store = DataStore::new(Box::new(FileStorage { path: "data.bin".into() }));
    store.save(vec![4, 5, 6]);
}
```

### 3.6 Inlining and Optimization

**Static Dispatch Optimization**:

```rust
// Small function marked for inlining
#[inline]
fn add<T: std::ops::Add<Output = T>>(a: T, b: T) -> T {
    a + b
}

// Aggressive inlining
#[inline(always)]
fn fast_path<T: FastOp>(value: T) -> T {
    value.fast_operation()
}

// Compiler can optimize to:
let x = 5;
let y = 10;
let result = x + y; // Direct addition, no function call
```

**Dynamic Dispatch Cannot Inline**:

```rust
fn process(op: &dyn Operation) {
    // Cannot be inlined - must go through vtable
    op.execute();
}
```

**Inline Attributes**:
- `#[inline]`: Suggest inlining (compiler may ignore)
- `#[inline(always)]`: Force inlining (use sparingly)
- `#[inline(never)]`: Prevent inlining

---

## 4. Pattern 1: Static Dispatch with Monomorphization

### 4.1 Basic Static Dispatch

**Simple Generic Function**:

```rust
trait Validator {
    fn validate(&self) -> bool;
}

struct EmailValidator {
    email: String,
}

impl Validator for EmailValidator {
    fn validate(&self) -> bool {
        self.email.contains('@')
    }
}

struct AgeValidator {
    age: u32,
}

impl Validator for AgeValidator {
    fn validate(&self) -> bool {
        self.age >= 18 && self.age <= 120
    }
}

// Static dispatch - monomorphized for each type
fn validate_input<V: Validator>(validator: &V) -> Result<(), String> {
    if validator.validate() {
        Ok(())
    } else {
        Err("Validation failed".to_string())
    }
}

fn main() {
    let email = EmailValidator { email: "user@example.com".to_string() };
    let age = AgeValidator { age: 25 };
    
    // Two specialized versions generated
    validate_input(&email).unwrap();
    validate_input(&age).unwrap();
}
```

### 4.2 Multiple Trait Bounds

**Complex Constraints**:

```rust
use std::fmt::Display;

trait Serialize {
    fn to_bytes(&self) -> Vec<u8>;
}

trait Compress {
    fn compress(&self, data: Vec<u8>) -> Vec<u8>;
}

// Multiple trait bounds
fn save<T>(item: &T) -> Vec<u8>
where
    T: Serialize + Display + Clone,
{
    println!("Saving: {}", item);
    let data = item.to_bytes();
    println!("Serialized {} bytes", data.len());
    data
}

// Shorthand syntax
fn save_compact<T: Serialize + Display + Clone>(item: &T) -> Vec<u8> {
    item.to_bytes()
}
```

### 4.3 Associated Types with Static Dispatch

**Type Families**:

```rust
trait Parser {
    type Output;
    type Error;
    
    fn parse(&self, input: &str) -> Result<Self::Output, Self::Error>;
}

struct JsonParser;
struct XmlParser;

#[derive(Debug)]
struct JsonValue(String);

#[derive(Debug)]
struct XmlNode(String);

impl Parser for JsonParser {
    type Output = JsonValue;
    type Error = String;
    
    fn parse(&self, input: &str) -> Result<JsonValue, String> {
        Ok(JsonValue(input.to_string()))
    }
}

impl Parser for XmlParser {
    type Output = XmlNode;
    type Error = String;
    
    fn parse(&self, input: &str) -> Result<XmlNode, String> {
        Ok(XmlNode(input.to_string()))
    }
}

// Static dispatch with associated types
fn parse_and_log<P: Parser>(parser: &P, input: &str)
where
    P::Output: std::fmt::Debug,
    P::Error: std::fmt::Display,
{
    match parser.parse(input) {
        Ok(output) => println!("Parsed: {:?}", output),
        Err(e) => eprintln!("Error: {}", e),
    }
}

fn main() {
    let json = JsonParser;
    let xml = XmlParser;
    
    parse_and_log(&json, "{}");
    parse_and_log(&xml, "<root/>");
}
```

### 4.4 Generic Struct with Static Dispatch

**Type-Parameterized Containers**:

```rust
trait Cache {
    type Key;
    type Value;
    
    fn get(&self, key: &Self::Key) -> Option<&Self::Value>;
    fn insert(&mut self, key: Self::Key, value: Self::Value);
}

// Generic cache implementation
struct MemoryCache<K, V> {
    map: std::collections::HashMap<K, V>,
}

impl<K, V> MemoryCache<K, V>
where
    K: std::hash::Hash + Eq,
{
    fn new() -> Self {
        Self {
            map: std::collections::HashMap::new(),
        }
    }
}

impl<K, V> Cache for MemoryCache<K, V>
where
    K: std::hash::Hash + Eq,
{
    type Key = K;
    type Value = V;
    
    fn get(&self, key: &K) -> Option<&V> {
        self.map.get(key)
    }
    
    fn insert(&mut self, key: K, value: V) {
        self.map.insert(key, value);
    }
}

// Generic function using cache
fn cached_computation<C>(cache: &mut C, key: C::Key, compute: impl FnOnce() -> C::Value) -> C::Value
where
    C: Cache,
    C::Key: Clone,
    C::Value: Clone,
{
    if let Some(value) = cache.get(&key) {
        return value.clone();
    }
    
    let value = compute();
    cache.insert(key.clone(), value.clone());
    value
}

fn main() {
    let mut cache = MemoryCache::<String, i32>::new();
    
    let result = cached_computation(&mut cache, "answer".to_string(), || {
        println!("Computing...");
        42
    });
    
    println!("Result: {}", result);
    
    // Second call uses cache
    let result = cached_computation(&mut cache, "answer".to_string(), || {
        println!("Computing...");
        42
    });
    
    println!("Result: {}", result);
}
```

### 4.5 Iterator-Based Static Dispatch

**Zero-Cost Iterator Chains**:

```rust
trait Processor {
    fn process(&self, value: i32) -> i32;
}

struct Doubler;
impl Processor for Doubler {
    fn process(&self, value: i32) -> i32 {
        value * 2
    }
}

struct Incrementer;
impl Processor for Incrementer {
    fn process(&self, value: i32) -> i32 {
        value + 1
    }
}

// Static dispatch with iterator adapters
fn process_stream<P, I>(processor: P, items: I) -> Vec<i32>
where
    P: Processor,
    I: IntoIterator<Item = i32>,
{
    items
        .into_iter()
        .map(|item| processor.process(item))
        .collect()
}

// Complex iterator chain - all static dispatch
fn complex_pipeline<I>(data: I) -> i32
where
    I: IntoIterator<Item = i32>,
{
    data.into_iter()
        .filter(|x| x % 2 == 0)
        .map(|x| x * x)
        .take(10)
        .sum()
}

fn main() {
    let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    
    let doubled = process_stream(Doubler, data.clone());
    println!("Doubled: {:?}", doubled);
    
    let sum = complex_pipeline(data);
    println!("Sum: {}", sum);
}
```

### 4.6 Conditional Compilation with Static Dispatch

**Feature-Dependent Implementations**:

```rust
trait Logger {
    fn log(&self, message: &str);
}

#[cfg(debug_assertions)]
struct DebugLogger;

#[cfg(debug_assertions)]
impl Logger for DebugLogger {
    fn log(&self, message: &str) {
        eprintln!("[DEBUG] {}", message);
    }
}

#[cfg(not(debug_assertions))]
struct ProductionLogger;

#[cfg(not(debug_assertions))]
impl Logger for ProductionLogger {
    fn log(&self, message: &str) {
        // Write to file or send to server
        println!("[INFO] {}", message);
    }
}

// Compile-time selection
fn create_logger() -> impl Logger {
    #[cfg(debug_assertions)]
    {
        DebugLogger
    }
    
    #[cfg(not(debug_assertions))]
    {
        ProductionLogger
    }
}

fn process_with_logging<L: Logger>(logger: &L) {
    logger.log("Processing started");
    // Do work...
    logger.log("Processing completed");
}

fn main() {
    let logger = create_logger();
    process_with_logging(&logger);
}
```

### 4.7 Real-World Example: HTTP Client

**Static Dispatch in Action**:

```rust
use std::error::Error;
use std::fmt;

// Custom error type
#[derive(Debug)]
struct HttpError(String);

impl fmt::Display for HttpError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "HTTP Error: {}", self.0)
    }
}

impl Error for HttpError {}

// Trait for HTTP methods
trait HttpMethod {
    fn name(&self) -> &'static str;
}

struct Get;
struct Post;
struct Put;
struct Delete;

impl HttpMethod for Get {
    fn name(&self) -> &'static str { "GET" }
}

impl HttpMethod for Post {
    fn name(&self) -> &'static str { "POST" }
}

impl HttpMethod for Put {
    fn name(&self) -> &'static str { "PUT" }
}

impl HttpMethod for Delete {
    fn name(&self) -> &'static str { "DELETE" }
}

// Trait for request builders
trait RequestBuilder {
    fn with_header(self, key: &str, value: &str) -> Self;
    fn with_body(self, body: Vec<u8>) -> Self;
    fn build(self) -> Request;
}

#[derive(Debug, Clone)]
struct Request {
    method: String,
    url: String,
    headers: Vec<(String, String)>,
    body: Option<Vec<u8>>,
}

struct TypedRequestBuilder<M: HttpMethod> {
    method: M,
    url: String,
    headers: Vec<(String, String)>,
    body: Option<Vec<u8>>,
}

impl<M: HttpMethod> TypedRequestBuilder<M> {
    fn new(method: M, url: String) -> Self {
        Self {
            method,
            url,
            headers: vec![],
            body: None,
        }
    }
}

impl<M: HttpMethod> RequestBuilder for TypedRequestBuilder<M> {
    fn with_header(mut self, key: &str, value: &str) -> Self {
        self.headers.push((key.to_string(), value.to_string()));
        self
    }
    
    fn with_body(mut self, body: Vec<u8>) -> Self {
        self.body = Some(body);
        self
    }
    
    fn build(self) -> Request {
        Request {
            method: self.method.name().to_string(),
            url: self.url,
            headers: self.headers,
            body: self.body,
        }
    }
}

// Generic execute function - static dispatch
fn execute<M: HttpMethod, B: RequestBuilder>(builder: B) -> Result<String, Box<dyn Error>> {
    let request = builder.build();
    println!("Executing: {} {}", request.method, request.url);
    
    for (key, value) in &request.headers {
        println!("  {}: {}", key, value);
    }
    
    if let Some(body) = &request.body {
        println!("  Body: {} bytes", body.len());
    }
    
    Ok("Success".to_string())
}

fn main() -> Result<(), Box<dyn Error>> {
    // Each call monomorphized to specific method type
    let get_request = TypedRequestBuilder::new(Get, "https://api.example.com/users".to_string())
        .with_header("Accept", "application/json")
        .build();
    
    let post_request = TypedRequestBuilder::new(Post, "https://api.example.com/users".to_string())
        .with_header("Content-Type", "application/json")
        .with_body(b"{\"name\":\"John\"}".to_vec())
        .build();
    
    println!("GET: {:?}", get_request);
    println!("POST: {:?}", post_request);
    
    Ok(())
}
```

---

## 5. Pattern 2: Dynamic Dispatch with Trait Objects

### 5.1 Basic Trait Objects

**Using `dyn Trait`**:

```rust
trait Shape {
    fn area(&self) -> f64;
    fn name(&self) -> &str;
}

struct Circle {
    radius: f64,
}

impl Shape for Circle {
    fn area(&self) -> f64 {
        std::f64::consts::PI * self.radius * self.radius
    }
    
    fn name(&self) -> &str {
        "Circle"
    }
}

struct Rectangle {
    width: f64,
    height: f64,
}

impl Shape for Rectangle {
    fn area(&self) -> f64 {
        self.width * self.height
    }
    
    fn name(&self) -> &str {
        "Rectangle"
    }
}

// Dynamic dispatch - single function for all shapes
fn print_shape_info(shape: &dyn Shape) {
    println!("{}: area = {:.2}", shape.name(), shape.area());
}

fn main() {
    let circle = Circle { radius: 5.0 };
    let rectangle = Rectangle { width: 4.0, height: 6.0 };
    
    // Both use same function via dynamic dispatch
    print_shape_info(&circle);
    print_shape_info(&rectangle);
    
    // Heterogeneous collection
    let shapes: Vec<&dyn Shape> = vec![&circle, &rectangle];
    
    for shape in shapes {
        print_shape_info(shape);
    }
}
```

### 5.2 Boxed Trait Objects

**Owned Trait Objects**:

```rust
trait Command {
    fn execute(&self) -> Result<String, String>;
    fn describe(&self) -> String;
}

struct CreateUser {
    username: String,
}

impl Command for CreateUser {
    fn execute(&self) -> Result<String, String> {
        println!("Creating user: {}", self.username);
        Ok(format!("User {} created", self.username))
    }
    
    fn describe(&self) -> String {
        format!("CreateUser({})", self.username)
    }
}

struct DeleteUser {
    user_id: u64,
}

impl Command for DeleteUser {
    fn execute(&self) -> Result<String, String> {
        println!("Deleting user: {}", self.user_id);
        Ok(format!("User {} deleted", self.user_id))
    }
    
    fn describe(&self) -> String {
        format!("DeleteUser({})", self.user_id)
    }
}

// Command queue using boxed trait objects
struct CommandQueue {
    commands: Vec<Box<dyn Command>>,
}

impl CommandQueue {
    fn new() -> Self {
        Self { commands: vec![] }
    }
    
    fn enqueue(&mut self, command: Box<dyn Command>) {
        self.commands.push(command);
    }
    
    fn execute_all(&mut self) -> Vec<Result<String, String>> {
        let mut results = vec![];
        
        for command in &self.commands {
            println!("Executing: {}", command.describe());
            results.push(command.execute());
        }
        
        results
    }
}

fn main() {
    let mut queue = CommandQueue::new();
    
    queue.enqueue(Box::new(CreateUser {
        username: "alice".to_string(),
    }));
    
    queue.enqueue(Box::new(DeleteUser {
        user_id: 123,
    }));
    
    queue.enqueue(Box::new(CreateUser {
        username: "bob".to_string(),
    }));
    
    let results = queue.execute_all();
    
    for (i, result) in results.iter().enumerate() {
        match result {
            Ok(msg) => println!("Command {}: {}", i, msg),
            Err(e) => eprintln!("Command {} failed: {}", i, e),
        }
    }
}
```

### 5.3 Arc/Rc Trait Objects

**Shared Ownership with Dynamic Dispatch**:

```rust
use std::sync::Arc;
use std::thread;

trait EventHandler: Send + Sync {
    fn handle(&self, event: &str);
}

struct LogHandler {
    name: String,
}

impl EventHandler for LogHandler {
    fn handle(&self, event: &str) {
        println!("[{}] Event: {}", self.name, event);
    }
}

struct MetricsHandler {
    counter: std::sync::atomic::AtomicUsize,
}

impl EventHandler for MetricsHandler {
    fn handle(&self, event: &str) {
        let count = self.counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        println!("Metrics: event #{} - {}", count + 1, event);
    }
}

// Multi-threaded event dispatcher
struct EventDispatcher {
    handlers: Vec<Arc<dyn EventHandler>>,
}

impl EventDispatcher {
    fn new() -> Self {
        Self { handlers: vec![] }
    }
    
    fn add_handler(&mut self, handler: Arc<dyn EventHandler>) {
        self.handlers.push(handler);
    }
    
    fn dispatch(&self, event: &str) {
        for handler in &self.handlers {
            let handler = Arc::clone(handler);
            let event = event.to_string();
            
            thread::spawn(move || {
                handler.handle(&event);
            });
        }
    }
}

fn main() {
    let mut dispatcher = EventDispatcher::new();
    
    dispatcher.add_handler(Arc::new(LogHandler {
        name: "Handler1".to_string(),
    }));
    
    dispatcher.add_handler(Arc::new(MetricsHandler {
        counter: std::sync::atomic::AtomicUsize::new(0),
    }));
    
    dispatcher.dispatch("user_login");
    dispatcher.dispatch("page_view");
    
    thread::sleep(std::time::Duration::from_millis(100));
}
```

### 5.4 Trait Objects with Lifetimes

**Borrowed Data in Trait Objects**:

```rust
trait Formatter {
    fn format(&self, data: &str) -> String;
}

struct PrefixFormatter<'a> {
    prefix: &'a str,
}

impl<'a> Formatter for PrefixFormatter<'a> {
    fn format(&self, data: &str) -> String {
        format!("{}: {}", self.prefix, data)
    }
}

struct SuffixFormatter<'a> {
    suffix: &'a str,
}

impl<'a> Formatter for SuffixFormatter<'a> {
    fn format(&self, data: &str) -> String {
        format!("{} {}", data, self.suffix)
    }
}

// Trait object with lifetime
fn apply_formatter<'a>(formatter: &'a dyn Formatter, items: &[&str]) -> Vec<String> {
    items.iter()
        .map(|item| formatter.format(item))
        .collect()
}

fn main() {
    let prefix = "LOG";
    let suffix = "[END]";
    
    let items = vec!["Message 1", "Message 2", "Message 3"];
    
    let prefix_formatter = PrefixFormatter { prefix: &prefix };
    let formatted = apply_formatter(&prefix_formatter, &items);
    println!("Prefix: {:?}", formatted);
    
    let suffix_formatter = SuffixFormatter { suffix: &suffix };
    let formatted = apply_formatter(&suffix_formatter, &items);
    println!("Suffix: {:?}", formatted);
}
```

### 5.5 Trait Objects for Plugin Systems

**Runtime Plugin Loading**:

```rust
use std::collections::HashMap;

trait Plugin {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn execute(&self, context: &Context) -> Result<Response, Error>;
}

#[derive(Debug)]
struct Context {
    params: HashMap<String, String>,
}

#[derive(Debug)]
struct Response {
    status: u16,
    body: String,
}

#[derive(Debug)]
struct Error {
    message: String,
}

// Concrete plugins
struct AuthPlugin;

impl Plugin for AuthPlugin {
    fn name(&self) -> &str {
        "auth"
    }
    
    fn version(&self) -> &str {
        "1.0.0"
    }
    
    fn execute(&self, context: &Context) -> Result<Response, Error> {
        if let Some(token) = context.params.get("token") {
            Ok(Response {
                status: 200,
                body: format!("Authenticated with token: {}", token),
            })
        } else {
            Err(Error {
                message: "No token provided".to_string(),
            })
        }
    }
}

struct LoggingPlugin;

impl Plugin for LoggingPlugin {
    fn name(&self) -> &str {
        "logging"
    }
    
    fn version(&self) -> &str {
        "2.1.0"
    }
    
    fn execute(&self, context: &Context) -> Result<Response, Error> {
        println!("Logging request with params: {:?}", context.params);
        Ok(Response {
            status: 200,
            body: "Logged".to_string(),
        })
    }
}

// Plugin registry
struct PluginRegistry {
    plugins: HashMap<String, Box<dyn Plugin>>,
}

impl PluginRegistry {
    fn new() -> Self {
        Self {
            plugins: HashMap::new(),
        }
    }
    
    fn register(&mut self, plugin: Box<dyn Plugin>) {
        let name = plugin.name().to_string();
        println!("Registering plugin: {} v{}", plugin.name(), plugin.version());
        self.plugins.insert(name, plugin);
    }
    
    fn execute(&self, plugin_name: &str, context: &Context) -> Result<Response, Error> {
        if let Some(plugin) = self.plugins.get(plugin_name) {
            plugin.execute(context)
        } else {
            Err(Error {
                message: format!("Plugin not found: {}", plugin_name),
            })
        }
    }
    
    fn list_plugins(&self) {
        println!("Registered plugins:");
        for (name, plugin) in &self.plugins {
            println!("  - {} v{}", name, plugin.version());
        }
    }
}

fn main() {
    let mut registry = PluginRegistry::new();
    
    // Register plugins at runtime
    registry.register(Box::new(AuthPlugin));
    registry.register(Box::new(LoggingPlugin));
    
    registry.list_plugins();
    
    // Execute plugins dynamically
    let mut context = Context {
        params: HashMap::new(),
    };
    context.params.insert("token".to_string(), "abc123".to_string());
    
    match registry.execute("auth", &context) {
        Ok(response) => println!("Auth response: {:?}", response),
        Err(e) => eprintln!("Auth error: {:?}", e),
    }
    
    match registry.execute("logging", &context) {
        Ok(response) => println!("Logging response: {:?}", response),
        Err(e) => eprintln!("Logging error: {:?}", e),
    }
}
```

### 5.6 Real-World Example: Database Abstraction

**Dynamic Backend Selection**:

```rust
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct Record {
    id: String,
    data: HashMap<String, String>,
}

trait Database {
    fn connect(&mut self, connection_string: &str) -> Result<(), String>;
    fn insert(&mut self, record: Record) -> Result<String, String>;
    fn find(&self, id: &str) -> Result<Option<Record>, String>;
    fn delete(&mut self, id: &str) -> Result<bool, String>;
}

// PostgreSQL implementation
struct PostgresDb {
    connected: bool,
    records: HashMap<String, Record>,
}

impl PostgresDb {
    fn new() -> Self {
        Self {
            connected: false,
            records: HashMap::new(),
        }
    }
}

impl Database for PostgresDb {
    fn connect(&mut self, connection_string: &str) -> Result<(), String> {
        println!("PostgreSQL: Connecting to {}", connection_string);
        self.connected = true;
        Ok(())
    }
    
    fn insert(&mut self, record: Record) -> Result<String, String> {
        if !self.connected {
            return Err("Not connected".to_string());
        }
        let id = record.id.clone();
        self.records.insert(id.clone(), record);
        println!("PostgreSQL: Inserted record {}", id);
        Ok(id)
    }
    
    fn find(&self, id: &str) -> Result<Option<Record>, String> {
        if !self.connected {
            return Err("Not connected".to_string());
        }
        Ok(self.records.get(id).cloned())
    }
    
    fn delete(&mut self, id: &str) -> Result<bool, String> {
        if !self.connected {
            return Err("Not connected".to_string());
        }
        Ok(self.records.remove(id).is_some())
    }
}

// MongoDB implementation
struct MongoDb {
    connected: bool,
    records: HashMap<String, Record>,
}

impl MongoDb {
    fn new() -> Self {
        Self {
            connected: false,
            records: HashMap::new(),
        }
    }
}

impl Database for MongoDb {
    fn connect(&mut self, connection_string: &str) -> Result<(), String> {
        println!("MongoDB: Connecting to {}", connection_string);
        self.connected = true;
        Ok(())
    }
    
    fn insert(&mut self, record: Record) -> Result<String, String> {
        if !self.connected {
            return Err("Not connected".to_string());
        }
        let id = record.id.clone();
        self.records.insert(id.clone(), record);
        println!("MongoDB: Inserted document {}", id);
        Ok(id)
    }
    
    fn find(&self, id: &str) -> Result<Option<Record>, String> {
        if !self.connected {
            return Err("Not connected".to_string());
        }
        Ok(self.records.get(id).cloned())
    }
    
    fn delete(&mut self, id: &str) -> Result<bool, String> {
        if !self.connected {
            return Err("Not connected".to_string());
        }
        Ok(self.records.remove(id).is_some())
    }
}

// Application service using dynamic dispatch
struct DataService {
    db: Box<dyn Database>,
}

impl DataService {
    fn new(db: Box<dyn Database>) -> Self {
        Self { db }
    }
    
    fn save_user(&mut self, name: &str, email: &str) -> Result<String, String> {
        let mut data = HashMap::new();
        data.insert("name".to_string(), name.to_string());
        data.insert("email".to_string(), email.to_string());
        
        let record = Record {
            id: format!("user_{}", uuid()),
            data,
        };
        
        self.db.insert(record)
    }
    
    fn get_user(&self, id: &str) -> Result<Option<Record>, String> {
        self.db.find(id)
    }
}

fn uuid() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    format!("{}", now.as_millis())
}

fn main() {
    // Runtime selection of database backend
    let db_type = std::env::var("DB_TYPE").unwrap_or_else(|_| "postgres".to_string());
    
    let mut db: Box<dyn Database> = match db_type.as_str() {
        "postgres" => Box::new(PostgresDb::new()),
        "mongo" => Box::new(MongoDb::new()),
        _ => Box::new(PostgresDb::new()),
    };
    
    db.connect("localhost:5432").unwrap();
    
    let mut service = DataService::new(db);
    
    let user_id = service.save_user("Alice", "alice@example.com").unwrap();
    println!("Saved user: {}", user_id);
    
    if let Some(user) = service.get_user(&user_id).unwrap() {
        println!("Found user: {:?}", user);
    }
}
```

---

## 6. Pattern 3: Hybrid Dispatch Strategies

### 6.1 Static Wrapper Around Dynamic Core

**Best of Both Worlds**:

```rust
trait Processor {
    fn process(&self, data: &[u8]) -> Vec<u8>;
}

struct CompressProcessor;
impl Processor for CompressProcessor {
    fn process(&self, data: &[u8]) -> Vec<u8> {
        println!("Compressing {} bytes", data.len());
        data.to_vec() // Simplified
    }
}

struct EncryptProcessor;
impl Processor for EncryptProcessor {
    fn process(&self, data: &[u8]) -> Vec<u8> {
        println!("Encrypting {} bytes", data.len());
        data.iter().map(|b| b.wrapping_add(1)).collect()
    }
}

// Dynamic core for flexibility
struct DynamicPipeline {
    processors: Vec<Box<dyn Processor>>,
}

impl DynamicPipeline {
    fn new() -> Self {
        Self { processors: vec![] }
    }
    
    fn add(&mut self, processor: Box<dyn Processor>) {
        self.processors.push(processor);
    }
    
    fn execute(&self, data: &[u8]) -> Vec<u8> {
        let mut result = data.to_vec();
        for processor in &self.processors {
            result = processor.process(&result);
        }
        result
    }
}

// Static wrapper for performance-critical paths
#[inline]
fn process_fast<P: Processor>(processor: &P, data: &[u8]) -> Vec<u8> {
    processor.process(data)
}

fn main() {
    // Dynamic for configuration flexibility
    let mut pipeline = DynamicPipeline::new();
    pipeline.add(Box::new(CompressProcessor));
    pipeline.add(Box::new(EncryptProcessor));
    
    let data = b"Hello, World!";
    let result = pipeline.execute(data);
    println!("Pipeline result: {} bytes", result.len());
    
    // Static for hot paths
    let processor = EncryptProcessor;
    let result = process_fast(&processor, data);
    println!("Fast result: {} bytes", result.len());
}
```

### 6.2 Enum Dispatch

**Manual Dispatch for Known Types**:

```rust
// Instead of dynamic dispatch, use enums
#[derive(Debug)]
enum Transport {
    Http(HttpTransport),
    Websocket(WsTransport),
    Grpc(GrpcTransport),
}

#[derive(Debug)]
struct HttpTransport {
    url: String,
}

#[derive(Debug)]
struct WsTransport {
    endpoint: String,
}

#[derive(Debug)]
struct GrpcTransport {
    address: String,
}

impl Transport {
    fn send(&self, message: &str) -> Result<String, String> {
        match self {
            Transport::Http(t) => {
                println!("HTTP: Sending to {}", t.url);
                Ok(format!("HTTP response from {}", t.url))
            }
            Transport::Websocket(t) => {
                println!("WS: Sending to {}", t.endpoint);
                Ok(format!("WS response from {}", t.endpoint))
            }
            Transport::Grpc(t) => {
                println!("gRPC: Sending to {}", t.address);
                Ok(format!("gRPC response from {}", t.address))
            }
        }
    }
    
    fn close(&self) {
        match self {
            Transport::Http(t) => println!("Closing HTTP connection to {}", t.url),
            Transport::Websocket(t) => println!("Closing WS connection to {}", t.endpoint),
            Transport::Grpc(t) => println!("Closing gRPC connection to {}", t.address),
        }
    }
}

// Benefits:
// 1. No vtable indirection
// 2. Exhaustiveness checking
// 3. Better optimization
// 4. Can add new variants without trait implementation

fn main() {
    let transports = vec![
        Transport::Http(HttpTransport { url: "https://api.example.com".to_string() }),
        Transport::Websocket(WsTransport { endpoint: "wss://chat.example.com".to_string() }),
        Transport::Grpc(GrpcTransport { address: "grpc.example.com:50051".to_string() }),
    ];
    
    for transport in &transports {
        match transport.send("Hello") {
            Ok(response) => println!("Response: {}", response),
            Err(e) => eprintln!("Error: {}", e),
        }
    }
    
    for transport in &transports {
        transport.close();
    }
}
```

### 6.3 Conditional Dispatch

**Choose Strategy at Compile Time**:

```rust
trait Allocator {
    fn allocate(&self, size: usize) -> Vec<u8>;
}

struct HeapAllocator;
impl Allocator for HeapAllocator {
    fn allocate(&self, size: usize) -> Vec<u8> {
        vec![0; size]
    }
}

struct PoolAllocator {
    pool: Vec<Vec<u8>>,
}

impl PoolAllocator {
    fn new() -> Self {
        Self { pool: vec![] }
    }
}

impl Allocator for PoolAllocator {
    fn allocate(&self, size: usize) -> Vec<u8> {
        vec![0; size] // Simplified
    }
}

// Static dispatch in performance mode
#[cfg(not(feature = "flexible"))]
fn allocate_buffer(size: usize) -> Vec<u8> {
    let allocator = HeapAllocator;
    allocator.allocate(size)
}

// Dynamic dispatch in flexible mode
#[cfg(feature = "flexible")]
fn allocate_buffer(size: usize) -> Vec<u8> {
    let allocator: Box<dyn Allocator> = Box::new(HeapAllocator);
    allocator.allocate(size)
}

fn main() {
    let buffer = allocate_buffer(1024);
    println!("Allocated {} bytes", buffer.len());
}
```

### 6.4 Type State Pattern with Static Dispatch

**Compile-Time State Machine**:

```rust
use std::marker::PhantomData;

// State types
struct Disconnected;
struct Connected;
struct Authenticated;

// Connection with type-state
struct Connection<State> {
    address: String,
    _state: PhantomData<State>,
}

impl Connection<Disconnected> {
    fn new(address: String) -> Self {
        Self {
            address,
            _state: PhantomData,
        }
    }
    
    fn connect(self) -> Result<Connection<Connected>, String> {
        println!("Connecting to {}", self.address);
        Ok(Connection {
            address: self.address,
            _state: PhantomData,
        })
    }
}

impl Connection<Connected> {
    fn authenticate(self, token: &str) -> Result<Connection<Authenticated>, String> {
        println!("Authenticating with token: {}", token);
        Ok(Connection {
            address: self.address,
            _state: PhantomData,
        })
    }
    
    fn disconnect(self) -> Connection<Disconnected> {
        println!("Disconnecting from {}", self.address);
        Connection {
            address: self.address,
            _state: PhantomData,
        }
    }
}

impl Connection<Authenticated> {
    fn send_message(&self, message: &str) -> Result<String, String> {
        println!("Sending message: {}", message);
        Ok("Message sent".to_string())
    }
    
    fn disconnect(self) -> Connection<Disconnected> {
        println!("Disconnecting from {}", self.address);
        Connection {
            address: self.address,
            _state: PhantomData,
        }
    }
}

fn main() {
    let conn = Connection::new("server.example.com".to_string());
    
    // Compile-time enforcement of state transitions
    let conn = conn.connect().unwrap();
    // conn.send_message("Hello"); // ERROR: method not available
    
    let conn = conn.authenticate("token123").unwrap();
    conn.send_message("Hello").unwrap();
    
    let _conn = conn.disconnect();
    // conn.send_message("Hello"); // ERROR: value moved
}
```

### 6.5 Specialization via Feature Flags

**Compile-Time Backend Selection**:

```rust
trait Backend {
    fn execute(&self, query: &str) -> Result<String, String>;
}

#[cfg(feature = "postgres")]
struct PostgresBackend;

#[cfg(feature = "postgres")]
impl Backend for PostgresBackend {
    fn execute(&self, query: &str) -> Result<String, String> {
        println!("PostgreSQL: {}", query);
        Ok("postgres result".to_string())
    }
}

#[cfg(feature = "mysql")]
struct MySqlBackend;

#[cfg(feature = "mysql")]
impl Backend for MySqlBackend {
    fn execute(&self, query: &str) -> Result<String, String> {
        println!("MySQL: {}", query);
        Ok("mysql result".to_string())
    }
}

#[cfg(not(any(feature = "postgres", feature = "mysql")))]
struct DefaultBackend;

#[cfg(not(any(feature = "postgres", feature = "mysql")))]
impl Backend for DefaultBackend {
    fn execute(&self, query: &str) -> Result<String, String> {
        println!("Default: {}", query);
        Ok("default result".to_string())
    }
}

fn create_backend() -> impl Backend {
    #[cfg(feature = "postgres")]
    return PostgresBackend;
    
    #[cfg(all(feature = "mysql", not(feature = "postgres")))]
    return MySqlBackend;
    
    #[cfg(not(any(feature = "postgres", feature = "mysql")))]
    return DefaultBackend;
}

fn main() {
    let backend = create_backend();
    let result = backend.execute("SELECT * FROM users");
    println!("Result: {:?}", result);
}
```

---

## 7. Pattern 4: Async Trait Dispatch

### 7.1 Problem: Async Traits and Object Safety

**Why `async fn` in Traits is Challenging**:

```rust
// This doesn't work directly with trait objects
trait AsyncProcessor {
    async fn process(&self, data: Vec<u8>) -> Result<Vec<u8>, String>;
    // ERROR: method `process` is not object-safe
    // Reason: Return type is opaque Future with unknown size
}

// Cannot create trait object
// fn use_processor(p: &dyn AsyncProcessor) { } // ERROR
```

**The Issue**: 
- `async fn` returns `impl Future<Output = T>`
- `impl Trait` in return position is not object-safe
- Future size unknown at compile time

### 7.2 Solution 1: Manual Boxing

**Box the Future**:

```rust
use std::future::Future;
use std::pin::Pin;

trait AsyncProcessor {
    fn process(&self, data: Vec<u8>) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, String>> + Send + '_>>;
}

struct Compressor;

impl AsyncProcessor for Compressor {
    fn process(&self, data: Vec<u8>) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, String>> + Send + '_>> {
        Box::pin(async move {
            println!("Compressing {} bytes", data.len());
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            Ok(data) // Simplified
        })
    }
}

struct Encryptor;

impl AsyncProcessor for Encryptor {
    fn process(&self, data: Vec<u8>) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, String>> + Send + '_>> {
        Box::pin(async move {
            println!("Encrypting {} bytes", data.len());
            tokio::time::sleep(tokio::time::Duration::from_millis(150)).await;
            Ok(data.iter().map(|b| b.wrapping_add(1)).collect())
        })
    }
}

async fn run_processor(processor: &dyn AsyncProcessor, data: Vec<u8>) -> Result<Vec<u8>, String> {
    processor.process(data).await
}

#[tokio::main]
async fn main() {
    let compressor = Compressor;
    let encryptor = Encryptor;
    
    let data = vec![1, 2, 3, 4, 5];
    
    let result = run_processor(&compressor, data.clone()).await.unwrap();
    println!("Compressed: {:?}", result);
    
    let result = run_processor(&encryptor, data).await.unwrap();
    println!("Encrypted: {:?}", result);
}
```

### 7.3 Solution 2: async-trait Crate

**Using `async_trait` Macro**:

```rust
use async_trait::async_trait;

#[async_trait]
trait AsyncService {
    async fn fetch(&self, url: &str) -> Result<String, String>;
    async fn store(&self, key: &str, value: String) -> Result<(), String>;
}

struct HttpService {
    base_url: String,
}

#[async_trait]
impl AsyncService for HttpService {
    async fn fetch(&self, url: &str) -> Result<String, String> {
        println!("HTTP: Fetching {}/{}", self.base_url, url);
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        Ok(format!("Data from {}", url))
    }
    
    async fn store(&self, key: &str, value: String) -> Result<(), String> {
        println!("HTTP: Storing {} = {}", key, value);
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        Ok(())
    }
}

struct CacheService {
    cache: std::sync::Arc<tokio::sync::Mutex<std::collections::HashMap<String, String>>>,
}

#[async_trait]
impl AsyncService for CacheService {
    async fn fetch(&self, url: &str) -> Result<String, String> {
        let cache = self.cache.lock().await;
        if let Some(value) = cache.get(url) {
            println!("Cache: Hit for {}", url);
            Ok(value.clone())
        } else {
            println!("Cache: Miss for {}", url);
            Err("Not found".to_string())
        }
    }
    
    async fn store(&self, key: &str, value: String) -> Result<(), String> {
        let mut cache = self.cache.lock().await;
        cache.insert(key.to_string(), value);
        println!("Cache: Stored {}", key);
        Ok(())
    }
}

// Can now use trait objects!
async fn fetch_with_fallback(
    primary: &dyn AsyncService,
    fallback: &dyn AsyncService,
    url: &str,
) -> Result<String, String> {
    match primary.fetch(url).await {
        Ok(data) => Ok(data),
        Err(_) => fallback.fetch(url).await,
    }
}

#[tokio::main]
async fn main() {
    let cache = CacheService {
        cache: std::sync::Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new())),
    };
    
    let http = HttpService {
        base_url: "https://api.example.com".to_string(),
    };
    
    // Try cache first, fallback to HTTP
    let result = fetch_with_fallback(&cache, &http, "users/1").await.unwrap();
    println!("Result: {}", result);
    
    // Store in cache
    cache.store("users/1", "John Doe".to_string()).await.unwrap();
    
    // Now cache will hit
    let result = fetch_with_fallback(&cache, &http, "users/1").await.unwrap();
    println!("Result: {}", result);
}
```

### 7.4 Async Trait Objects in Collections

**Heterogeneous Async Operations**:

```rust
use async_trait::async_trait;
use std::sync::Arc;

#[async_trait]
trait AsyncTask: Send + Sync {
    async fn execute(&self) -> Result<String, String>;
    fn name(&self) -> &str;
}

struct FetchTask {
    url: String,
}

#[async_trait]
impl AsyncTask for FetchTask {
    async fn execute(&self) -> Result<String, String> {
        println!("Fetching {}", self.url);
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        Ok(format!("Data from {}", self.url))
    }
    
    fn name(&self) -> &str {
        "FetchTask"
    }
}

struct ComputeTask {
    value: i32,
}

#[async_trait]
impl AsyncTask for ComputeTask {
    async fn execute(&self) -> Result<String, String> {
        println!("Computing for {}", self.value);
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
        Ok(format!("Computed: {}", self.value * 2))
    }
    
    fn name(&self) -> &str {
        "ComputeTask"
    }
}

struct TaskRunner {
    tasks: Vec<Arc<dyn AsyncTask>>,
}

impl TaskRunner {
    fn new() -> Self {
        Self { tasks: vec![] }
    }
    
    fn add_task(&mut self, task: Arc<dyn AsyncTask>) {
        self.tasks.push(task);
    }
    
    async fn run_all(&self) -> Vec<Result<String, String>> {
        let mut results = vec![];
        
        for task in &self.tasks {
            println!("Running: {}", task.name());
            results.push(task.execute().await);
        }
        
        results
    }
    
    async fn run_parallel(&self) -> Vec<Result<String, String>> {
        let futures: Vec<_> = self.tasks
            .iter()
            .map(|task| task.execute())
            .collect();
        
        futures::future::join_all(futures).await
    }
}

#[tokio::main]
async fn main() {
    let mut runner = TaskRunner::new();
    
    runner.add_task(Arc::new(FetchTask {
        url: "https://api.example.com/users".to_string(),
    }));
    
    runner.add_task(Arc::new(ComputeTask { value: 42 }));
    
    runner.add_task(Arc::new(FetchTask {
        url: "https://api.example.com/posts".to_string(),
    }));
    
    println!("Running sequentially:");
    let results = runner.run_all().await;
    for (i, result) in results.iter().enumerate() {
        println!("Task {}: {:?}", i, result);
    }
    
    println!("\nRunning in parallel:");
    let results = runner.run_parallel().await;
    for (i, result) in results.iter().enumerate() {
        println!("Task {}: {:?}", i, result);
    }
}
```

### 7.5 Real-World Example: Async Middleware Chain

**HTTP Middleware with Dynamic Dispatch**:

```rust
use async_trait::async_trait;
use std::sync::Arc;
use std::collections::HashMap;

#[derive(Clone, Debug)]
struct Request {
    path: String,
    headers: HashMap<String, String>,
    body: Option<String>,
}

#[derive(Clone, Debug)]
struct Response {
    status: u16,
    headers: HashMap<String, String>,
    body: String,
}

#[async_trait]
trait Middleware: Send + Sync {
    async fn process(&self, req: Request) -> Result<Request, Response>;
    fn name(&self) -> &str;
}

struct AuthMiddleware {
    required_token: String,
}

#[async_trait]
impl Middleware for AuthMiddleware {
    async fn process(&self, req: Request) -> Result<Request, Response> {
        println!("[{}] Checking authentication", self.name());
        
        if let Some(token) = req.headers.get("Authorization") {
            if token == &self.required_token {
                Ok(req)
            } else {
                Err(Response {
                    status: 401,
                    headers: HashMap::new(),
                    body: "Invalid token".to_string(),
                })
            }
        } else {
            Err(Response {
                status: 401,
                headers: HashMap::new(),
                body: "Missing Authorization header".to_string(),
            })
        }
    }
    
    fn name(&self) -> &str {
        "AuthMiddleware"
    }
}

struct LoggingMiddleware;

#[async_trait]
impl Middleware for LoggingMiddleware {
    async fn process(&self, req: Request) -> Result<Request, Response> {
        println!("[{}] {} {}", self.name(), "GET", req.path);
        println!("[{}] Headers: {:?}", self.name(), req.headers);
        Ok(req)
    }
    
    fn name(&self) -> &str {
        "LoggingMiddleware"
    }
}

struct RateLimitMiddleware {
    max_requests: u32,
    current: std::sync::Arc<tokio::sync::Mutex<u32>>,
}

#[async_trait]
impl Middleware for RateLimitMiddleware {
    async fn process(&self, req: Request) -> Result<Request, Response> {
        let mut current = self.current.lock().await;
        *current += 1;
        
        println!("[{}] Request count: {}/{}", self.name(), *current, self.max_requests);
        
        if *current > self.max_requests {
            Err(Response {
                status: 429,
                headers: HashMap::new(),
                body: "Rate limit exceeded".to_string(),
            })
        } else {
            Ok(req)
        }
    }
    
    fn name(&self) -> &str {
        "RateLimitMiddleware"
    }
}

struct MiddlewareChain {
    middlewares: Vec<Arc<dyn Middleware>>,
}

impl MiddlewareChain {
    fn new() -> Self {
        Self { middlewares: vec![] }
    }
    
    fn add(&mut self, middleware: Arc<dyn Middleware>) {
        self.middlewares.push(middleware);
    }
    
    async fn execute(&self, mut req: Request) -> Result<Request, Response> {
        for middleware in &self.middlewares {
            match middleware.process(req).await {
                Ok(next_req) => req = next_req,
                Err(response) => return Err(response),
            }
        }
        Ok(req)
    }
}

#[tokio::main]
async fn main() {
    let mut chain = MiddlewareChain::new();
    
    chain.add(Arc::new(LoggingMiddleware));
    chain.add(Arc::new(RateLimitMiddleware {
        max_requests: 3,
        current: Arc::new(tokio::sync::Mutex::new(0)),
    }));
    chain.add(Arc::new(AuthMiddleware {
        required_token: "secret123".to_string(),
    }));
    
    let mut request = Request {
        path: "/api/users".to_string(),
        headers: HashMap::new(),
        body: None,
    };
    request.headers.insert("Authorization".to_string(), "secret123".to_string());
    
    // First request
    match chain.execute(request.clone()).await {
        Ok(req) => println!("Success: {:?}", req.path),
        Err(resp) => println!("Error {}: {}", resp.status, resp.body),
    }
    
    // Second request
    match chain.execute(request.clone()).await {
        Ok(req) => println!("Success: {:?}", req.path),
        Err(resp) => println!("Error {}: {}", resp.status, resp.body),
    }
}
```

---

## 8. Pattern 5: Object Safety and Sized Bounds

### 8.1 Understanding Object Safety

**What Makes a Trait Object-Safe**:

```rust
// Object-safe trait
trait ObjectSafe {
    fn method1(&self) -> i32;
    fn method2(&mut self, value: String);
    fn method3(self: Box<Self>);
}

// Can create trait object
fn use_object_safe(obj: &dyn ObjectSafe) {
    obj.method1();
}

// NOT object-safe: returns Self
trait NotObjectSafe1 {
    fn clone_self(&self) -> Self;
}

// Cannot create: &dyn NotObjectSafe1

// NOT object-safe: generic method
trait NotObjectSafe2 {
    fn process<T>(&self, value: T);
}

// Cannot create: &dyn NotObjectSafe2

// NOT object-safe: requires Sized
trait NotObjectSafe3: Sized {
    fn method(&self);
}

// Cannot create: &dyn NotObjectSafe3
```

### 8.2 Workaround: Split Traits

**Separate Object-Safe Parts**:

```rust
// Non-object-safe trait
trait Cloneable {
    fn clone_box(&self) -> Box<dyn Cloneable>;
}

// Object-safe trait
trait Processor {
    fn process(&self, data: &[u8]) -> Vec<u8>;
}

// Combined trait
trait CloneableProcessor: Processor + Cloneable {}

// Implementation helper
impl<T> Cloneable for T
where
    T: Processor + Clone + 'static,
{
    fn clone_box(&self) -> Box<dyn Cloneable> {
        Box::new(self.clone())
    }
}

#[derive(Clone)]
struct MyProcessor {
    factor: i32,
}

impl Processor for MyProcessor {
    fn process(&self, data: &[u8]) -> Vec<u8> {
        data.iter().map(|b| b.wrapping_mul(self.factor as u8)).collect()
    }
}

impl CloneableProcessor for MyProcessor {}

fn main() {
    let processor = MyProcessor { factor: 2 };
    let boxed: Box<dyn Cloneable> = Box::new(processor);
    
    let cloned = boxed.clone_box();
    println!("Cloned processor");
}
```

### 8.3 Using `Self: Sized` Bound

**Opt-Out of Object Safety**:

```rust
trait Builder {
    // Object-safe methods
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    
    // Non-object-safe method (explicitly marked)
    fn build(self) -> String where Self: Sized {
        format!("{} v{}", self.name(), self.version())
    }
    
    // Generic method (not object-safe)
    fn configure<T: ToString>(&mut self, key: &str, value: T) where Self: Sized;
}

struct AppBuilder {
    name: String,
    version: String,
    config: std::collections::HashMap<String, String>,
}

impl Builder for AppBuilder {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn version(&self) -> &str {
        &self.version
    }
    
    fn configure<T: ToString>(&mut self, key: &str, value: T) {
        self.config.insert(key.to_string(), value.to_string());
    }
}

fn print_info(builder: &dyn Builder) {
    // Can only call object-safe methods
    println!("{} {}", builder.name(), builder.version());
    
    // Cannot call:
    // builder.build(); // ERROR
    // builder.configure("key", "value"); // ERROR
}

fn main() {
    let mut builder = AppBuilder {
        name: "MyApp".to_string(),
        version: "1.0.0".to_string(),
        config: std::collections::HashMap::new(),
    };
    
    // Can call all methods on concrete type
    builder.configure("port", 8080);
    let result = builder.build();
    println!("Built: {}", result);
    
    // Can create trait object but with limitations
    let builder2 = AppBuilder {
        name: "OtherApp".to_string(),
        version: "2.0.0".to_string(),
        config: std::collections::HashMap::new(),
    };
    
    print_info(&builder2);
}
```

### 8.4 Supertraits and Object Safety

**Inherited Constraints**:

```rust
// Base trait
trait Base {
    fn base_method(&self) -> String;
}

// Supertrait adds object-safe methods
trait Extended: Base {
    fn extended_method(&self) -> i32;
}

// Can create trait object for Extended
fn use_extended(obj: &dyn Extended) {
    println!("{}", obj.base_method());
    println!("{}", obj.extended_method());
}

// NOT object-safe if supertrait requires Sized
trait NotObjectSafeSuper: Base + Sized {
    fn method(&self);
}

// Cannot create: &dyn NotObjectSafeSuper

struct MyType;

impl Base for MyType {
    fn base_method(&self) -> String {
        "Base".to_string()
    }
}

impl Extended for MyType {
    fn extended_method(&self) -> i32 {
        42
    }
}

fn main() {
    let obj = MyType;
    use_extended(&obj);
}
```

### 8.5 Associated Types vs Generic Parameters

**Object Safety Implications**:

```rust
// Object-safe: associated type
trait Container {
    type Item;
    
    fn get(&self, index: usize) -> Option<&Self::Item>;
    fn len(&self) -> usize;
}

struct VecContainer<T> {
    items: Vec<T>,
}

impl<T> Container for VecContainer<T> {
    type Item = T;
    
    fn get(&self, index: usize) -> Option<&T> {
        self.items.get(index)
    }
    
    fn len(&self) -> usize {
        self.items.len()
    }
}

// Can create trait object BUT must specify associated type
fn use_container(container: &dyn Container<Item = String>) {
    if let Some(item) = container.get(0) {
        println!("First item: {}", item);
    }
}

// NOT object-safe: generic method parameter
trait NotObjectSafeGeneric {
    fn insert<T>(&mut self, item: T);
}

// Cannot create: &dyn NotObjectSafeGeneric

fn main() {
    let container = VecContainer {
        items: vec!["Hello".to_string(), "World".to_string()],
    };
    
    use_container(&container);
}
```

### 8.6 Real-World Example: Extensible Serializer

**Object-Safe Serialization Framework**:

```rust
use std::collections::HashMap;

// Object-safe serializer trait
trait Serializer {
    fn serialize_bool(&mut self, value: bool);
    fn serialize_i32(&mut self, value: i32);
    fn serialize_string(&mut self, value: &str);
    fn serialize_bytes(&mut self, value: &[u8]);
    fn finalize(self: Box<Self>) -> Vec<u8>;
}

// JSON serializer
struct JsonSerializer {
    buffer: String,
}

impl JsonSerializer {
    fn new() -> Self {
        Self { buffer: String::new() }
    }
}

impl Serializer for JsonSerializer {
    fn serialize_bool(&mut self, value: bool) {
        self.buffer.push_str(&value.to_string());
    }
    
    fn serialize_i32(&mut self, value: i32) {
        self.buffer.push_str(&value.to_string());
    }
    
    fn serialize_string(&mut self, value: &str) {
        self.buffer.push('"');
        self.buffer.push_str(value);
        self.buffer.push('"');
    }
    
    fn serialize_bytes(&mut self, value: &[u8]) {
        self.buffer.push('[');
        for (i, byte) in value.iter().enumerate() {
            if i > 0 {
                self.buffer.push(',');
            }
            self.buffer.push_str(&byte.to_string());
        }
        self.buffer.push(']');
    }
    
    fn finalize(self: Box<Self>) -> Vec<u8> {
        self.buffer.into_bytes()
    }
}

// Binary serializer
struct BinarySerializer {
    buffer: Vec<u8>,
}

impl BinarySerializer {
    fn new() -> Self {
        Self { buffer: vec![] }
    }
}

impl Serializer for BinarySerializer {
    fn serialize_bool(&mut self, value: bool) {
        self.buffer.push(if value { 1 } else { 0 });
    }
    
    fn serialize_i32(&mut self, value: i32) {
        self.buffer.extend_from_slice(&value.to_le_bytes());
    }
    
    fn serialize_string(&mut self, value: &str) {
        let bytes = value.as_bytes();
        self.buffer.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
        self.buffer.extend_from_slice(bytes);
    }
    
    fn serialize_bytes(&mut self, value: &[u8]) {
        self.buffer.extend_from_slice(&(value.len() as u32).to_le_bytes());
        self.buffer.extend_from_slice(value);
    }
    
    fn finalize(self: Box<Self>) -> Vec<u8> {
        self.buffer
    }
}

// Serializable trait
trait Serializable {
    fn serialize(&self, serializer: &mut dyn Serializer);
}

// User type
struct User {
    id: i32,
    name: String,
    active: bool,
}

impl Serializable for User {
    fn serialize(&self, serializer: &mut dyn Serializer) {
        serializer.serialize_i32(self.id);
        serializer.serialize_string(&self.name);
        serializer.serialize_bool(self.active);
    }
}

fn serialize_object(obj: &dyn Serializable, format: &str) -> Vec<u8> {
    let mut serializer: Box<dyn Serializer> = match format {
        "json" => Box::new(JsonSerializer::new()),
        "binary" => Box::new(BinarySerializer::new()),
        _ => Box::new(JsonSerializer::new()),
    };
    
    obj.serialize(&mut *serializer);
    serializer.finalize()
}

fn main() {
    let user = User {
        id: 42,
        name: "Alice".to_string(),
        active: true,
    };
    
    let json = serialize_object(&user, "json");
    println!("JSON: {}", String::from_utf8_lossy(&json));
    
    let binary = serialize_object(&user, "binary");
    println!("Binary: {} bytes", binary.len());
}
```

---

## 9. Performance Characteristics

### 9.1 Benchmarking Static vs Dynamic

**Microbenchmark Setup**:

```rust
use std::time::Instant;

trait Operation {
    fn compute(&self, x: i32) -> i32;
}

struct AddOne;
impl Operation for AddOne {
    fn compute(&self, x: i32) -> i32 {
        x + 1
    }
}

struct MultiplyTwo;
impl Operation for MultiplyTwo {
    fn compute(&self, x: i32) -> i32 {
        x * 2
    }
}

// Static dispatch benchmark
fn benchmark_static<O: Operation>(op: &O, iterations: usize) -> std::time::Duration {
    let start = Instant::now();
    let mut result = 0;
    
    for i in 0..iterations {
        result += op.compute(i as i32);
    }
    
    let duration = start.elapsed();
    println!("Static result: {} (to prevent optimization)", result);
    duration
}

// Dynamic dispatch benchmark
fn benchmark_dynamic(op: &dyn Operation, iterations: usize) -> std::time::Duration {
    let start = Instant::now();
    let mut result = 0;
    
    for i in 0..iterations {
        result += op.compute(i as i32);
    }
    
    let duration = start.elapsed();
    println!("Dynamic result: {} (to prevent optimization)", result);
    duration
}

fn main() {
    let iterations = 10_000_000;
    let op = AddOne;
    
    // Warm up
    benchmark_static(&op, 1000);
    benchmark_dynamic(&op, 1000);
    
    // Actual benchmark
    println!("\nBenchmarking {} iterations:", iterations);
    
    let static_time = benchmark_static(&op, iterations);
    println!("Static dispatch: {:?}", static_time);
    
    let dynamic_time = benchmark_dynamic(&op, iterations);
    println!("Dynamic dispatch: {:?}", dynamic_time);
    
    let overhead = dynamic_time.as_nanos() as f64 / static_time.as_nanos() as f64;
    println!("Dynamic overhead: {:.2}x", overhead);
}
```

### 9.2 Code Size Impact

**Measuring Binary Size**:

```rust
// File: static_example.rs
trait Processor {
    fn process(&self, data: &[u8]) -> Vec<u8>;
}

struct TypeA;
impl Processor for TypeA {
    fn process(&self, data: &[u8]) -> Vec<u8> {
        data.iter().map(|b| b.wrapping_add(1)).collect()
    }
}

struct TypeB;
impl Processor for TypeB {
    fn process(&self, data: &[u8]) -> Vec<u8> {
        data.iter().map(|b| b.wrapping_mul(2)).collect()
    }
}

// This generic function will be monomorphized
fn process_all<P: Processor>(processors: &[P], data: &[u8]) {
    for processor in processors {
        let _ = processor.process(data);
    }
}

fn main() {
    let data = vec![1, 2, 3, 4, 5];
    
    process_all(&[TypeA, TypeA], &data);
    process_all(&[TypeB, TypeB], &data);
    // Generates two versions of process_all
}

// File: dynamic_example.rs
// Same traits and types...

// This function is NOT monomorphized
fn process_all_dyn(processors: &[&dyn Processor], data: &[u8]) {
    for processor in processors {
        let _ = processor.process(data);
    }
}

fn main() {
    let data = vec![1, 2, 3, 4, 5];
    
    let a = TypeA;
    let b = TypeB;
    
    process_all_dyn(&[&a, &a], &data);
    process_all_dyn(&[&b, &b], &data);
    // Only one version of process_all_dyn
}

// Build and compare:
// cargo build --release
// ls -lh target/release/static_example
// ls -lh target/release/dynamic_example
```

### 9.3 Cache Performance

**Instruction Cache Effects**:

```rust
// Large function to demonstrate code bloat
trait ComplexProcessor {
    fn step1(&self, x: i32) -> i32;
    fn step2(&self, x: i32) -> i32;
    fn step3(&self, x: i32) -> i32;
    fn step4(&self, x: i32) -> i32;
    fn step5(&self, x: i32) -> i32;
}

struct ProcessorA;
impl ComplexProcessor for ProcessorA {
    fn step1(&self, x: i32) -> i32 { x + 1 }
    fn step2(&self, x: i32) -> i32 { x * 2 }
    fn step3(&self, x: i32) -> i32 { x - 3 }
    fn step4(&self, x: i32) -> i32 { x / 4 }
    fn step5(&self, x: i32) -> i32 { x % 5 }
}

struct ProcessorB;
impl ComplexProcessor for ProcessorB {
    fn step1(&self, x: i32) -> i32 { x * 2 }
    fn step2(&self, x: i32) -> i32 { x + 2 }
    fn step3(&self, x: i32) -> i32 { x - 1 }
    fn step4(&self, x: i32) -> i32 { x / 2 }
    fn step5(&self, x: i32) -> i32 { x % 3 }
}

// 10 more processor types...

// Static dispatch: 12 monomorphized versions
// May cause instruction cache misses

// Dynamic dispatch: 1 version
// Better instruction cache utilization
```

### 9.4 Optimization Opportunities

**What Compiler Can Do**:

```rust
trait Transform {
    fn transform(&self, x: i32) -> i32;
}

struct ConstantTransform {
    value: i32,
}

impl Transform for ConstantTransform {
    fn transform(&self, x: i32) -> i32 {
        x + self.value
    }
}

// Static dispatch: can be optimized aggressively
#[inline]
fn apply_static<T: Transform>(t: &T, data: &mut [i32]) {
    for item in data {
        *item = t.transform(*item);
        // Compiler can:
        // 1. Inline transform()
        // 2. Vectorize loop (SIMD)
        // 3. Unroll loop
        // 4. Constant propagate if value known
    }
}

// Dynamic dispatch: limited optimization
fn apply_dynamic(t: &dyn Transform, data: &mut [i32]) {
    for item in data {
        *item = t.transform(*item);
        // Compiler cannot:
        // 1. Inline (unknown target)
        // 2. Easily vectorize
        // 3. Constant propagate
        // But still can:
        // 1. Optimize loop structure
        // 2. Branch prediction hints
    }
}
```

### 9.5 Real-World Performance Comparison

**Production Scenario**:

```rust
use std::time::Instant;

trait Validator {
    fn validate(&self, input: &str) -> bool;
}

struct EmailValidator;
impl Validator for EmailValidator {
    fn validate(&self, input: &str) -> bool {
        input.contains('@') && input.contains('.')
    }
}

struct UrlValidator;
impl Validator for UrlValidator {
    fn validate(&self, input: &str) -> bool {
        input.starts_with("http://") || input.starts_with("https://")
    }
}

struct PhoneValidator;
impl Validator for PhoneValidator {
    fn validate(&self, input: &str) -> bool {
        input.chars().filter(|c| c.is_numeric()).count() >= 10
    }
}

// Static dispatch version
fn validate_batch_static<V: Validator>(validator: &V, inputs: &[String]) -> usize {
    inputs.iter().filter(|s| validator.validate(s)).count()
}

// Dynamic dispatch version
fn validate_batch_dynamic(validator: &dyn Validator, inputs: &[String]) -> usize {
    inputs.iter().filter(|s| validator.validate(s)).count()
}

fn main() {
    // Generate test data
    let test_data: Vec<String> = (0..100_000)
        .map(|i| format!("user{}@example.com", i))
        .collect();
    
    let email_validator = EmailValidator;
    
    // Benchmark static
    let start = Instant::now();
    for _ in 0..100 {
        let _ = validate_batch_static(&email_validator, &test_data);
    }
    let static_time = start.elapsed();
    
    // Benchmark dynamic
    let start = Instant::now();
    for _ in 0..100 {
        let _ = validate_batch_dynamic(&email_validator, &test_data);
    }
    let dynamic_time = start.elapsed();
    
    println!("Static: {:?}", static_time);
    println!("Dynamic: {:?}", dynamic_time);
    println!("Ratio: {:.2}x", dynamic_time.as_secs_f64() / static_time.as_secs_f64());
}
```

### 9.6 When Performance Doesn't Matter

**Scenarios Where Dynamic Dispatch is Fine**:

```rust
// 1. I/O bound operations
trait HttpClient {
    fn fetch(&self, url: &str) -> Result<String, String>;
}

fn fetch_data(client: &dyn HttpClient, urls: &[&str]) -> Vec<String> {
    // Network I/O dominates, dispatch overhead negligible
    urls.iter()
        .filter_map(|url| client.fetch(url).ok())
        .collect()
}

// 2. Infrequent calls
trait ConfigLoader {
    fn load(&self) -> Config;
}

fn initialize(loader: &dyn ConfigLoader) -> Config {
    // Called once at startup, performance irrelevant
    loader.load()
}

// 3. UI event handlers
trait EventHandler {
    fn handle(&self, event: &Event);
}

fn dispatch_event(handlers: &[Box<dyn EventHandler>], event: &Event) {
    // Human interaction timescales, microseconds don't matter
    for handler in handlers {
        handler.handle(event);
    }
}

// 4. Complex business logic
trait PaymentProcessor {
    fn process_payment(&self, amount: f64) -> Result<Receipt, Error>;
}

fn checkout(processor: &dyn PaymentProcessor, cart: &Cart) -> Result<Receipt, Error> {
    // Database access, network calls, validation - all dwarf dispatch cost
    processor.process_payment(cart.total())
}

struct Config;
struct Event;
struct Cart {
    items: Vec<String>,
}

impl Cart {
    fn total(&self) -> f64 {
        100.0
    }
}

struct Receipt;
struct Error;
```

---

## 10. Testing Strategies

### 10.1 Testing Static Dispatch

**Unit Testing Generic Functions**:

```rust
trait Calculator {
    fn calculate(&self, a: i32, b: i32) -> i32;
}

struct Adder;
impl Calculator for Adder {
    fn calculate(&self, a: i32, b: i32) -> i32 {
        a + b
    }
}

struct Multiplier;
impl Calculator for Multiplier {
    fn calculate(&self, a: i32, b: i32) -> i32 {
        a * b
    }
}

// Generic function under test
fn apply_operation<C: Calculator>(calc: &C, values: &[(i32, i32)]) -> Vec<i32> {
    values.iter()
        .map(|(a, b)| calc.calculate(*a, *b))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_apply_with_adder() {
        let adder = Adder;
        let values = vec![(1, 2), (3, 4), (5, 6)];
        let result = apply_operation(&adder, &values);
        assert_eq!(result, vec![3, 7, 11]);
    }
    
    #[test]
    fn test_apply_with_multiplier() {
        let mult = Multiplier;
        let values = vec![(2, 3), (4, 5), (6, 7)];
        let result = apply_operation(&mult, &values);
        assert_eq!(result, vec![6, 20, 42]);
    }
    
    // Mock calculator for testing
    struct MockCalculator {
        expected_calls: Vec<(i32, i32, i32)>, // (a, b, result)
        call_count: std::cell::RefCell<usize>,
    }
    
    impl MockCalculator {
        fn new(expected: Vec<(i32, i32, i32)>) -> Self {
            Self {
                expected_calls: expected,
                call_count: std::cell::RefCell::new(0),
            }
        }
    }
    
    impl Calculator for MockCalculator {
        fn calculate(&self, a: i32, b: i32) -> i32 {
            let mut count = self.call_count.borrow_mut();
            let (exp_a, exp_b, result) = self.expected_calls[*count];
            assert_eq!(a, exp_a, "Unexpected value for 'a' at call {}", count);
            assert_eq!(b, exp_b, "Unexpected value for 'b' at call {}", count);
            *count += 1;
            result
        }
    }
    
    #[test]
    fn test_with_mock() {
        let mock = MockCalculator::new(vec![
            (1, 2, 10),
            (3, 4, 20),
        ]);
        
        let values = vec![(1, 2), (3, 4)];
        let result = apply_operation(&mock, &values);
        assert_eq!(result, vec![10, 20]);
        assert_eq!(*mock.call_count.borrow(), 2);
    }
}
```

### 10.2 Testing Dynamic Dispatch

**Testing Trait Objects**:

```rust
trait DataSource {
    fn fetch(&self, key: &str) -> Option<String>;
    fn store(&mut self, key: &str, value: String);
}

struct MemorySource {
    data: std::collections::HashMap<String, String>,
}

impl DataSource for MemorySource {
    fn fetch(&self, key: &str) -> Option<String> {
        self.data.get(key).cloned()
    }
    
    fn store(&mut self, key: &str, value: String) {
        self.data.insert(key.to_string(), value);
    }
}

// Function using dynamic dispatch
fn process_data(source: &mut dyn DataSource, key: &str) -> String {
    if let Some(value) = source.fetch(key) {
        value.to_uppercase()
    } else {
        let default = "DEFAULT".to_string();
        source.store(key, default.clone());
        default
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_with_existing_value() {
        let mut source = MemorySource {
            data: [("key1".to_string(), "value1".to_string())]
                .iter()
                .cloned()
                .collect(),
        };
        
        let result = process_data(&mut source, "key1");
        assert_eq!(result, "VALUE1");
    }
    
    #[test]
    fn test_with_missing_value() {
        let mut source = MemorySource {
            data: std::collections::HashMap::new(),
        };
        
        let result = process_data(&mut source, "key1");
        assert_eq!(result, "DEFAULT");
        assert_eq!(source.fetch("key1"), Some("DEFAULT".to_string()));
    }
    
    // Test spy to verify method calls
    struct SpySource {
        fetch_calls: std::cell::RefCell<Vec<String>>,
        store_calls: std::cell::RefCell<Vec<(String, String)>>,
        data: std::collections::HashMap<String, String>,
    }
    
    impl SpySource {
        fn new() -> Self {
            Self {
                fetch_calls: std::cell::RefCell::new(vec![]),
                store_calls: std::cell::RefCell::new(vec![]),
                data: std::collections::HashMap::new(),
            }
        }
        
        fn verify_fetch_called(&self, key: &str) -> bool {
            self.fetch_calls.borrow().contains(&key.to_string())
        }
        
        fn verify_store_called(&self, key: &str, value: &str) -> bool {
            self.store_calls
                .borrow()
                .contains(&(key.to_string(), value.to_string()))
        }
    }
    
    impl DataSource for SpySource {
        fn fetch(&self, key: &str) -> Option<String> {
            self.fetch_calls.borrow_mut().push(key.to_string());
            self.data.get(key).cloned()
        }
        
        fn store(&mut self, key: &str, value: String) {
            self.store_calls
                .borrow_mut()
                .push((key.to_string(), value.clone()));
        }
    }
    
    #[test]
    fn test_method_calls() {
        let mut spy = SpySource::new();
        let _ = process_data(&mut spy, "test_key");
        
        assert!(spy.verify_fetch_called("test_key"));
        assert!(spy.verify_store_called("test_key", "DEFAULT"));
    }
}
```

### 10.3 Property-Based Testing

**Testing Dispatch Equivalence**:

```rust
trait Hasher {
    fn hash(&self, input: &str) -> u64;
}

struct SimpleHasher;
impl Hasher for SimpleHasher {
    fn hash(&self, input: &str) -> u64 {
        input.bytes().map(|b| b as u64).sum()
    }
}

// Static version
fn hash_all_static<H: Hasher>(hasher: &H, inputs: &[&str]) -> Vec<u64> {
    inputs.iter().map(|s| hasher.hash(s)).collect()
}

// Dynamic version
fn hash_all_dynamic(hasher: &dyn Hasher, inputs: &[&str]) -> Vec<u64> {
    inputs.iter().map(|s| hasher.hash(s)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_dispatch_equivalence() {
        let hasher = SimpleHasher;
        let inputs = vec!["hello", "world", "rust", "testing"];
        
        let static_result = hash_all_static(&hasher, &inputs);
        let dynamic_result = hash_all_dynamic(&hasher, &inputs);
        
        assert_eq!(static_result, dynamic_result);
    }
    
    // Property: both dispatch methods should give same results
    #[test]
    fn property_dispatch_equivalence() {
        let hasher = SimpleHasher;
        
        // Test with various inputs
        let test_cases = vec![
            vec![],
            vec!["a"],
            vec!["a", "b", "c"],
            vec!["longer", "strings", "here"],
            vec!["", "", ""],
        ];
        
        for inputs in test_cases {
            let static_result = hash_all_static(&hasher, &inputs);
            let dynamic_result = hash_all_dynamic(&hasher, &inputs);
            
            assert_eq!(
                static_result, dynamic_result,
                "Mismatch for inputs: {:?}",
                inputs
            );
        }
    }
}
```

### 10.4 Testing Async Trait Dispatch

**Async Test Strategies**:

```rust
use async_trait::async_trait;

#[async_trait]
trait AsyncValidator {
    async fn validate(&self, input: &str) -> Result<bool, String>;
}

struct EmailAsyncValidator;

#[async_trait]
impl AsyncValidator for EmailAsyncValidator {
    async fn validate(&self, input: &str) -> Result<bool, String> {
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        Ok(input.contains('@'))
    }
}

async fn validate_batch(validator: &dyn AsyncValidator, inputs: &[String]) -> Vec<bool> {
    let mut results = vec![];
    for input in inputs {
        if let Ok(valid) = validator.validate(input).await {
            results.push(valid);
        }
    }
    results
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_async_validator() {
        let validator = EmailAsyncValidator;
        let inputs = vec![
            "user@example.com".to_string(),
            "invalid".to_string(),
            "another@test.com".to_string(),
        ];
        
        let results = validate_batch(&validator, &inputs).await;
        assert_eq!(results, vec![true, false, true]);
    }
    
    // Mock async validator
    struct MockAsyncValidator {
        responses: Vec<Result<bool, String>>,
        call_count: std::sync::Arc<tokio::sync::Mutex<usize>>,
    }
    
    impl MockAsyncValidator {
        fn new(responses: Vec<Result<bool, String>>) -> Self {
            Self {
                responses,
                call_count: std::sync::Arc::new(tokio::sync::Mutex::new(0)),
            }
        }
    }
    
    #[async_trait]
    impl AsyncValidator for MockAsyncValidator {
        async fn validate(&self, _input: &str) -> Result<bool, String> {
            let mut count = self.call_count.lock().await;
            let response = self.responses[*count].clone();
            *count += 1;
            response
        }
    }
    
    #[tokio::test]
    async fn test_with_mock() {
        let mock = MockAsyncValidator::new(vec![
            Ok(true),
            Ok(false),
            Ok(true),
        ]);
        
        let inputs = vec![
            "test1".to_string(),
            "test2".to_string(),
            "test3".to_string(),
        ];
        
        let results = validate_batch(&mock, &inputs).await;
        assert_eq!(results, vec![true, false, true]);
    }
}
```

### 10.5 Integration Testing

**Testing Dispatch in Complex Scenarios**:

```rust
trait Storage {
    fn read(&self, key: &str) -> Option<Vec<u8>>;
    fn write(&mut self, key: &str, value: Vec<u8>) -> Result<(), String>;
}

trait Serializer {
    fn serialize(&self, data: &str) -> Vec<u8>;
    fn deserialize(&self, data: &[u8]) -> String;
}

// Service using both traits with dynamic dispatch
struct DataService {
    storage: Box<dyn Storage>,
    serializer: Box<dyn Serializer>,
}

impl DataService {
    fn new(storage: Box<dyn Storage>, serializer: Box<dyn Serializer>) -> Self {
        Self { storage, serializer }
    }
    
    fn save(&mut self, key: &str, value: &str) -> Result<(), String> {
        let serialized = self.serializer.serialize(value);
        self.storage.write(key, serialized)
    }
    
    fn load(&self, key: &str) -> Option<String> {
        self.storage
            .read(key)
            .map(|data| self.serializer.deserialize(&data))
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use std::collections::HashMap;
    
    struct MemStorage {
        data: HashMap<String, Vec<u8>>,
    }
    
    impl MemStorage {
        fn new() -> Self {
            Self { data: HashMap::new() }
        }
    }
    
    impl Storage for MemStorage {
        fn read(&self, key: &str) -> Option<Vec<u8>> {
            self.data.get(key).cloned()
        }
        
        fn write(&mut self, key: &str, value: Vec<u8>) -> Result<(), String> {
            self.data.insert(key.to_string(), value);
            Ok(())
        }
    }
    
    struct JsonSerializer;
    
    impl Serializer for JsonSerializer {
        fn serialize(&self, data: &str) -> Vec<u8> {
            format!("\"{}\"", data).into_bytes()
        }
        
        fn deserialize(&self, data: &[u8]) -> String {
            String::from_utf8_lossy(data)
                .trim_matches('"')
                .to_string()
        }
    }
    
    #[test]
    fn test_service_integration() {
        let storage = Box::new(MemStorage::new());
        let serializer = Box::new(JsonSerializer);
        let mut service = DataService::new(storage, serializer);
        
        // Save data
        service.save("user:1", "Alice").unwrap();
        service.save("user:2", "Bob").unwrap();
        
        // Load data
        assert_eq!(service.load("user:1"), Some("Alice".to_string()));
        assert_eq!(service.load("user:2"), Some("Bob".to_string()));
        assert_eq!(service.load("user:3"), None);
    }
}
```

---

## 11. Anti-Patterns

### 11.1 Over-Using Dynamic Dispatch

**Problem**: Using trait objects when static dispatch would suffice

```rust
// ❌ BAD: Unnecessary dynamic dispatch
fn process_items(items: Vec<Box<dyn Item>>) {
    for item in items {
        item.process();
    }
}

// ✅ GOOD: Use static dispatch when possible
fn process_items<I: Item>(items: Vec<I>) {
    for item in items {
        item.process();
    }
}

// ✅ GOOD: Or use iterator for flexibility
fn process_items_iter<I>(items: impl Iterator<Item = I>)
where
    I: Item,
{
    for item in items {
        item.process();
    }
}

trait Item {
    fn process(&self);
}
```

**Why It's Bad**:
- Performance overhead for no benefit
- Loses zero-cost abstraction
- Requires heap allocation
- Prevents compiler optimizations

**When Dynamic is Actually Needed**:
```rust
// ✅ GOOD: Heterogeneous collection requires dynamic dispatch
fn process_mixed(items: Vec<Box<dyn Item>>) {
    // items can be different types
    for item in items {
        item.process();
    }
}
```

### 11.2 Ignoring Object Safety

**Problem**: Trying to use non-object-safe traits as trait objects

```rust
trait Processor {
    // ❌ BAD: Returns Self - not object safe
    fn clone_processor(&self) -> Self;
    
    // ❌ BAD: Generic method - not object safe
    fn process<T>(&self, item: T) -> T;
}

// ERROR: Cannot create &dyn Processor

// ✅ GOOD: Make trait object-safe
trait ObjectSafeProcessor {
    fn clone_box(&self) -> Box<dyn ObjectSafeProcessor>;
    fn process_bytes(&self, item: &[u8]) -> Vec<u8>;
}

// ✅ GOOD: Use Self: Sized to opt out of object safety
trait FlexibleProcessor {
    fn process_bytes(&self, item: &[u8]) -> Vec<u8>;
    
    // Can't be called on trait object, but trait is still object-safe
    fn clone_processor(&self) -> Self where Self: Sized + Clone {
        self.clone()
    }
}
```

### 11.3 Premature Abstraction

**Problem**: Creating trait hierarchies before understanding requirements

```rust
// ❌ BAD: Over-engineered from the start
trait BaseProcessor {
    fn init(&mut self);
}

trait AdvancedProcessor: BaseProcessor {
    fn configure(&mut self, config: Config);
}

trait SuperAdvancedProcessor: AdvancedProcessor {
    fn optimize(&mut self);
}

// Used in only one place with one implementation

// ✅ GOOD: Start simple
struct Processor {
    config: Config,
}

impl Processor {
    fn new(config: Config) -> Self {
        Self { config }
    }
    
    fn process(&self, data: &[u8]) -> Vec<u8> {
        // Implementation
        data.to_vec()
    }
}

// Add traits only when you have multiple implementations
// or when you need polymorphism

struct Config;
```

**When to Add Abstraction**:
- You have 2+ concrete implementations
- You need runtime polymorphism
- You're defining a library interface
- You're following an established pattern (Iterator, Future, etc.)

### 11.4 Forgetting Send + Sync Bounds

**Problem**: Using trait objects in concurrent contexts without proper bounds

```rust
use std::sync::Arc;
use std::thread;

trait Handler {
    fn handle(&self, data: &str);
}

// ❌ BAD: Won't compile in threaded context
fn spawn_handler(handler: Arc<dyn Handler>) {
    thread::spawn(move || {
        handler.handle("data");
        // ERROR: `dyn Handler` cannot be sent between threads safely
    });
}

// ✅ GOOD: Add required bounds
fn spawn_handler_safe(handler: Arc<dyn Handler + Send + Sync>) {
    thread::spawn(move || {
        handler.handle("data");
        // Works!
    });
}

// ✅ GOOD: Make trait require bounds
trait ThreadSafeHandler: Send + Sync {
    fn handle(&self, data: &str);
}

fn spawn_threadsafe_handler(handler: Arc<dyn ThreadSafeHandler>) {
    thread::spawn(move || {
        handler.handle("data");
    });
}
```

### 11.5 Boxed Everything Syndrome

**Problem**: Boxing all trait objects unnecessarily

```rust
// ❌ BAD: Unnecessary allocations
fn process_borrowed(handler: Box<dyn Handler>) {
    handler.handle("data");
    // Box is dropped here, wasting allocation
}

// ✅ GOOD: Use reference when you don't need ownership
fn process_borrowed_good(handler: &dyn Handler) {
    handler.handle("data");
}

// ❌ BAD: Boxing when you could use generics
fn process_generic_bad(handlers: Vec<Box<dyn Handler>>) {
    for handler in handlers {
        handler.handle("data");
    }
}

// ✅ GOOD: Use generics for homogeneous collections
fn process_generic_good<H: Handler>(handlers: Vec<H>) {
    for handler in &handlers {
        handler.handle("data");
    }
}

// ✅ Box is correct: heterogeneous collection
fn process_heterogeneous(handlers: Vec<Box<dyn Handler>>) {
    for handler in handlers {
        handler.handle("data");
    }
}

struct Handler;
impl Handler {
    fn handle(&self, _data: &str) {}
}
```

### 11.6 Incorrect Lifetime Bounds

**Problem**: Overly restrictive or missing lifetime bounds

```rust
trait Formatter {
    fn format(&self, data: &str) -> String;
}

// ❌ BAD: Unnecessary 'static bound
fn format_data(formatter: &'static dyn Formatter, data: &str) -> String {
    formatter.format(data)
    // 'static is too restrictive
}

// ✅ GOOD: Appropriate lifetime
fn format_data_good<'a>(formatter: &'a dyn Formatter, data: &str) -> String {
    formatter.format(data)
}

// ❌ BAD: Missing lifetime on trait object
fn store_formatter(formatter: Box<dyn Formatter>) -> Box<dyn Formatter> {
    formatter
    // Implicitly 'static, may not be desired
}

// ✅ GOOD: Explicit lifetime for flexibility
fn store_formatter_good<'a>(formatter: Box<dyn Formatter + 'a>) -> Box<dyn Formatter + 'a> {
    formatter
}
```

### 11.7 Clone Trait with Trait Objects

**Problem**: Trying to clone trait objects without proper setup

```rust
trait Processor: Clone {
    fn process(&self, data: &[u8]) -> Vec<u8>;
}

// ERROR: Processor is not object-safe due to Clone

// ✅ GOOD: Implement manual clone for trait objects
trait CloneableProcessor {
    fn process(&self, data: &[u8]) -> Vec<u8>;
    fn clone_box(&self) -> Box<dyn CloneableProcessor>;
}

// Blanket implementation
impl<T> CloneableProcessor for T
where
    T: Processor + Clone + 'static,
{
    fn process(&self, data: &[u8]) -> Vec<u8> {
        Processor::process(self, data)
    }
    
    fn clone_box(&self) -> Box<dyn CloneableProcessor> {
        Box::new(self.clone())
    }
}

// Now you can clone trait objects
fn clone_processor(p: &Box<dyn CloneableProcessor>) -> Box<dyn CloneableProcessor> {
    p.clone_box()
}
```

### 11.8 Forgetting async_trait for Async Methods

**Problem**: Trying to use async methods in traits without proper setup

```rust
// ❌ BAD: Won't work with trait objects
trait AsyncProcessor {
    async fn process(&self, data: Vec<u8>) -> Result<Vec<u8>, String>;
    // ERROR: async fn in trait not object-safe
}

// ✅ GOOD: Use async_trait
use async_trait::async_trait;

#[async_trait]
trait AsyncProcessorGood {
    async fn process(&self, data: Vec<u8>) -> Result<Vec<u8>, String>;
}

// Now you can use trait objects
async fn use_processor(p: &dyn AsyncProcessorGood, data: Vec<u8>) {
    let _ = p.process(data).await;
}
```

### 11.9 Mixing Static and Dynamic Without Reason

**Problem**: Inconsistent dispatch strategy

```rust
trait Service {
    fn execute(&self) -> String;
}

// ❌ BAD: Inconsistent - sometimes static, sometimes dynamic
fn process_services<S: Service>(static_svc: &S, dynamic_svc: &dyn Service) {
    static_svc.execute();
    dynamic_svc.execute();
    // Why the inconsistency?
}

// ✅ GOOD: Consistent approach
fn process_services_static<S1: Service, S2: Service>(svc1: &S1, svc2: &S2) {
    svc1.execute();
    svc2.execute();
}

// ✅ GOOD: Or all dynamic
fn process_services_dynamic(svc1: &dyn Service, svc2: &dyn Service) {
    svc1.execute();
    svc2.execute();
}

// ✅ GOOD: Mix only when there's a clear reason
fn process_hot_and_cold<S: Service>(
    hot_path: &S,           // Static for performance
    cold_path: &dyn Service // Dynamic for flexibility
) {
    // hot_path called frequently
    for _ in 0..1000000 {
        hot_path.execute();
    }
    
    // cold_path called rarely
    cold_path.execute();
}
```

### 11.10 Large Trait Objects in Collections

**Problem**: Storing large trait objects by value

```rust
// ❌ BAD: Each trait object is 2 pointers (16 bytes on 64-bit)
// Plus heap allocation overhead
struct Registry {
    handlers: Vec<Box<dyn Handler>>,
}

// If you have thousands of handlers, consider:

// ✅ GOOD: Pool allocations
use std::sync::Arc;

struct EfficientRegistry {
    handlers: Vec<Arc<dyn Handler>>,
}

// ✅ GOOD: Or use arena allocation
struct ArenaRegistry<'a> {
    handlers: Vec<&'a dyn Handler>,
}

// ✅ GOOD: Or consider enum dispatch for known types
enum HandlerEnum {
    TypeA(HandlerA),
    TypeB(HandlerB),
    TypeC(HandlerC),
}

impl HandlerEnum {
    fn handle(&self, data: &str) {
        match self {
            HandlerEnum::TypeA(h) => h.handle(data),
            HandlerEnum::TypeB(h) => h.handle(data),
            HandlerEnum::TypeC(h) => h.handle(data),
        }
    }
}

struct EnumRegistry {
    handlers: Vec<HandlerEnum>,
}

struct HandlerA;
struct HandlerB;
struct HandlerC;

impl HandlerA {
    fn handle(&self, _data: &str) {}
}
impl HandlerB {
    fn handle(&self, _data: &str) {}
}
impl HandlerC {
    fn handle(&self, _data: &str) {}
}
```

---

## 12. Conclusion

### 12.1 Key Takeaways

**Static Dispatch**:
- ✅ Zero-cost abstraction
- ✅ Aggressive optimization (inlining, vectorization)
- ✅ No runtime overhead
- ❌ Code bloat from monomorphization
- ❌ Longer compile times
- ❌ Cannot store heterogeneous collections

**Dynamic Dispatch**:
- ✅ Runtime polymorphism
- ✅ Heterogeneous collections
- ✅ Smaller binary size
- ✅ Faster compilation
- ❌ Virtual call overhead
- ❌ Limited optimization
- ❌ Object safety restrictions

### 12.2 Decision Tree

```
START: Need polymorphism?
│
├─ NO → Use concrete types
│
└─ YES → Types known at compile time?
    │
    ├─ YES → Need heterogeneous collection?
    │   │
    │   ├─ NO → Use static dispatch (generics)
    │   │
    │   └─ YES → Types from finite set?
    │       │
    │       ├─ YES → Consider enum dispatch
    │       │
    │       └─ NO → Use dynamic dispatch (trait objects)
    │
    └─ NO → Need runtime polymorphism?
        │
        ├─ YES → Use dynamic dispatch (trait objects)
        │
        └─ NO → Reconsider design
```

### 12.3 Selection Checklist

**Use Static Dispatch When**:
- [ ] Performance is critical
- [ ] All types known at compile time
- [ ] Code is in hot path (called frequently)
- [ ] Binary size is not a constraint
- [ ] You want maximum optimization
- [ ] Working on embedded systems
- [ ] Using `no_std` environment

**Use Dynamic Dispatch When**:
- [ ] Need heterogeneous collections
- [ ] Types determined at runtime
- [ ] Building plugin system
- [ ] Binary size is a concern
- [ ] Code bloat from monomorphization is problematic
- [ ] API stability across versions is important
- [ ] I/O or business logic dominates (performance less critical)

**Consider Enum Dispatch When**:
- [ ] Finite set of known types
- [ ] Want exhaustiveness checking
- [ ] Need pattern matching
- [ ] Performance critical but need flexibility
- [ ] Types known at compile time but varying at runtime

### 12.4 Hybrid Strategy Recommendations

**Layered Approach**:

```rust
// Public API: Dynamic for flexibility
pub struct Engine {
    optimizer: Box<dyn Optimizer>,
}

impl Engine {
    pub fn new(optimizer: Box<dyn Optimizer>) -> Self {
        Self { optimizer }
    }
    
    pub fn optimize(&self, data: &[f64]) -> Vec<f64> {
        self.optimizer.optimize(data)
    }
}

// Internal hot paths: Static for performance
impl Engine {
    #[inline]
    fn internal_transform<T: Transform>(&self, transform: &T, data: &[f64]) -> Vec<f64> {
        data.iter().map(|x| transform.apply(*x)).collect()
    }
}

trait Optimizer {
    fn optimize(&self, data: &[f64]) -> Vec<f64>;
}

trait Transform {
    fn apply(&self, value: f64) -> f64;
}
```

### 12.5 Common Patterns Summary

| Pattern | Use Case | Dispatch Type |
|---------|----------|---------------|
| Generic functions | Known types, performance critical | Static |
| Trait objects | Heterogeneous collections | Dynamic |
| Enum dispatch | Finite type set, exhaustiveness | Manual |
| Type-state pattern | Compile-time state machine | Static |
| Strategy pattern | Runtime algorithm selection | Dynamic |
| Plugin architecture | Dynamic loading | Dynamic |
| Iterator chains | Data processing pipelines | Static |
| Async traits | Async polymorphism | Dynamic (with async_trait) |

### 12.6 Performance Guidelines

**Optimize Hot Paths**:
1. Profile first - measure actual performance
2. Use static dispatch in tight loops
3. Consider `#[inline]` for small functions
4. Avoid dynamic dispatch in inner loops
5. Use enum dispatch as middle ground

**When to Stop Optimizing**:
1. Performance meets requirements
2. Code becomes less maintainable
3. I/O or business logic dominates
4. Optimization won't be noticed by users

### 12.7 Maintainability Considerations

**Favor Readability**:
```rust
// Readable but slightly slower
fn process(handler: &dyn Handler) {
    handler.handle();
}

// Faster but more complex
fn process<H: Handler>(handler: &H) {
    handler.handle();
}
```

Choose dynamic dispatch when:
- Code clarity improves significantly
- Performance difference is negligible
- Easier to test and mock
- API is more flexible

### 12.8 Testing Strategy

1. **Unit tests**: Test both static and dynamic paths
2. **Integration tests**: Test real dispatch scenarios
3. **Benchmarks**: Measure actual performance impact
4. **Property tests**: Verify dispatch equivalence

### 12.9 Further Reading

**Official Documentation**:
- [Rust Book - Trait Objects](https://doc.rust-lang.org/book/ch17-02-trait-objects.html)
- [Rust Reference - Trait Objects](https://doc.rust-lang.org/reference/types/trait-object.html)
- [Rust Performance Book](https://nnethercote.github.io/perf-book/)

**Related Guides**:
- [Rust ADT Implementation Guide](./rust-adt-implementation-guide.md)
- [Rust Builder Pattern Guide](./rust-builder-pattern-guide.md)
- [Rust Dependency Management Guide](./rust-dependency-management-guide.md)
- [Rust Factory Pattern Guide](./rust-factory-pattern-guide.md)

**External Resources**:
- [Trait Objects and Dynamic Dispatch (Brandeis CS)](https://www.cs.brandeis.edu/~cs146a/rust/doc-02-21-2015/book/static-and-dynamic-dispatch.html)
- [Assembly Comparison: Static vs Dynamic](https://eventhelix.com/rust/rust-to-assembly-static-vs-dynamic-dispatch/)
- [SoftwareMill: Rust Static vs Dynamic Dispatch](https://softwaremill.com/rust-static-vs-dynamic-dispatch/)

### 12.10 Final Recommendations

**Start Simple**:
1. Begin with concrete types
2. Add generics when you have 2+ implementations
3. Use trait objects only when you need runtime polymorphism
4. Profile before optimizing

**Design for Change**:
1. Rust makes it easy to refactor between static/dynamic
2. Start with the simpler approach
3. Measure and optimize based on real data
4. Don't prematurely optimize

**Remember**:
> "Premature optimization is the root of all evil" - Donald Knuth

Choose the dispatch mechanism that makes your code:
1. **Correct** first
2. **Clear** second  
3. **Fast** third

In most cases, the clarity and correctness matter more than microseconds.

---

**Document Version**: 1.0  
**Last Updated**: 2026-01  
**Rust Version**: 1.75+  
**License**: CC BY-SA 4.0

