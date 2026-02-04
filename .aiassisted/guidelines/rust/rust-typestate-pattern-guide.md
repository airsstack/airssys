# Rust TypeState Pattern Implementation Guide

> **Comprehensive guide to the TypeState pattern in Rust**

**Status:** Complete  
**Assumed Knowledge:** Intermediate Rust (traits, generics, ownership, PhantomData, zero-sized types)  
**Related Documents:**
- [Rust ADT Implementation Guide](./rust-adt-implementation-guide.md)
- [Rust Builder Pattern Guide](./rust-builder-pattern-guide.md)
- [Rust Static and Dynamic Dispatch Guide](./rust-dispatch-guide.md)
- [Rust Dependency Management Guide](./rust-dependency-management-guide.md)

---

## Table of Contents

1. [Overview](#1-overview)
2. [When to Use TypeState Pattern](#2-when-to-use-typestate-pattern)
3. [Core Concepts](#3-core-concepts)
4. [Pattern 1: Two-State TypeState (RAII Pattern)](#4-pattern-1-two-state-typestate-raii-pattern)
5. [Pattern 2: Multi-State TypeState with Distinct Types](#5-pattern-2-multi-state-typestate-with-distinct-types)
6. [Pattern 3: Generic TypeState with Phantom Types](#6-pattern-3-generic-typestate-with-phantom-types)
7. [Pattern 4: TypeState with Stateful Markers](#7-pattern-4-typestate-with-stateful-markers)
8. [Pattern 5: Finite State Machines with TypeState](#8-pattern-5-finite-state-machines-with-typestate)
9. [Pattern 6: Builder Pattern with TypeState](#9-pattern-6-builder-pattern-with-typestate)
10. [Advanced Patterns](#10-advanced-patterns)
11. [Testing Strategies](#11-testing-strategies)
12. [Anti-Patterns](#12-anti-patterns)
13. [Conclusion](#13-conclusion)

---

## 1. Overview

### What is the TypeState Pattern?

The **TypeState pattern** is an API design pattern that encodes an object's **runtime state** into its **compile-time type**. This allows the Rust compiler to enforce state-based constraints, preventing invalid operations at compile time rather than runtime.

### Core Principles

1. **State in Types**: Each distinct state is represented by a unique type
2. **Compile-Time Enforcement**: Invalid state transitions fail to compile
3. **Move Semantics**: State transitions consume the old state, preventing reuse
4. **Zero Runtime Cost**: State tracking has no runtime overhead

### Key Benefits

```rust
// Without TypeState - runtime checks required
struct Connection {
    state: ConnectionState,
}

impl Connection {
    fn send(&self, data: &[u8]) -> Result<(), Error> {
        if self.state != ConnectionState::Connected {
            return Err(Error::NotConnected); // Runtime check
        }
        // Send data...
    }
}

// With TypeState - compile-time enforcement
struct Connection<S> {
    _state: PhantomData<S>,
}

impl Connection<Connected> {
    fn send(&self, data: &[u8]) {
        // Can only call this when Connected
        // No runtime check needed!
    }
}
```

### Why TypeState Matters

- **Prevents Invalid Operations**: `file.read()` after `file.close()` won't compile
- **Eliminates Runtime Checks**: No need for `if state == X` branches
- **Self-Documenting APIs**: Type signatures reveal valid operations
- **IDE Integration**: Autocomplete only shows valid methods for current state
- **No Performance Overhead**: Zero-cost abstraction via Rust's type system

### Real-World Applications

- **Protocol Implementations**: HTTP request builders, WebSocket state machines
- **Resource Management**: File handles, database connections, network sockets
- **Workflow Engines**: Multi-step processes with validation requirements
- **Hardware Interfaces**: Device initialization sequences, peripheral states
- **Security**: Authentication flows, capability-based access control
- **Data Validation**: Ensuring data passes through required validation steps

### Document Structure

This guide covers:
- Fundamental TypeState patterns from simple to complex
- PhantomData and zero-sized type techniques
- Finite state machine implementations
- Builder pattern integration with TypeState
- Async TypeState patterns
- Performance characteristics and optimization
- Testing strategies for TypeState APIs
- Common pitfalls and anti-patterns

---

## 2. When to Use TypeState Pattern

### Use TypeState When ✅

**Scenario 1: Sequential Operations Must Occur in Order**
```rust
// Database transaction must: begin → operate → commit/rollback
struct Transaction<S> { /* ... */ }

impl Transaction<Started> {
    fn execute(&mut self, sql: &str) -> Result<()> { /* ... */ }
    fn commit(self) -> Transaction<Committed> { /* ... */ }
}

// Can't commit before starting - won't compile!
```
✅ **Why**: Compiler enforces correct operation order

**Scenario 2: Resource Lifecycle Management**
```rust
// File must be opened before reading, closed after use
struct File<S> { /* ... */ }

impl File<Closed> {
    fn open(path: &Path) -> Result<File<Open>> { /* ... */ }
}

impl File<Open> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> { /* ... */ }
    fn close(self) -> File<Closed> { /* ... */ }
}
```
✅ **Why**: Prevents use-after-close bugs at compile time

**Scenario 3: Multi-Step Validation or Configuration**
```rust
// HTTP request builder requiring specific fields
struct RequestBuilder<S> { /* ... */ }

impl RequestBuilder<NoMethod> {
    fn method(self, m: Method) -> RequestBuilder<HasMethod> { /* ... */ }
}

impl RequestBuilder<HasMethod> {
    fn url(self, u: Url) -> RequestBuilder<Complete> { /* ... */ }
}

impl RequestBuilder<Complete> {
    fn send(self) -> Result<Response> { /* ... */ }
}
```
✅ **Why**: Ensures all required fields are set before sending

**Scenario 4: State Machines with Strict Transitions**
```rust
// Traffic light with enforced transition rules
enum TrafficLight<S> { /* ... */ }

impl TrafficLight<Red> {
    fn next(self) -> TrafficLight<Green> { /* ... */ }
}

impl TrafficLight<Green> {
    fn next(self) -> TrafficLight<Yellow> { /* ... */ }
}

impl TrafficLight<Yellow> {
    fn next(self) -> TrafficLight<Red> { /* ... */ }
}
```
✅ **Why**: Invalid transitions (e.g., Red → Yellow) won't compile

**Scenario 5: Authorization and Capability Control**
```rust
// Document can only be edited after authentication
struct Document<S> { /* ... */ }

impl Document<Public> {
    fn authenticate(self, creds: &Credentials) -> Result<Document<Authenticated>> 
    { /* ... */ }
}

impl Document<Authenticated> {
    fn edit(&mut self, content: &str) { /* ... */ }
    fn save(&mut self) -> Result<()> { /* ... */ }
}
```
✅ **Why**: Prevents unauthorized operations

**Scenario 6: Hardware Initialization Sequences**
```rust
// Peripheral must be: powered → configured → enabled
struct Peripheral<S> { /* ... */ }

impl Peripheral<PoweredOff> {
    fn power_on(self) -> Peripheral<PoweredOn> { /* ... */ }
}

impl Peripheral<PoweredOn> {
    fn configure(self, cfg: Config) -> Peripheral<Configured> { /* ... */ }
}

impl Peripheral<Configured> {
    fn enable(self) -> Peripheral<Enabled> { /* ... */ }
}
```
✅ **Why**: Embedded systems require strict initialization order

### Avoid TypeState When ❌

**Scenario 1: Simple Boolean Flags**
```rust
// Don't use TypeState for this:
struct Connection<S> { /* ... */ }
impl Connection<Active> { /* ... */ }
impl Connection<Inactive> { /* ... */ }

// Just use a bool instead:
struct Connection {
    active: bool,
}
```
❌ **Why**: Over-engineering for simple true/false states

**Scenario 2: Frequently Changing States**
```rust
// Don't use TypeState for rapidly changing states:
struct Player<S> { /* ... */ }
impl Player<Running> { /* ... */ }
impl Player<Jumping> { /* ... */ }
impl Player<Falling> { /* ... */ }
// Player changes state 60 times per second in game loop
```
❌ **Why**: Move semantics add overhead; runtime enum is better

**Scenario 3: States with Many Possible Transitions**
```rust
// Don't use TypeState when any state can transition to any other:
struct Workflow<S> { /* ... */ }
// If you have 10 states with 90 possible transitions between them
```
❌ **Why**: Combinatorial explosion of impl blocks; use state pattern instead

**Scenario 4: Runtime-Determined State Flows**
```rust
// Don't use TypeState when transition logic is data-driven:
struct Process<S> { /* ... */ }
// When next state depends on database query results or user input
```
❌ **Why**: TypeState is compile-time; use runtime state machines

**Scenario 5: Need to Store Multiple States**
```rust
// Don't use TypeState when you need heterogeneous collections:
let mut items: Vec<Item<???>> = vec![]; // What type goes here?
items.push(Item::<StateA>::new());
items.push(Item::<StateB>::new()); // Won't work!
```
❌ **Why**: Can't mix types in homogeneous collections without trait objects

### Decision Matrix

| Factor | TypeState ✅ | Runtime Enum ✅ |
|--------|-------------|----------------|
| **State Transitions** | Fixed at compile time | Dynamic at runtime |
| **Number of States** | Small (2-10) | Any number |
| **Transition Complexity** | Simple, linear flows | Complex, branching flows |
| **Performance** | Zero overhead | Small branch cost |
| **Memory Usage** | Zero state storage | Stores discriminant |
| **Compile Time** | Slower (more code) | Faster |
| **Binary Size** | Larger (monomorphization) | Smaller |
| **API Ergonomics** | Excellent (IDE support) | Good |
| **Error Detection** | Compile time | Runtime |
| **Collections** | Difficult (same type) | Easy (trait objects) |

---

## 3. Core Concepts

### 3.1 Zero-Sized Types (ZSTs)

**Zero-Sized Types** are types that occupy zero bytes in memory. They exist purely at the type level for compile-time tracking.

```rust
// All of these are ZSTs:
struct Empty;           // Unit struct
enum Void {}            // Empty enum (uninhabited type)
struct Marker<T>(PhantomData<T>); // Generic marker

fn main() {
    use std::mem::size_of;
    
    assert_eq!(size_of::<Empty>(), 0);
    assert_eq!(size_of::<Void>(), 0);
    assert_eq!(size_of::<Marker<u64>>(), 0);
}
```

**Why ZSTs Matter for TypeState:**
- State markers add zero runtime overhead
- Only used for compile-time type checking
- Optimizer eliminates them entirely

**Common ZST Patterns:**

```rust
// Unit structs - most common for state markers
struct StateA;
struct StateB;

// Empty enums - uninhabited types
enum NeverInstantiated {}

// Phantom types - generic markers
struct State<T> {
    _marker: PhantomData<T>,
}
```

### 3.2 PhantomData

**PhantomData** is a special marker type that tells the compiler a generic parameter is "used" even though it doesn't appear in the struct's fields.

```rust
use std::marker::PhantomData;

// Without PhantomData - compile error
struct Container<T> {
    // Error: parameter `T` is never used
}

// With PhantomData - compiles
struct Container<T> {
    data: Vec<u8>,
    _marker: PhantomData<T>, // "Uses" T at type level
}
```

**Why PhantomData is Needed:**

1. **Variance Control**: Controls how lifetimes behave
2. **Drop Check**: Ensures proper drop order
3. **Type Parameter Usage**: Satisfies "unused parameter" errors

**PhantomData in TypeState:**

```rust
struct Connection<S> {
    socket: TcpStream,
    _state: PhantomData<S>, // S is not stored, just tracked
}

// Different types:
// Connection<Disconnected>
// Connection<Connected>
// Connection<Authenticated>
```

**Common PhantomData Patterns:**

```rust
// Pattern 1: Simple marker
struct FSM<State> {
    _state: PhantomData<State>,
}

// Pattern 2: Multiple markers
struct Process<Input, Output> {
    _marker: PhantomData<(Input, Output)>,
}

// Pattern 3: With lifetime
struct Transaction<'a, S> {
    connection: &'a Connection,
    _state: PhantomData<S>,
}
```

### 3.3 Move Semantics and State Transitions

**Move semantics** are fundamental to TypeState. When a value is moved, the previous binding becomes unusable.

```rust
struct File<S> {
    fd: FileDescriptor,
    _state: PhantomData<S>,
}

impl File<Closed> {
    fn open(path: &str) -> File<Open> {
        File {
            fd: open_file(path),
            _state: PhantomData,
        }
    }
}

impl File<Open> {
    // Takes ownership of self (move)
    fn close(self) -> File<Closed> {
        close_file(self.fd);
        File {
            fd: self.fd, // Reuse same file descriptor
            _state: PhantomData, // New type
        }
    }
}

fn example() {
    let file = File::open("data.txt");
    let file = file.close(); // `file` moved, old binding invalid
    // file.read(); // Error: no method `read` on File<Closed>
}
```

**Why Moves Matter:**

1. **Prevents Double Use**: Can't use old state after transition
2. **Linear Types**: Ensures single ownership
3. **Resource Safety**: Prevents use-after-free style bugs

**Move vs Borrow in Transitions:**

```rust
impl File<Open> {
    // Move: consumes self, returns new type
    fn to_buffered(self) -> BufferedFile<Open> {
        BufferedFile::new(self)
    }
    
    // Borrow: doesn't change state
    fn metadata(&self) -> Metadata {
        read_metadata(&self.fd)
    }
    
    // Mutable borrow: modifies but doesn't transition
    fn write(&mut self, data: &[u8]) -> Result<()> {
        write_to_file(&mut self.fd, data)
    }
}
```


### 3.4 Trait Bounds and State Constraints

**Trait bounds** can restrict which states are valid for certain operations.

```rust
// Trait to mark valid states
trait ConnectionState {}

struct Disconnected;
struct Connected;
struct Authenticated;

impl ConnectionState for Disconnected {}
impl ConnectionState for Connected {}
impl ConnectionState for Authenticated {}

// Generic over any ConnectionState
struct Connection<S: ConnectionState> {
    socket: Option<TcpStream>,
    _state: PhantomData<S>,
}

// Operations available in ANY state
impl<S: ConnectionState> Connection<S> {
    fn peer_addr(&self) -> Option<SocketAddr> {
        self.socket.as_ref()?.peer_addr().ok()
    }
}

// Trait for states that can send data
trait CanSend: ConnectionState {}
impl CanSend for Connected {}
impl CanSend for Authenticated {}

// Operations available only for states that impl CanSend
impl<S: CanSend> Connection<S> {
    fn send(&mut self, data: &[u8]) -> Result<()> {
        // Only Connected and Authenticated can call this
        self.socket.as_mut().unwrap().write_all(data)?;
        Ok(())
    }
}
```

**Benefits of Trait-Based Constraints:**

1. **Shared Operations**: Define ops for multiple states
2. **State Groups**: Categorize states by capabilities
3. **Sealed Traits**: Prevent external state definitions

**Sealed Trait Pattern:**

```rust
mod private {
    pub trait Sealed {}
}

// Public trait that uses sealed trait
pub trait State: private::Sealed {}

// Only module can implement Sealed
pub struct Open;
pub struct Closed;

impl private::Sealed for Open {}
impl private::Sealed for Closed {}
impl State for Open {}
impl State for Closed {}

// Users can't implement State for their own types
```

### 3.5 Type-Level State Machines

TypeState implements **state machines** at the type level, where:
- **States** = Distinct types
- **Transitions** = Functions that change types
- **Actions** = Methods available in specific states

**State Machine Diagram to TypeState:**

```
┌─────────┐  connect   ┌───────────┐  auth    ┌──────────────┐
│  Idle   │ ────────> │ Connected │ ───────> │ Authenticated│
└─────────┘           └───────────┘          └──────────────┘
     ▲                     │                        │
     │                     │ disconnect             │ logout
     └─────────────────────┴────────────────────────┘
```

**Implementation:**

```rust
struct Session<S> {
    user_id: Option<u64>,
    _state: PhantomData<S>,
}

struct Idle;
struct Connected;
struct Authenticated;

impl Session<Idle> {
    fn new() -> Self {
        Session {
            user_id: None,
            _state: PhantomData,
        }
    }
    
    fn connect(self) -> Session<Connected> {
        Session {
            user_id: self.user_id,
            _state: PhantomData,
        }
    }
}

impl Session<Connected> {
    fn authenticate(self, user_id: u64) -> Session<Authenticated> {
        Session {
            user_id: Some(user_id),
            _state: PhantomData,
        }
    }
    
    fn disconnect(self) -> Session<Idle> {
        Session {
            user_id: self.user_id,
            _state: PhantomData,
        }
    }
}

impl Session<Authenticated> {
    fn logout(self) -> Session<Idle> {
        Session {
            user_id: None,
            _state: PhantomData,
        }
    }
    
    fn get_user_id(&self) -> u64 {
        self.user_id.unwrap()
    }
}
```

### 3.6 Compile-Time vs Runtime State

**Understanding the Difference:**

```rust
// RUNTIME STATE - stored in memory
enum RuntimeState {
    Open,
    Closed,
}

struct FileRuntime {
    fd: i32,
    state: RuntimeState, // Occupies memory
}

impl FileRuntime {
    fn read(&self, buf: &mut [u8]) -> Result<()> {
        match self.state {
            RuntimeState::Open => { /* read */ },
            RuntimeState::Closed => Err(Error::Closed), // Runtime check!
        }
    }
}

// COMPILE-TIME STATE - only in types
struct FileTypeState<S> {
    fd: i32,
    _state: PhantomData<S>, // Zero bytes!
}

impl FileTypeState<Open> {
    fn read(&self, buf: &mut [u8]) -> Result<()> {
        // No state check needed - compiler guarantees we're Open
        // This method doesn't even exist for FileTypeState<Closed>
        todo!()
    }
}
```

**Comparison:**

| Aspect | Runtime State | Compile-Time State |
|--------|---------------|-------------------|
| **Storage** | Discriminant in memory | Zero bytes (PhantomData) |
| **Checks** | Runtime branches | Compile-time type checking |
| **Errors** | Runtime panics/Results | Compile errors |
| **Performance** | Branch predictor | No branches |
| **Flexibility** | Dynamic transitions | Static transitions |
| **Collections** | Easy (homogeneous) | Hard (different types) |

**When to Mix Both:**

```rust
// Some states known at compile time, others at runtime
struct Connection<S> {
    socket: TcpStream,
    runtime_flags: ConnectionFlags, // Runtime state
    _state: PhantomData<S>,         // Compile-time state
}

struct ConnectionFlags {
    encrypted: bool,    // Determined at runtime
    compressed: bool,   // Determined at runtime
}

// Compile-time: Connected vs Disconnected
// Runtime: encrypted, compressed flags
```

---

## 4. Pattern 1: Two-State TypeState (RAII Pattern)

### 4.1 Basic Two-State Pattern

The simplest TypeState pattern has two states: **active** and **inactive** (or **open** and **closed**).

```rust
use std::marker::PhantomData;

// State markers
struct Open;
struct Closed;

// Main type - generic over state
struct Database<S> {
    connection_string: String,
    handle: Option<DbHandle>,
    _state: PhantomData<S>,
}

struct DbHandle {
    // Actual connection details
}

// Constructor - starts in Closed state
impl Database<Closed> {
    fn new(connection_string: String) -> Self {
        Database {
            connection_string,
            handle: None,
            _state: PhantomData,
        }
    }
    
    // Transition: Closed -> Open
    fn connect(self) -> Result<Database<Open>, Error> {
        let handle = establish_connection(&self.connection_string)?;
        Ok(Database {
            connection_string: self.connection_string,
            handle: Some(handle),
            _state: PhantomData,
        })
    }
}

// Operations only available when Open
impl Database<Open> {
    fn query(&self, sql: &str) -> Result<Vec<Row>> {
        let handle = self.handle.as_ref().unwrap();
        execute_query(handle, sql)
    }
    
    fn execute(&mut self, sql: &str) -> Result<u64> {
        let handle = self.handle.as_mut().unwrap();
        execute_statement(handle, sql)
    }
    
    // Transition: Open -> Closed
    fn disconnect(self) -> Database<Closed> {
        if let Some(handle) = self.handle {
            close_connection(handle);
        }
        Database {
            connection_string: self.connection_string,
            handle: None,
            _state: PhantomData,
        }
    }
}

// Usage
fn example() -> Result<()> {
    let db = Database::new("postgres://localhost/mydb".to_string());
    
    // db.query("SELECT ..."); // Error: no method `query` for Database<Closed>
    
    let db = db.connect()?;
    let results = db.query("SELECT * FROM users")?;
    
    let db = db.disconnect();
    
    // db.query("SELECT ..."); // Error: no method `query` for Database<Closed>
    
    Ok(())
}
```

**Key Characteristics:**

- ✅ Prevents operations on closed resources
- ✅ Zero runtime overhead for state checks
- ✅ Self-documenting API
- ✅ Compiler-enforced lifecycle


### 4.2 RAII Integration

**Resource Acquisition Is Initialization (RAII)** naturally pairs with TypeState.

```rust
struct File<S> {
    path: PathBuf,
    handle: Option<fs::File>,
    _state: PhantomData<S>,
}

struct Closed;
struct Open;

impl File<Closed> {
    fn new(path: PathBuf) -> Self {
        File {
            path,
            handle: None,
            _state: PhantomData,
        }
    }
    
    fn open(self) -> io::Result<File<Open>> {
        let handle = fs::File::open(&self.path)?;
        Ok(File {
            path: self.path,
            handle: Some(handle),
            _state: PhantomData,
        })
    }
}

impl File<Open> {
    fn read_to_string(&mut self) -> io::Result<String> {
        let mut contents = String::new();
        self.handle.as_mut().unwrap().read_to_string(&mut contents)?;
        Ok(contents)
    }
}

// Automatic cleanup when Open file is dropped
impl Drop for File<Open> {
    fn drop(&mut self) {
        println!("Closing file: {:?}", self.path);
        // File handle automatically closed when dropped
    }
}

// Closed files don't need cleanup
impl Drop for File<Closed> {
    fn drop(&mut self) {
        println!("File was never opened: {:?}", self.path);
    }
}
```

**Benefits of TypeState + RAII:**

1. **Automatic Resource Management**: Resources cleaned up when dropped
2. **No Use-After-Free**: Can't use file after close
3. **Explicit Lifecycles**: State transitions are visible in code
4. **Different Drop Behavior**: Each state can have custom cleanup

### 4.3 Preventing Invalid State Transitions

TypeState prevents accessing old states after transition:

```rust
fn invalid_usage() {
    let file = File::new(PathBuf::from("data.txt"));
    let file_open = file.open().unwrap();
    
    // file.open(); // Error: value moved
    // Can't reopen because original File<Closed> was consumed
    
    let contents = file_open.read_to_string().unwrap();
    drop(file_open);
    
    // file_open.read_to_string(); // Error: value moved
    // Can't read after drop
}
```

**This Prevents Common Bugs:**

```rust
// Without TypeState - runtime error possible
struct FileBad {
    fd: Option<FileDescriptor>,
}

impl FileBad {
    fn close(&mut self) {
        self.fd = None;
    }
    
    fn read(&mut self) -> Result<Vec<u8>> {
        match &self.fd {
            Some(fd) => read_from(fd),
            None => Err(Error::NotOpen), // Runtime check!
        }
    }
}

fn oops() {
    let mut file = FileBad::open("data.txt");
    file.close();
    file.read(); // Compiles! Runtime error!
}

// With TypeState - compile error
fn safe() {
    let file = File::new(PathBuf::from("data.txt"));
    let file = file.open().unwrap();
    drop(file);
    // file.read_to_string(); // Won't compile!
}
```

### 4.4 Shared Operations Across States

Some operations make sense in any state:

```rust
impl<S> File<S> {
    // Available in both Open and Closed states
    fn path(&self) -> &Path {
        &self.path
    }
    
    fn exists(&self) -> bool {
        self.path.exists()
    }
}

fn example() {
    let file = File::new(PathBuf::from("data.txt"));
    println!("Path: {:?}", file.path()); // Works on Closed
    
    let file = file.open().unwrap();
    println!("Path: {:?}", file.path()); // Works on Open
}
```

### 4.5 Real-World Example: TCP Connection

```rust
use std::net::{TcpStream, SocketAddr};
use std::io::{self, Read, Write};
use std::marker::PhantomData;

// State markers
struct Disconnected;
struct Connected;

// Connection with state
struct TcpConnection<S> {
    addr: SocketAddr,
    stream: Option<TcpStream>,
    _state: PhantomData<S>,
}

impl TcpConnection<Disconnected> {
    fn new(addr: SocketAddr) -> Self {
        TcpConnection {
            addr,
            stream: None,
            _state: PhantomData,
        }
    }
    
    fn connect(self) -> io::Result<TcpConnection<Connected>> {
        let stream = TcpStream::connect(self.addr)?;
        Ok(TcpConnection {
            addr: self.addr,
            stream: Some(stream),
            _state: PhantomData,
        })
    }
}

impl TcpConnection<Connected> {
    fn send(&mut self, data: &[u8]) -> io::Result<()> {
        self.stream.as_mut().unwrap().write_all(data)?;
        Ok(())
    }
    
    fn receive(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.stream.as_mut().unwrap().read(buf)
    }
    
    fn disconnect(self) -> TcpConnection<Disconnected> {
        // Stream is dropped here, closing connection
        TcpConnection {
            addr: self.addr,
            stream: None,
            _state: PhantomData,
        }
    }
}

// Shared operations
impl<S> TcpConnection<S> {
    fn address(&self) -> &SocketAddr {
        &self.addr
    }
}

// Automatic cleanup
impl Drop for TcpConnection<Connected> {
    fn drop(&mut self) {
        println!("Closing connection to {}", self.addr);
    }
}

// Usage
fn tcp_example() -> io::Result<()> {
    let addr = "127.0.0.1:8080".parse().unwrap();
    let conn = TcpConnection::new(addr);
    
    let mut conn = conn.connect()?;
    conn.send(b"Hello, server!")?;
    
    let mut buf = [0u8; 1024];
    let n = conn.receive(&mut buf)?;
    println!("Received {} bytes", n);
    
    let conn = conn.disconnect();
    // conn.send(...); // Error: no method `send` on Disconnected
    
    Ok(())
}
```

---

## 5. Pattern 2: Multi-State TypeState with Distinct Types

### 5.1 Linear State Progression

When operations must occur in a specific order, use distinct types for each state.

```rust
// Text processing pipeline: Raw -> Parsed -> Formatted -> Validated
struct RawText(String);
struct ParsedText(Vec<String>);
struct FormattedText(String);
struct ValidatedText(String);

impl RawText {
    fn new(text: String) -> Self {
        RawText(text)
    }
    
    // Raw -> Parsed
    fn parse(self) -> ParsedText {
        let words = self.0
            .split_whitespace()
            .map(String::from)
            .collect();
        ParsedText(words)
    }
}

impl ParsedText {
    // Parsed -> Formatted
    fn format(self) -> FormattedText {
        let formatted = self.0.join(" | ");
        FormattedText(formatted)
    }
}

impl FormattedText {
    // Formatted -> Validated
    fn validate(self) -> Result<ValidatedText, ValidationError> {
        if self.0.len() > 1000 {
            Err(ValidationError::TooLong)
        } else {
            Ok(ValidatedText(self.0))
        }
    }
}

impl ValidatedText {
    fn into_string(self) -> String {
        self.0
    }
}

// Usage - enforces pipeline order
fn process_text(input: String) -> Result<String, ValidationError> {
    let raw = RawText::new(input);
    let parsed = raw.parse();
    let formatted = parsed.format();
    let validated = formatted.validate()?;
    Ok(validated.into_string())
}

// Can't skip steps or go backwards
fn invalid() {
    let raw = RawText::new("hello world".to_string());
    // raw.format(); // Error: no method `format` on RawText
    // let validated = raw.validate(); // Error: no method `validate`
}
```


### 5.2 Branching State Transitions

Some workflows have conditional transitions where one state can transition to multiple possible next states.

```rust
// HTTP request processing with branching paths
struct Request {
    method: String,
    path: String,
}

struct RequestReceived(Request);
struct Authenticated(Request, UserId);
struct Rejected(Request, RejectionReason);

type UserId = u64;
type RejectionReason = String;

impl RequestReceived {
    fn new(request: Request) -> Self {
        RequestReceived(request)
    }
    
    // Branching transition: can go to Authenticated OR Rejected
    fn authenticate(self, credentials: &str) -> Result<Authenticated, Rejected> {
        if is_valid_credentials(credentials) {
            let user_id = get_user_id(credentials);
            Ok(Authenticated(self.0, user_id))
        } else {
            Err(Rejected(self.0, "Invalid credentials".to_string()))
        }
    }
}

impl Authenticated {
    fn process(self) -> Response {
        process_authenticated_request(&self.0, self.1)
    }
}

impl Rejected {
    fn send_error_response(self) -> Response {
        error_response(&self.0, &self.1)
    }
}

// Usage with branching
fn handle_request(request: Request, creds: &str) -> Response {
    let received = RequestReceived::new(request);
    
    match received.authenticate(creds) {
        Ok(authenticated) => authenticated.process(),
        Err(rejected) => rejected.send_error_response(),
    }
}
```

**Key Points:**
- Uses `Result` to model branching transitions
- Each branch becomes a different type
- Can't accidentally call wrong method after branch

### 5.3 Method Chaining with State Transitions

TypeState works elegantly with fluent APIs:

```rust
struct QueryBuilder<S> {
    sql: String,
    _state: PhantomData<S>,
}

struct NoTable;
struct HasTable;
struct HasWhere;
struct Complete;

impl QueryBuilder<NoTable> {
    fn new() -> Self {
        QueryBuilder {
            sql: "SELECT * FROM ".to_string(),
            _state: PhantomData,
        }
    }
    
    fn table(self, name: &str) -> QueryBuilder<HasTable> {
        QueryBuilder {
            sql: format!("{}{}", self.sql, name),
            _state: PhantomData,
        }
    }
}

impl QueryBuilder<HasTable> {
    fn where_clause(self, condition: &str) -> QueryBuilder<HasWhere> {
        QueryBuilder {
            sql: format!("{} WHERE {}", self.sql, condition),
            _state: PhantomData,
        }
    }
    
    fn build(self) -> QueryBuilder<Complete> {
        QueryBuilder {
            sql: self.sql,
            _state: PhantomData,
        }
    }
}

impl QueryBuilder<HasWhere> {
    fn and(self, condition: &str) -> QueryBuilder<HasWhere> {
        QueryBuilder {
            sql: format!("{} AND {}", self.sql, condition),
            _state: PhantomData,
        }
    }
    
    fn build(self) -> QueryBuilder<Complete> {
        QueryBuilder {
            sql: self.sql,
            _state: PhantomData,
        }
    }
}

impl QueryBuilder<Complete> {
    fn execute(self) -> Vec<Row> {
        execute_query(&self.sql)
    }
}

// Fluent usage
fn build_query() {
    let results = QueryBuilder::new()
        .table("users")
        .where_clause("age > 18")
        .and("active = true")
        .build()
        .execute();
}
```

### 5.4 Multiple Independent State Dimensions

Sometimes you need to track multiple independent state dimensions:

```rust
// Document with two independent state dimensions:
// 1. Edit state: Draft | Published
// 2. Review state: Unreviewed | Reviewed

struct Draft;
struct Published;
struct Unreviewed;
struct Reviewed;

// Use tuple of state types
struct Document<EditState, ReviewState> {
    content: String,
    _edit: PhantomData<EditState>,
    _review: PhantomData<ReviewState>,
}

// New documents are Draft + Unreviewed
impl Document<Draft, Unreviewed> {
    fn new(content: String) -> Self {
        Document {
            content,
            _edit: PhantomData,
            _review: PhantomData,
        }
    }
}

// Can edit draft documents regardless of review state
impl<R> Document<Draft, R> {
    fn edit(mut self, new_content: String) -> Self {
        self.content = new_content;
        self
    }
}

// Can review documents regardless of edit state
impl<E> Document<E, Unreviewed> {
    fn review(self, approved: bool) -> Document<E, Reviewed> {
        Document {
            content: self.content,
            _edit: PhantomData,
            _review: PhantomData,
        }
    }
}

// Can only publish reviewed drafts
impl Document<Draft, Reviewed> {
    fn publish(self) -> Document<Published, Reviewed> {
        Document {
            content: self.content,
            _edit: PhantomData,
            _review: PhantomData,
        }
    }
}

// Can't edit published documents
// (No impl for Document<Published, _>)

// Usage
fn document_workflow() {
    let doc = Document::new("Initial content".to_string());
    let doc = doc.edit("Updated content".to_string());
    let doc = doc.review(true);
    let doc = doc.publish();
    
    // doc.edit("More changes".to_string()); // Error: no method `edit`
}
```

### 5.5 Real-World Example: HTTP Response Builder

```rust
use std::collections::HashMap;

// States
struct NoStatus;
struct HasStatus { code: u16, message: String }
struct HeadersPhase { status: HasStatus, headers: HashMap<String, String> }
struct Complete { status: HasStatus, headers: HashMap<String, String>, body: Vec<u8> }

struct HttpResponse<S> {
    state: S,
}

impl HttpResponse<NoStatus> {
    fn new() -> Self {
        HttpResponse {
            state: NoStatus,
        }
    }
    
    fn status(self, code: u16, message: &str) -> HttpResponse<HasStatus> {
        HttpResponse {
            state: HasStatus {
                code,
                message: message.to_string(),
            },
        }
    }
}

impl HttpResponse<HasStatus> {
    fn headers(self) -> HttpResponse<HeadersPhase> {
        HttpResponse {
            state: HeadersPhase {
                status: self.state,
                headers: HashMap::new(),
            },
        }
    }
    
    fn body(self, body: Vec<u8>) -> HttpResponse<Complete> {
        HttpResponse {
            state: Complete {
                status: self.state,
                headers: HashMap::new(),
                body,
            },
        }
    }
}

impl HttpResponse<HeadersPhase> {
    fn header(mut self, key: &str, value: &str) -> Self {
        self.state.headers.insert(key.to_string(), value.to_string());
        self
    }
    
    fn body(self, body: Vec<u8>) -> HttpResponse<Complete> {
        HttpResponse {
            state: Complete {
                status: self.state.status,
                headers: self.state.headers,
                body,
            },
        }
    }
}

impl HttpResponse<Complete> {
    fn send(self) -> Result<(), Error> {
        // Send the complete response
        send_response(
            self.state.status.code,
            &self.state.status.message,
            &self.state.headers,
            &self.state.body,
        )
    }
}

// Usage
fn build_response() -> Result<(), Error> {
    HttpResponse::new()
        .status(200, "OK")
        .headers()
        .header("Content-Type", "application/json")
        .header("Cache-Control", "no-cache")
        .body(b"{\"message\":\"Hello\"}".to_vec())
        .send()
}
```

---

## 6. Pattern 3: Generic TypeState with Phantom Types

### 6.1 Basic Generic TypeState

Instead of separate types for each state, use a generic parameter with phantom types:

```rust
use std::marker::PhantomData;

// State markers (ZSTs)
struct Locked;
struct Unlocked;

// Generic container
struct Vault<S> {
    contents: Vec<String>,
    _state: PhantomData<S>,
}

impl Vault<Locked> {
    fn new(contents: Vec<String>) -> Self {
        Vault {
            contents,
            _state: PhantomData,
        }
    }
    
    fn unlock(self, password: &str) -> Result<Vault<Unlocked>, Error> {
        if verify_password(password) {
            Ok(Vault {
                contents: self.contents,
                _state: PhantomData,
            })
        } else {
            Err(Error::InvalidPassword)
        }
    }
}

impl Vault<Unlocked> {
    fn add_item(&mut self, item: String) {
        self.contents.push(item);
    }
    
    fn get_items(&self) -> &[String] {
        &self.contents
    }
    
    fn lock(self) -> Vault<Locked> {
        Vault {
            contents: self.contents,
            _state: PhantomData,
        }
    }
}

// Operations available in both states
impl<S> Vault<S> {
    fn item_count(&self) -> usize {
        self.contents.len()
    }
}
```

**Benefits:**
- Single type definition for all states
- Better documentation (all impls in one place)
- Easier to add cross-state operations


### 6.2 Trait-Constrained States

Use traits to group states and constrain operations:

```rust
// Base trait for all states
trait State {}

// Specific state markers
struct Idle;
struct Running;
struct Paused;
struct Stopped;

impl State for Idle {}
impl State for Running {}
impl State for Paused {}
impl State for Stopped {}

// Trait for states that can be resumed
trait Resumable: State {}
impl Resumable for Paused {}

// Trait for states that can be paused
trait Pausable: State {}
impl Pausable for Running {}

// Generic processor
struct Processor<S: State> {
    data: Vec<u8>,
    position: usize,
    _state: PhantomData<S>,
}

impl Processor<Idle> {
    fn new(data: Vec<u8>) -> Self {
        Processor {
            data,
            position: 0,
            _state: PhantomData,
        }
    }
    
    fn start(self) -> Processor<Running> {
        Processor {
            data: self.data,
            position: self.position,
            _state: PhantomData,
        }
    }
}

// Operations for any pausable state
impl<S: Pausable> Processor<S> {
    fn pause(self) -> Processor<Paused> {
        Processor {
            data: self.data,
            position: self.position,
            _state: PhantomData,
        }
    }
}

// Operations for any resumable state
impl<S: Resumable> Processor<S> {
    fn resume(self) -> Processor<Running> {
        Processor {
            data: self.data,
            position: self.position,
            _state: PhantomData,
        }
    }
}

impl Processor<Running> {
    fn process_chunk(&mut self) -> Option<&[u8]> {
        if self.position >= self.data.len() {
            return None;
        }
        let chunk = &self.data[self.position..];
        self.position += chunk.len();
        Some(chunk)
    }
    
    fn stop(self) -> Processor<Stopped> {
        Processor {
            data: self.data,
            position: self.position,
            _state: PhantomData,
        }
    }
}

// Usage
fn process_workflow(data: Vec<u8>) {
    let proc = Processor::new(data);
    let mut proc = proc.start();
    
    proc.process_chunk();
    
    let proc = proc.pause();
    let mut proc = proc.resume();
    
    proc.process_chunk();
    
    let proc = proc.stop();
}
```

### 6.3 Sealed State Trait Pattern

Prevent external crates from defining new states:

```rust
mod private {
    pub trait Sealed {}
}

// Public trait that uses sealed trait
pub trait ConnectionState: private::Sealed {}

// Only this module can implement Sealed
pub struct Connected;
pub struct Disconnected;
pub struct Connecting;

impl private::Sealed for Connected {}
impl private::Sealed for Disconnected {}
impl private::Sealed for Connecting {}

impl ConnectionState for Connected {}
impl ConnectionState for Disconnected {}
impl ConnectionState for Connecting {}

// External crates can't implement ConnectionState
// because they can't implement private::Sealed
pub struct Connection<S: ConnectionState> {
    _state: PhantomData<S>,
}
```

**Why Seal States:**
- Prevents invalid states from external crates
- Ensures your state machine logic is complete
- Allows adding states without breaking changes

### 6.4 Type-Level State Validation

Use const generics or associated types for compile-time validation:

```rust
// State with compile-time properties
trait State {
    const CAN_WRITE: bool;
    const CAN_READ: bool;
}

struct ReadOnly;
struct WriteOnly;
struct ReadWrite;

impl State for ReadOnly {
    const CAN_WRITE: bool = false;
    const CAN_READ: bool = true;
}

impl State for WriteOnly {
    const CAN_WRITE: bool = true;
    const CAN_READ: bool = false;
}

impl State for ReadWrite {
    const CAN_WRITE: bool = true;
    const CAN_READ: bool = true;
}

struct Buffer<S: State> {
    data: Vec<u8>,
    _state: PhantomData<S>,
}

impl<S: State> Buffer<S> {
    fn read(&self) -> Option<&[u8]> 
    where 
        [(); S::CAN_READ as usize]: // Const generic trick
    {
        if S::CAN_READ {
            Some(&self.data)
        } else {
            None
        }
    }
    
    fn write(&mut self, data: &[u8]) 
    where
        [(); S::CAN_WRITE as usize]:
    {
        if S::CAN_WRITE {
            self.data.extend_from_slice(data);
        }
    }
}
```

### 6.5 Real-World Example: Transaction Management

```rust
use std::marker::PhantomData;

// State markers
struct Inactive;
struct Active;
struct Committed;
struct RolledBack;

// Transaction states
trait TransactionState {}
impl TransactionState for Inactive {}
impl TransactionState for Active {}
impl TransactionState for Committed {}
impl TransactionState for RolledBack {}

// Can be finalized
trait Finalizable: TransactionState {}
impl Finalizable for Active {}

struct Transaction<S: TransactionState> {
    id: u64,
    operations: Vec<Operation>,
    _state: PhantomData<S>,
}

struct Operation {
    sql: String,
}

impl Transaction<Inactive> {
    fn begin(id: u64) -> Transaction<Active> {
        println!("BEGIN TRANSACTION {}", id);
        Transaction {
            id,
            operations: Vec::new(),
            _state: PhantomData,
        }
    }
}

impl Transaction<Active> {
    fn execute(&mut self, sql: &str) -> Result<(), Error> {
        println!("EXEC: {}", sql);
        self.operations.push(Operation {
            sql: sql.to_string(),
        });
        Ok(())
    }
    
    fn commit(self) -> Transaction<Committed> {
        println!("COMMIT TRANSACTION {}", self.id);
        for op in &self.operations {
            println!("  Applying: {}", op.sql);
        }
        Transaction {
            id: self.id,
            operations: self.operations,
            _state: PhantomData,
        }
    }
    
    fn rollback(self) -> Transaction<RolledBack> {
        println!("ROLLBACK TRANSACTION {}", self.id);
        Transaction {
            id: self.id,
            operations: self.operations,
            _state: PhantomData,
        }
    }
}

// Can't execute on committed or rolled back transactions
// (No impl for Transaction<Committed> or Transaction<RolledBack>)

// Usage
fn transfer_funds(from: u64, to: u64, amount: f64) -> Result<(), Error> {
    let mut txn = Transaction::begin(1);
    
    txn.execute(&format!("UPDATE accounts SET balance = balance - {} WHERE id = {}", amount, from))?;
    txn.execute(&format!("UPDATE accounts SET balance = balance + {} WHERE id = {}", amount, to))?;
    
    if amount > 10000.0 {
        txn.rollback();
        return Err(Error::AmountTooLarge);
    }
    
    txn.commit();
    Ok(())
}
```

---

## 7. Pattern 4: TypeState with Stateful Markers

### 7.1 States That Carry Data

State markers don't have to be empty - they can carry state-specific data:

```rust
// States with data
struct Uninitialized;

struct Configured {
    max_connections: usize,
    timeout: Duration,
}

struct Running {
    config: Configured,
    active_connections: usize,
    start_time: Instant,
}

// Server parameterized by state
struct Server<S> {
    port: u16,
    state: S, // State contains actual data!
}

impl Server<Uninitialized> {
    fn new(port: u16) -> Self {
        Server {
            port,
            state: Uninitialized,
        }
    }
    
    fn configure(
        self, 
        max_connections: usize,
        timeout: Duration,
    ) -> Server<Configured> {
        Server {
            port: self.port,
            state: Configured {
                max_connections,
                timeout,
            },
        }
    }
}

impl Server<Configured> {
    fn start(self) -> Server<Running> {
        Server {
            port: self.port,
            state: Running {
                config: self.state, // Move config into Running
                active_connections: 0,
                start_time: Instant::now(),
            },
        }
    }
    
    // Access config data
    fn max_connections(&self) -> usize {
        self.state.max_connections
    }
}

impl Server<Running> {
    fn accept_connection(&mut self) {
        if self.state.active_connections < self.state.config.max_connections {
            self.state.active_connections += 1;
        }
    }
    
    fn uptime(&self) -> Duration {
        self.state.start_time.elapsed()
    }
    
    fn current_connections(&self) -> usize {
        self.state.active_connections
    }
}

// Usage
fn server_lifecycle() {
    let server = Server::new(8080);
    
    let server = server.configure(100, Duration::from_secs(30));
    println!("Max connections: {}", server.max_connections());
    
    let mut server = server.start();
    server.accept_connection();
    println!("Active: {}", server.current_connections());
    println!("Uptime: {:?}", server.uptime());
}
```

**Benefits:**
- Different states store different data
- No wasted memory for unused fields
- Type-safe access to state-specific data


### 7.2 Progressive Data Accumulation

States can accumulate data as they progress through transitions:

```rust
// Order processing: each state adds more information
struct OrderPlaced {
    order_id: u64,
    items: Vec<Item>,
}

struct OrderPaid {
    order_info: OrderPlaced,
    payment_id: String,
    amount: f64,
}

struct OrderShipped {
    payment_info: OrderPaid,
    tracking_number: String,
    carrier: String,
}

struct OrderDelivered {
    shipment_info: OrderShipped,
    delivery_time: DateTime<Utc>,
    signature: Option<String>,
}

struct Order<S> {
    state: S,
}

impl Order<OrderPlaced> {
    fn new(order_id: u64, items: Vec<Item>) -> Self {
        Order {
            state: OrderPlaced { order_id, items },
        }
    }
    
    fn pay(self, payment_id: String, amount: f64) -> Order<OrderPaid> {
        Order {
            state: OrderPaid {
                order_info: self.state,
                payment_id,
                amount,
            },
        }
    }
}

impl Order<OrderPaid> {
    fn ship(self, tracking_number: String, carrier: String) -> Order<OrderShipped> {
        Order {
            state: OrderShipped {
                payment_info: self.state,
                tracking_number,
                carrier,
            },
        }
    }
}

impl Order<OrderShipped> {
    fn deliver(
        self, 
        delivery_time: DateTime<Utc>,
        signature: Option<String>,
    ) -> Order<OrderDelivered> {
        Order {
            state: OrderDelivered {
                shipment_info: self.state,
                delivery_time,
                signature,
            },
        }
    }
}

// Access accumulated data at final state
impl Order<OrderDelivered> {
    fn order_id(&self) -> u64 {
        self.state.shipment_info.payment_info.order_info.order_id
    }
    
    fn total_amount(&self) -> f64 {
        self.state.shipment_info.payment_info.amount
    }
    
    fn tracking_number(&self) -> &str {
        &self.state.shipment_info.tracking_number
    }
    
    fn delivery_time(&self) -> DateTime<Utc> {
        self.state.delivery_time
    }
}
```

### 7.3 Flattening Nested State Data

Deep nesting can be unwieldy. Consider flattening:

```rust
// Better: flatten the accumulated data
struct OrderDeliveredFlat {
    // Original order data
    order_id: u64,
    items: Vec<Item>,
    // Payment data
    payment_id: String,
    amount: f64,
    // Shipment data
    tracking_number: String,
    carrier: String,
    // Delivery data
    delivery_time: DateTime<Utc>,
    signature: Option<String>,
}

impl Order<OrderShipped> {
    fn deliver_flat(
        self,
        delivery_time: DateTime<Utc>,
        signature: Option<String>,
    ) -> Order<OrderDeliveredFlat> {
        Order {
            state: OrderDeliveredFlat {
                order_id: self.state.payment_info.order_info.order_id,
                items: self.state.payment_info.order_info.items,
                payment_id: self.state.payment_info.payment_id,
                amount: self.state.payment_info.amount,
                tracking_number: self.state.tracking_number,
                carrier: self.state.carrier,
                delivery_time,
                signature,
            },
        }
    }
}

// Much easier access
impl Order<OrderDeliveredFlat> {
    fn order_id(&self) -> u64 {
        self.state.order_id
    }
    
    fn total_amount(&self) -> f64 {
        self.state.amount
    }
}
```

### 7.4 Validation Data in States

Store validation results in state markers:

```rust
struct Unvalidated;

struct ValidationPassed {
    checks_performed: Vec<String>,
    timestamp: DateTime<Utc>,
}

struct ValidationFailed {
    errors: Vec<ValidationError>,
    timestamp: DateTime<Utc>,
}

struct Input<S> {
    data: String,
    state: S,
}

impl Input<Unvalidated> {
    fn new(data: String) -> Self {
        Input {
            data,
            state: Unvalidated,
        }
    }
    
    fn validate(self) -> Result<Input<ValidationPassed>, Input<ValidationFailed>> {
        let mut checks = Vec::new();
        let mut errors = Vec::new();
        
        // Perform validation checks
        if self.data.len() < 10 {
            errors.push(ValidationError::TooShort);
        } else {
            checks.push("Length check".to_string());
        }
        
        if !self.data.chars().all(|c| c.is_alphanumeric()) {
            errors.push(ValidationError::InvalidCharacters);
        } else {
            checks.push("Character check".to_string());
        }
        
        let timestamp = Utc::now();
        
        if errors.is_empty() {
            Ok(Input {
                data: self.data,
                state: ValidationPassed {
                    checks_performed: checks,
                    timestamp,
                },
            })
        } else {
            Err(Input {
                data: self.data,
                state: ValidationFailed {
                    errors,
                    timestamp,
                },
            })
        }
    }
}

impl Input<ValidationPassed> {
    fn process(self) -> ProcessedOutput {
        // Only validated input can be processed
        process_data(&self.data)
    }
    
    fn validation_report(&self) -> String {
        format!(
            "Validated at {}: passed {} checks",
            self.state.timestamp,
            self.state.checks_performed.len()
        )
    }
}

impl Input<ValidationFailed> {
    fn error_report(&self) -> String {
        format!(
            "Validation failed at {} with {} errors",
            self.state.timestamp,
            self.state.errors.len()
        )
    }
}
```

### 7.5 Real-World Example: Package Lifecycle with Chain of Custody

```rust
use std::rc::Rc;

// Type alias for cleaner code
type Timestamp = DateTime<Utc>;

// States with rich data
struct Queued {
    queued_at: Timestamp,
}

struct InTransit {
    queued: Queued,
    picked_up_at: Timestamp,
    carrier: String,
    estimated_delivery: Timestamp,
}

struct Delivered {
    transit: InTransit,
    delivered_at: Timestamp,
    signature: Option<String>,
    location: GpsCoordinates,
}

// Package with state-dependent data
struct Package<S> {
    tracking_number: String,
    sender: Address,
    recipient: Address,
    weight_kg: f64,
    state: S,
}

impl Package<Queued> {
    fn new(
        tracking_number: String,
        sender: Address,
        recipient: Address,
        weight_kg: f64,
    ) -> Self {
        Package {
            tracking_number,
            sender,
            recipient,
            weight_kg,
            state: Queued {
                queued_at: Utc::now(),
            },
        }
    }
    
    fn ship(
        self,
        carrier: String,
        estimated_delivery: Timestamp,
    ) -> Package<InTransit> {
        Package {
            tracking_number: self.tracking_number,
            sender: self.sender,
            recipient: self.recipient,
            weight_kg: self.weight_kg,
            state: InTransit {
                queued: self.state,
                picked_up_at: Utc::now(),
                carrier,
                estimated_delivery,
            },
        }
    }
}

impl Package<InTransit> {
    fn deliver(
        self,
        signature: Option<String>,
        location: GpsCoordinates,
    ) -> Package<Delivered> {
        Package {
            tracking_number: self.tracking_number,
            sender: self.sender,
            recipient: self.recipient,
            weight_kg: self.weight_kg,
            state: Delivered {
                transit: self.state,
                delivered_at: Utc::now(),
                signature,
                location,
            },
        }
    }
    
    fn current_carrier(&self) -> &str {
        &self.state.carrier
    }
    
    fn eta(&self) -> Timestamp {
        self.state.estimated_delivery
    }
}

impl Package<Delivered> {
    fn generate_report(&self) -> ShipmentReport {
        ShipmentReport {
            tracking_number: self.tracking_number.clone(),
            queued_at: self.state.transit.queued.queued_at,
            picked_up_at: self.state.transit.picked_up_at,
            delivered_at: self.state.delivered_at,
            total_transit_time: self.state.delivered_at 
                - self.state.transit.picked_up_at,
            carrier: self.state.transit.carrier.clone(),
            signature_obtained: self.state.signature.is_some(),
        }
    }
}

// Operations available in all states
impl<S> Package<S> {
    fn tracking_number(&self) -> &str {
        &self.tracking_number
    }
    
    fn weight(&self) -> f64 {
        self.weight_kg
    }
}
```

---

## 8. Pattern 5: Finite State Machines with TypeState

### 8.1 Simple FSM with TypeState

Implement a formal finite state machine using TypeState:

```rust
// Traffic light FSM
// States: Red, Yellow, Green
// Transitions: Red -> Green, Green -> Yellow, Yellow -> Red

struct Red;
struct Yellow;
struct Green;

struct TrafficLight<S> {
    location: String,
    _state: PhantomData<S>,
}

impl TrafficLight<Red> {
    fn new(location: String) -> Self {
        TrafficLight {
            location,
            _state: PhantomData,
        }
    }
    
    // Red -> Green (only valid transition from Red)
    fn next(self) -> TrafficLight<Green> {
        println!("{}: Red -> Green", self.location);
        TrafficLight {
            location: self.location,
            _state: PhantomData,
        }
    }
}

impl TrafficLight<Green> {
    // Green -> Yellow (only valid transition from Green)
    fn next(self) -> TrafficLight<Yellow> {
        println!("{}: Green -> Yellow", self.location);
        TrafficLight {
            location: self.location,
            _state: PhantomData,
        }
    }
}

impl TrafficLight<Yellow> {
    // Yellow -> Red (only valid transition from Yellow)
    fn next(self) -> TrafficLight<Red> {
        println!("{}: Yellow -> Red", self.location);
        TrafficLight {
            location: self.location,
            _state: PhantomData,
        }
    }
}

// Invalid transitions won't compile
fn traffic_light_cycle() {
    let light = TrafficLight::new("Main St & 1st Ave".to_string());
    let light = light.next(); // Red -> Green
    let light = light.next(); // Green -> Yellow
    let light = light.next(); // Yellow -> Red
    
    // light.next().next(); // Error: can't skip Yellow
}
```


### 8.2 FSM with Event-Driven Transitions

Use events to drive state transitions:

```rust
// Events
enum ConnectionEvent {
    Connect,
    Disconnect,
    Timeout,
    Reset,
}

// States
struct Idle;
struct Connecting;
struct Connected;
struct Disconnected;
struct Error;

// FSM
struct Connection<S> {
    attempt_count: usize,
    _state: PhantomData<S>,
}

impl Connection<Idle> {
    fn new() -> Self {
        Connection {
            attempt_count: 0,
            _state: PhantomData,
        }
    }
    
    fn on_event(self, event: ConnectionEvent) -> ConnectionResult {
        match event {
            ConnectionEvent::Connect => {
                ConnectionResult::Connecting(Connection {
                    attempt_count: self.attempt_count + 1,
                    _state: PhantomData,
                })
            }
            ConnectionEvent::Reset => ConnectionResult::Idle(self),
            _ => ConnectionResult::Error(Connection {
                attempt_count: self.attempt_count,
                _state: PhantomData,
            }),
        }
    }
}

impl Connection<Connecting> {
    fn on_event(self, event: ConnectionEvent) -> ConnectionResult {
        match event {
            ConnectionEvent::Connect => {
                ConnectionResult::Connected(Connection {
                    attempt_count: self.attempt_count,
                    _state: PhantomData,
                })
            }
            ConnectionEvent::Timeout if self.attempt_count < 3 => {
                ConnectionResult::Connecting(Connection {
                    attempt_count: self.attempt_count + 1,
                    _state: PhantomData,
                })
            }
            ConnectionEvent::Timeout | ConnectionEvent::Disconnect => {
                ConnectionResult::Disconnected(Connection {
                    attempt_count: self.attempt_count,
                    _state: PhantomData,
                })
            }
            _ => ConnectionResult::Error(Connection {
                attempt_count: self.attempt_count,
                _state: PhantomData,
            }),
        }
    }
}

impl Connection<Connected> {
    fn on_event(self, event: ConnectionEvent) -> ConnectionResult {
        match event {
            ConnectionEvent::Disconnect | ConnectionEvent::Timeout => {
                ConnectionResult::Disconnected(Connection {
                    attempt_count: self.attempt_count,
                    _state: PhantomData,
                })
            }
            _ => ConnectionResult::Connected(self),
        }
    }
}

// Result type to handle different outcomes
enum ConnectionResult {
    Idle(Connection<Idle>),
    Connecting(Connection<Connecting>),
    Connected(Connection<Connected>),
    Disconnected(Connection<Disconnected>),
    Error(Connection<Error>),
}
```

### 8.3 Generic FSM Framework

Create a reusable FSM framework using TypeState:

```rust
use std::marker::PhantomData;

// Trait for FSM states
trait FsmState: Sized {
    type Event;
    type Error;
}

// Generic FSM container
struct Fsm<S: FsmState, D> {
    data: D,
    _state: PhantomData<S>,
}

impl<S: FsmState, D> Fsm<S, D> {
    fn transition<N: FsmState>(self, new_state: PhantomData<N>) -> Fsm<N, D> {
        Fsm {
            data: self.data,
            _state: new_state,
        }
    }
}

// Example: Document workflow FSM
struct DraftState;
struct ReviewState;
struct PublishedState;

// Event types
enum DocumentEvent {
    Submit,
    Approve,
    Reject,
    Publish,
}

// Data carried through all states
struct DocumentData {
    title: String,
    content: String,
    author: String,
}

// Implement FsmState for each state
impl FsmState for DraftState {
    type Event = DocumentEvent;
    type Error = WorkflowError;
}

impl FsmState for ReviewState {
    type Event = DocumentEvent;
    type Error = WorkflowError;
}

impl FsmState for PublishedState {
    type Event = DocumentEvent;
    type Error = WorkflowError;
}

// State-specific operations
impl Fsm<DraftState, DocumentData> {
    fn new(title: String, content: String, author: String) -> Self {
        Fsm {
            data: DocumentData { title, content, author },
            _state: PhantomData,
        }
    }
    
    fn edit(&mut self, content: String) {
        self.data.content = content;
    }
    
    fn submit(self) -> Fsm<ReviewState, DocumentData> {
        println!("Document submitted for review");
        self.transition(PhantomData)
    }
}

impl Fsm<ReviewState, DocumentData> {
    fn approve(self) -> Fsm<PublishedState, DocumentData> {
        println!("Document approved");
        self.transition(PhantomData)
    }
    
    fn reject(self) -> Fsm<DraftState, DocumentData> {
        println!("Document rejected, back to draft");
        self.transition(PhantomData)
    }
}

impl Fsm<PublishedState, DocumentData> {
    fn view(&self) -> &str {
        &self.data.content
    }
}
```

### 8.4 FSM with Dynamic Dispatch

Sometimes you need to store FSM instances with different states in a collection. Use trait objects:

```rust
// Base trait for all states
trait State: std::fmt::Debug {
    fn name(&self) -> &'static str;
}

// Concrete states
#[derive(Debug)]
struct Idle;

#[derive(Debug)]
struct Active;

#[derive(Debug)]
struct Suspended;

impl State for Idle {
    fn name(&self) -> &'static str { "Idle" }
}

impl State for Active {
    fn name(&self) -> &'static str { "Active" }
}

impl State for Suspended {
    fn name(&self) -> &'static str { "Suspended" }
}

// Task with type-erased state
struct Task {
    id: u64,
    state: Box<dyn State>,
}

impl Task {
    fn new_idle(id: u64) -> Self {
        Task {
            id,
            state: Box::new(Idle),
        }
    }
    
    fn new_active(id: u64) -> Self {
        Task {
            id,
            state: Box::new(Active),
        }
    }
    
    fn state_name(&self) -> &'static str {
        self.state.name()
    }
}

// Can store tasks in mixed states
fn manage_tasks() {
    let tasks: Vec<Task> = vec![
        Task::new_idle(1),
        Task::new_active(2),
        Task::new_idle(3),
    ];
    
    for task in &tasks {
        println!("Task {}: {}", task.id, task.state_name());
    }
}
```

**Trade-offs of Dynamic Dispatch:**
- ✅ Can store heterogeneous states in collections
- ✅ Runtime flexibility
- ❌ Loses compile-time state guarantees
- ❌ Runtime overhead (vtable indirection)

### 8.5 Real-World Example: Protocol State Machine

```rust
// WebSocket connection state machine
use std::marker::PhantomData;

// States
struct Handshaking;
struct Open;
struct Closing { code: u16, reason: String }
struct Closed;

// Events
enum WsEvent {
    HandshakeComplete,
    MessageReceived(Vec<u8>),
    CloseRequested(u16, String),
    ConnectionLost,
}

struct WebSocket<S> {
    url: String,
    socket: Option<TcpStream>,
    _state: PhantomData<S>,
}

impl WebSocket<Handshaking> {
    fn connect(url: String) -> Result<Self, Error> {
        let socket = TcpStream::connect(&url)?;
        Ok(WebSocket {
            url,
            socket: Some(socket),
            _state: PhantomData,
        })
    }
    
    fn complete_handshake(self) -> Result<WebSocket<Open>, Error> {
        // Perform WebSocket handshake
        perform_ws_handshake(self.socket.as_ref().unwrap())?;
        
        Ok(WebSocket {
            url: self.url,
            socket: self.socket,
            _state: PhantomData,
        })
    }
}

impl WebSocket<Open> {
    fn send(&mut self, data: &[u8]) -> Result<(), Error> {
        write_frame(self.socket.as_mut().unwrap(), data)
    }
    
    fn receive(&mut self) -> Result<Vec<u8>, Error> {
        read_frame(self.socket.as_mut().unwrap())
    }
    
    fn close(self, code: u16, reason: String) -> WebSocket<Closing> {
        send_close_frame(self.socket.as_ref().unwrap(), code, &reason);
        
        WebSocket {
            url: self.url,
            socket: self.socket,
            _state: PhantomData,
        }
    }
}

impl WebSocket<Closing> {
    fn finalize(self) -> WebSocket<Closed> {
        // Wait for close acknowledgment or timeout
        wait_for_close_ack(self.socket.as_ref().unwrap());
        
        WebSocket {
            url: self.url,
            socket: None,
            _state: PhantomData,
        }
    }
}

impl WebSocket<Closed> {
    fn reconnect(self) -> Result<WebSocket<Handshaking>, Error> {
        WebSocket::connect(self.url)
    }
}

// Usage
fn websocket_session() -> Result<(), Error> {
    let ws = WebSocket::connect("ws://example.com".to_string())?;
    let mut ws = ws.complete_handshake()?;
    
    ws.send(b"Hello, server!")?;
    let response = ws.receive()?;
    
    let ws = ws.close(1000, "Normal closure".to_string());
    let ws = ws.finalize();
    
    // ws.send(...); // Error: no method `send` on WebSocket<Closed>
    
    Ok(())
}
```

---

## 9. Pattern 6: Builder Pattern with TypeState

### 9.1 Basic TypeState Builder

Combine builder pattern with TypeState to enforce required fields:

```rust
// States representing builder progress
struct NoUrl;
struct HasUrl;
struct NoMethod;
struct HasMethod;

// Builder with multiple state dimensions
struct HttpRequestBuilder<UrlState, MethodState> {
    url: Option<String>,
    method: Option<String>,
    headers: HashMap<String, String>,
    body: Option<Vec<u8>>,
    _url_state: PhantomData<UrlState>,
    _method_state: PhantomData<MethodState>,
}

impl HttpRequestBuilder<NoUrl, NoMethod> {
    fn new() -> Self {
        HttpRequestBuilder {
            url: None,
            method: None,
            headers: HashMap::new(),
            body: None,
            _url_state: PhantomData,
            _method_state: PhantomData,
        }
    }
}

impl<M> HttpRequestBuilder<NoUrl, M> {
    fn url(self, url: String) -> HttpRequestBuilder<HasUrl, M> {
        HttpRequestBuilder {
            url: Some(url),
            method: self.method,
            headers: self.headers,
            body: self.body,
            _url_state: PhantomData,
            _method_state: PhantomData,
        }
    }
}

impl<U> HttpRequestBuilder<U, NoMethod> {
    fn method(self, method: String) -> HttpRequestBuilder<U, HasMethod> {
        HttpRequestBuilder {
            url: self.url,
            method: Some(method),
            headers: self.headers,
            body: self.body,
            _url_state: PhantomData,
            _method_state: PhantomData,
        }
    }
}

// Optional parameters available in any state
impl<U, M> HttpRequestBuilder<U, M> {
    fn header(mut self, key: String, value: String) -> Self {
        self.headers.insert(key, value);
        self
    }
    
    fn body(mut self, body: Vec<u8>) -> Self {
        self.body = Some(body);
        self
    }
}

// Can only build when both URL and Method are set
impl HttpRequestBuilder<HasUrl, HasMethod> {
    fn build(self) -> HttpRequest {
        HttpRequest {
            url: self.url.unwrap(),
            method: self.method.unwrap(),
            headers: self.headers,
            body: self.body,
        }
    }
}

// Usage
fn build_request() {
    let request = HttpRequestBuilder::new()
        .url("https://api.example.com/users".to_string())
        .method("POST".to_string())
        .header("Content-Type".to_string(), "application/json".to_string())
        .body(b"{\"name\":\"Alice\"}".to_vec())
        .build();
    
    // This won't compile - missing required fields:
    // let bad = HttpRequestBuilder::new().build();
}
```


### 9.2 Sequential Builder with Mandatory Steps

Force users to call builder methods in a specific order:

```rust
// Builder states for database connection
struct Unstarted;
struct HostSet { host: String }
struct PortSet { host: String, port: u16 }
struct DatabaseSet { host: String, port: u16, database: String }
struct Complete { host: String, port: u16, database: String, username: String }

struct DbConnectionBuilder<S> {
    state: S,
}

impl DbConnectionBuilder<Unstarted> {
    fn new() -> Self {
        DbConnectionBuilder {
            state: Unstarted,
        }
    }
    
    fn host(self, host: String) -> DbConnectionBuilder<HostSet> {
        DbConnectionBuilder {
            state: HostSet { host },
        }
    }
}

impl DbConnectionBuilder<HostSet> {
    fn port(self, port: u16) -> DbConnectionBuilder<PortSet> {
        DbConnectionBuilder {
            state: PortSet {
                host: self.state.host,
                port,
            },
        }
    }
}

impl DbConnectionBuilder<PortSet> {
    fn database(self, database: String) -> DbConnectionBuilder<DatabaseSet> {
        DbConnectionBuilder {
            state: DatabaseSet {
                host: self.state.host,
                port: self.state.port,
                database,
            },
        }
    }
}

impl DbConnectionBuilder<DatabaseSet> {
    fn username(self, username: String) -> DbConnectionBuilder<Complete> {
        DbConnectionBuilder {
            state: Complete {
                host: self.state.host,
                port: self.state.port,
                database: self.state.database,
                username,
            },
        }
    }
}

impl DbConnectionBuilder<Complete> {
    fn build(self) -> DbConnection {
        DbConnection {
            host: self.state.host,
            port: self.state.port,
            database: self.state.database,
            username: self.state.username,
        }
    }
}

// Must follow exact order
fn create_connection() {
    let conn = DbConnectionBuilder::new()
        .host("localhost".to_string())
        .port(5432)
        .database("mydb".to_string())
        .username("admin".to_string())
        .build();
    
    // Can't skip or reorder:
    // DbConnectionBuilder::new().port(5432); // Error: no method `port` on Unstarted
}
```

### 9.3 Optional vs Required Fields with TypeState

Distinguish between optional and required configuration:

```rust
// Required fields tracked in type
struct NoHost;
struct HasHost(String);
struct NoPort;
struct HasPort(u16);

// Builder with required and optional fields
struct ServerConfigBuilder<H, P> {
    host_state: H,
    port_state: P,
    // Optional fields - always available
    timeout: Option<Duration>,
    max_connections: Option<usize>,
    tls_enabled: bool,
}

impl ServerConfigBuilder<NoHost, NoPort> {
    fn new() -> Self {
        ServerConfigBuilder {
            host_state: NoHost,
            port_state: NoPort,
            timeout: None,
            max_connections: None,
            tls_enabled: false,
        }
    }
}

// Setting host
impl<P> ServerConfigBuilder<NoHost, P> {
    fn host(self, host: String) -> ServerConfigBuilder<HasHost, P> {
        ServerConfigBuilder {
            host_state: HasHost(host),
            port_state: self.port_state,
            timeout: self.timeout,
            max_connections: self.max_connections,
            tls_enabled: self.tls_enabled,
        }
    }
}

// Setting port
impl<H> ServerConfigBuilder<H, NoPort> {
    fn port(self, port: u16) -> ServerConfigBuilder<H, HasPort> {
        ServerConfigBuilder {
            host_state: self.host_state,
            port_state: HasPort(port),
            timeout: self.timeout,
            max_connections: self.max_connections,
            tls_enabled: self.tls_enabled,
        }
    }
}

// Optional parameters available anytime
impl<H, P> ServerConfigBuilder<H, P> {
    fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }
    
    fn max_connections(mut self, max: usize) -> Self {
        self.max_connections = Some(max);
        self
    }
    
    fn enable_tls(mut self) -> Self {
        self.tls_enabled = true;
        self
    }
}

// Build only when required fields are set
impl ServerConfigBuilder<HasHost, HasPort> {
    fn build(self) -> ServerConfig {
        ServerConfig {
            host: self.host_state.0,
            port: self.port_state.0,
            timeout: self.timeout.unwrap_or(Duration::from_secs(30)),
            max_connections: self.max_connections.unwrap_or(100),
            tls_enabled: self.tls_enabled,
        }
    }
}

// Usage
fn configure_server() {
    let config = ServerConfigBuilder::new()
        .host("localhost".to_string())
        .timeout(Duration::from_secs(60))
        .port(8080)
        .enable_tls()
        .max_connections(200)
        .build();
}
```

### 9.4 Builder with Validation States

Add validation as a state transition:

```rust
struct Unvalidated;
struct Validated;

struct ConfigBuilder<V> {
    settings: HashMap<String, String>,
    _validation: PhantomData<V>,
}

impl ConfigBuilder<Unvalidated> {
    fn new() -> Self {
        ConfigBuilder {
            settings: HashMap::new(),
            _validation: PhantomData,
        }
    }
    
    fn set(mut self, key: String, value: String) -> Self {
        self.settings.insert(key, value);
        self
    }
    
    fn validate(self) -> Result<ConfigBuilder<Validated>, ValidationErrors> {
        let mut errors = Vec::new();
        
        // Required settings
        if !self.settings.contains_key("app_name") {
            errors.push("Missing required setting: app_name");
        }
        
        if !self.settings.contains_key("database_url") {
            errors.push("Missing required setting: database_url");
        }
        
        // Value validation
        if let Some(port) = self.settings.get("port") {
            if port.parse::<u16>().is_err() {
                errors.push("Invalid port number");
            }
        }
        
        if errors.is_empty() {
            Ok(ConfigBuilder {
                settings: self.settings,
                _validation: PhantomData,
            })
        } else {
            Err(ValidationErrors { errors })
        }
    }
}

// Can only build after validation
impl ConfigBuilder<Validated> {
    fn build(self) -> Config {
        Config {
            settings: self.settings,
        }
    }
}

// Usage
fn create_config() -> Result<Config, ValidationErrors> {
    let config = ConfigBuilder::new()
        .set("app_name".to_string(), "MyApp".to_string())
        .set("database_url".to_string(), "postgres://localhost/db".to_string())
        .set("port".to_string(), "8080".to_string())
        .validate()?  // Must validate before building
        .build();
    
    Ok(config)
}
```

### 9.5 Real-World Example: SQL Query Builder

```rust
// Query builder with TypeState enforcing SQL grammar
struct NoFrom;
struct HasFrom(String);
struct NoWhere;
struct HasWhere(String);

struct SqlQuery<F, W> {
    select_cols: Vec<String>,
    from_state: F,
    where_state: W,
    order_by: Option<String>,
    limit: Option<usize>,
}

impl SqlQuery<NoFrom, NoWhere> {
    fn select(columns: &[&str]) -> Self {
        SqlQuery {
            select_cols: columns.iter().map(|s| s.to_string()).collect(),
            from_state: NoFrom,
            where_state: NoWhere,
            order_by: None,
            limit: None,
        }
    }
}

impl<W> SqlQuery<NoFrom, W> {
    fn from(self, table: &str) -> SqlQuery<HasFrom, W> {
        SqlQuery {
            select_cols: self.select_cols,
            from_state: HasFrom(table.to_string()),
            where_state: self.where_state,
            order_by: self.order_by,
            limit: self.limit,
        }
    }
}

impl<F> SqlQuery<F, NoWhere> {
    fn where_clause(self, condition: &str) -> SqlQuery<F, HasWhere> {
        SqlQuery {
            select_cols: self.select_cols,
            from_state: self.from_state,
            where_state: HasWhere(condition.to_string()),
            order_by: self.order_by,
            limit: self.limit,
        }
    }
}

// Optional clauses available anytime
impl<F, W> SqlQuery<F, W> {
    fn order_by(mut self, column: &str) -> Self {
        self.order_by = Some(column.to_string());
        self
    }
    
    fn limit(mut self, n: usize) -> Self {
        self.limit = Some(n);
        self
    }
}

// Can only build with FROM clause (WHERE is optional)
impl<W> SqlQuery<HasFrom, W> {
    fn build(self) -> String {
        let mut sql = format!("SELECT {} FROM {}", 
            self.select_cols.join(", "),
            self.from_state.0
        );
        
        if let HasWhere(condition) = self.where_state {
            sql.push_str(&format!(" WHERE {}", condition));
        }
        
        if let Some(order) = self.order_by {
            sql.push_str(&format!(" ORDER BY {}", order));
        }
        
        if let Some(n) = self.limit {
            sql.push_str(&format!(" LIMIT {}", n));
        }
        
        sql
    }
}

// Usage
fn query_users() {
    let sql = SqlQuery::select(&["id", "name", "email"])
        .from("users")
        .where_clause("age > 18")
        .order_by("name")
        .limit(10)
        .build();
    
    println!("{}", sql);
    // Output: SELECT id, name, email FROM users WHERE age > 18 ORDER BY name LIMIT 10
    
    // This won't compile - missing FROM clause:
    // let bad = SqlQuery::select(&["*"]).build();
}
```

---

## 10. Advanced Patterns

### 10.1 Async TypeState Patterns

TypeState works with async code:

```rust
use tokio::net::TcpStream;
use std::marker::PhantomData;

struct Disconnected;
struct Connected;

struct AsyncConnection<S> {
    addr: String,
    stream: Option<TcpStream>,
    _state: PhantomData<S>,
}

impl AsyncConnection<Disconnected> {
    fn new(addr: String) -> Self {
        AsyncConnection {
            addr,
            stream: None,
            _state: PhantomData,
        }
    }
    
    async fn connect(self) -> Result<AsyncConnection<Connected>, Error> {
        let stream = TcpStream::connect(&self.addr).await?;
        Ok(AsyncConnection {
            addr: self.addr,
            stream: Some(stream),
            _state: PhantomData,
        })
    }
}

impl AsyncConnection<Connected> {
    async fn send(&mut self, data: &[u8]) -> Result<(), Error> {
        use tokio::io::AsyncWriteExt;
        self.stream.as_mut().unwrap().write_all(data).await?;
        Ok(())
    }
    
    async fn receive(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        use tokio::io::AsyncReadExt;
        Ok(self.stream.as_mut().unwrap().read(buf).await?)
    }
    
    async fn disconnect(self) -> AsyncConnection<Disconnected> {
        // Drop stream, closing connection
        AsyncConnection {
            addr: self.addr,
            stream: None,
            _state: PhantomData,
        }
    }
}

// Async usage
async fn async_session() -> Result<(), Error> {
    let conn = AsyncConnection::new("127.0.0.1:8080".to_string());
    let mut conn = conn.connect().await?;
    
    conn.send(b"Hello").await?;
    
    let mut buf = [0u8; 1024];
    let n = conn.receive(&mut buf).await?;
    
    let conn = conn.disconnect().await;
    
    Ok(())
}
```


### 10.2 TypeState with Lifetimes

Combine TypeState with lifetime tracking:

```rust
struct Borrowed<'a, S> {
    data: &'a mut Vec<u8>,
    _state: PhantomData<S>,
}

struct Idle;
struct Processing;

impl<'a> Borrowed<'a, Idle> {
    fn new(data: &'a mut Vec<u8>) -> Self {
        Borrowed {
            data,
            _state: PhantomData,
        }
    }
    
    fn start_processing(self) -> Borrowed<'a, Processing> {
        Borrowed {
            data: self.data,
            _state: PhantomData,
        }
    }
}

impl<'a> Borrowed<'a, Processing> {
    fn process(&mut self) {
        self.data.reverse();
    }
    
    fn finish(self) -> Borrowed<'a, Idle> {
        Borrowed {
            data: self.data,
            _state: PhantomData,
        }
    }
}

// Usage
fn lifetime_example() {
    let mut data = vec![1, 2, 3, 4, 5];
    
    {
        let processor = Borrowed::new(&mut data);
        let mut processor = processor.start_processing();
        processor.process();
        let processor = processor.finish();
    }
    
    println!("{:?}", data); // [5, 4, 3, 2, 1]
}
```

### 10.3 TypeState with Associated Types

Use associated types to carry additional type information:

```rust
trait StorageState {
    type Data;
}

struct Empty;
struct Filled<T> {
    _phantom: PhantomData<T>,
}

impl StorageState for Empty {
    type Data = ();
}

impl<T> StorageState for Filled<T> {
    type Data = T;
}

struct Storage<S: StorageState> {
    data: Option<S::Data>,
    _state: PhantomData<S>,
}

impl Storage<Empty> {
    fn new() -> Self {
        Storage {
            data: None,
            _state: PhantomData,
        }
    }
    
    fn fill<T>(self, value: T) -> Storage<Filled<T>> {
        Storage {
            data: Some(value),
            _state: PhantomData,
        }
    }
}

impl<T> Storage<Filled<T>> {
    fn get(&self) -> &T {
        self.data.as_ref().unwrap()
    }
    
    fn take(mut self) -> (T, Storage<Empty>) {
        let value = self.data.take().unwrap();
        (value, Storage {
            data: None,
            _state: PhantomData,
        })
    }
}

// Usage
fn storage_example() {
    let storage = Storage::new();
    let storage = storage.fill(42);
    println!("Value: {}", storage.get());
    
    let (value, storage) = storage.take();
    // Can refill with different type
    let storage = storage.fill("Hello".to_string());
}
```

### 10.4 TypeState with Marker Traits

Use marker traits to group related states:

```rust
// Marker trait for states that allow reading
trait Readable {}

// Marker trait for states that allow writing
trait Writable {}

struct Closed;
struct ReadOnly;
struct WriteOnly;
struct ReadWrite;

impl Readable for ReadOnly {}
impl Readable for ReadWrite {}
impl Writable for WriteOnly {}
impl Writable for ReadWrite {}

struct FileHandle<S> {
    path: PathBuf,
    handle: Option<File>,
    _state: PhantomData<S>,
}

// Read operations only available for Readable states
impl<S: Readable> FileHandle<S> {
    fn read(&mut self) -> io::Result<Vec<u8>> {
        let mut buffer = Vec::new();
        self.handle.as_mut().unwrap().read_to_end(&mut buffer)?;
        Ok(buffer)
    }
}

// Write operations only available for Writable states
impl<S: Writable> FileHandle<S> {
    fn write(&mut self, data: &[u8]) -> io::Result<()> {
        self.handle.as_mut().unwrap().write_all(data)
    }
}

impl FileHandle<Closed> {
    fn open_read(path: PathBuf) -> io::Result<FileHandle<ReadOnly>> {
        let handle = File::open(&path)?;
        Ok(FileHandle {
            path,
            handle: Some(handle),
            _state: PhantomData,
        })
    }
    
    fn open_write(path: PathBuf) -> io::Result<FileHandle<WriteOnly>> {
        let handle = File::create(&path)?;
        Ok(FileHandle {
            path,
            handle: Some(handle),
            _state: PhantomData,
        })
    }
    
    fn open_read_write(path: PathBuf) -> io::Result<FileHandle<ReadWrite>> {
        let handle = OpenOptions::new()
            .read(true)
            .write(true)
            .open(&path)?;
        Ok(FileHandle {
            path,
            handle: Some(handle),
            _state: PhantomData,
        })
    }
}
```

### 10.5 TypeState with const Generics

Use const generics for compile-time array sizes in states:

```rust
trait BufferState {
    const SIZE: usize;
}

struct Small;
struct Medium;
struct Large;

impl BufferState for Small {
    const SIZE: usize = 64;
}

impl BufferState for Medium {
    const SIZE: usize = 256;
}

impl BufferState for Large {
    const SIZE: usize = 1024;
}

struct Buffer<S: BufferState> {
    data: [u8; S::SIZE],
    _state: PhantomData<S>,
}

impl<S: BufferState> Buffer<S> {
    fn new() -> Self {
        Buffer {
            data: [0; S::SIZE],
            _state: PhantomData,
        }
    }
    
    fn capacity(&self) -> usize {
        S::SIZE
    }
    
    // Upgrade to larger buffer
    fn upgrade<T: BufferState>(self) -> Buffer<T> 
    where
        [(); T::SIZE]: ,
    {
        let mut new_buffer = Buffer::<T>::new();
        let copy_len = S::SIZE.min(T::SIZE);
        new_buffer.data[..copy_len].copy_from_slice(&self.data[..copy_len]);
        new_buffer
    }
}

// Usage
fn buffer_example() {
    let small = Buffer::<Small>::new();
    println!("Small capacity: {}", small.capacity());
    
    let medium = small.upgrade::<Medium>();
    println!("Medium capacity: {}", medium.capacity());
    
    let large = medium.upgrade::<Large>();
    println!("Large capacity: {}", large.capacity());
}
```

### 10.6 Nested TypeState Machines

Combine multiple TypeState machines:

```rust
// Outer state machine: Application lifecycle
struct AppStopped;
struct AppRunning<S> {
    inner_state: S,
}

// Inner state machine: Connection state
struct NoConnection;
struct HasConnection;

struct Application<S> {
    config: Config,
    state: S,
}

impl Application<AppStopped> {
    fn new(config: Config) -> Self {
        Application {
            config,
            state: AppStopped,
        }
    }
    
    fn start(self) -> Application<AppRunning<NoConnection>> {
        Application {
            config: self.config,
            state: AppRunning {
                inner_state: NoConnection,
            },
        }
    }
}

impl Application<AppRunning<NoConnection>> {
    fn connect(self) -> Result<Application<AppRunning<HasConnection>>, Error> {
        // Establish connection
        Ok(Application {
            config: self.config,
            state: AppRunning {
                inner_state: HasConnection,
            },
        })
    }
}

impl Application<AppRunning<HasConnection>> {
    fn send_data(&mut self, data: &[u8]) -> Result<(), Error> {
        // Send data over connection
        Ok(())
    }
    
    fn disconnect(self) -> Application<AppRunning<NoConnection>> {
        Application {
            config: self.config,
            state: AppRunning {
                inner_state: NoConnection,
            },
        }
    }
}

impl<S> Application<AppRunning<S>> {
    fn stop(self) -> Application<AppStopped> {
        Application {
            config: self.config,
            state: AppStopped,
        }
    }
}
```

---

## 11. Testing Strategies

### 11.1 Testing State Transitions

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_transitions() {
        // Create initial state
        let conn = Connection::new("localhost".to_string());
        
        // Test transition
        let conn = conn.connect().unwrap();
        
        // Verify state-specific behavior
        assert!(conn.send(b"test").is_ok());
        
        // Test final transition
        let conn = conn.disconnect();
        
        // Type system prevents invalid operations
        // conn.send(b"test"); // Won't compile
    }
    
    #[test]
    fn test_error_handling() {
        let conn = Connection::new("invalid".to_string());
        
        // Test error transitions
        match conn.connect() {
            Ok(_) => panic!("Should have failed"),
            Err(e) => {
                assert_eq!(e.kind(), ErrorKind::InvalidAddress);
            }
        }
    }
}
```

### 11.2 Testing Builder Patterns

```rust
#[cfg(test)]
mod builder_tests {
    use super::*;
    
    #[test]
    fn test_complete_builder() {
        let request = HttpRequestBuilder::new()
            .url("https://example.com".to_string())
            .method("GET".to_string())
            .build();
        
        assert_eq!(request.url(), "https://example.com");
        assert_eq!(request.method(), "GET");
    }
    
    #[test]
    fn test_optional_fields() {
        let request = HttpRequestBuilder::new()
            .url("https://example.com".to_string())
            .method("POST".to_string())
            .header("Content-Type".to_string(), "application/json".to_string())
            .body(b"{}".to_vec())
            .build();
        
        assert!(request.headers().contains_key("Content-Type"));
        assert_eq!(request.body().unwrap(), b"{}");
    }
    
    // Compile-time tests (won't compile if uncommented)
    // #[test]
    // fn test_missing_required_field() {
    //     let request = HttpRequestBuilder::new()
    //         .url("https://example.com".to_string())
    //         // Missing method
    //         .build(); // Error: no method `build`
    // }
}
```

### 11.3 Property-Based Testing with TypeState

```rust
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;
    
    proptest! {
        #[test]
        fn test_state_machine_properties(
            operations in prop::collection::vec(
                prop_oneof![
                    Just(Operation::Connect),
                    Just(Operation::Send),
                    Just(Operation::Disconnect),
                ],
                0..100
            )
        ) {
            let mut machine = StateMachine::new();
            
            for op in operations {
                match (machine.current_state(), op) {
                    (State::Disconnected, Operation::Connect) => {
                        machine = machine.connect();
                    }
                    (State::Connected, Operation::Send) => {
                        machine.send(b"data");
                    }
                    (State::Connected, Operation::Disconnect) => {
                        machine = machine.disconnect();
                    }
                    _ => {
                        // Invalid operation, skip
                    }
                }
            }
            
            // Machine should always be in valid state
            assert!(machine.is_valid());
        }
    }
}
```

### 11.4 Integration Testing

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    
    #[test]
    fn test_full_workflow() {
        // Test complete workflow from start to finish
        let doc = Document::new("Test".to_string(), "Content".to_string());
        
        let doc = doc.submit_for_review();
        assert_eq!(doc.status(), "Under Review");
        
        let doc = doc.approve();
        assert_eq!(doc.status(), "Approved");
        
        let doc = doc.publish();
        assert_eq!(doc.status(), "Published");
        assert!(doc.is_public());
    }
    
    #[tokio::test]
    async fn test_async_workflow() {
        let conn = AsyncConnection::new("localhost:8080".to_string());
        let mut conn = conn.connect().await.unwrap();
        
        conn.send(b"request").await.unwrap();
        
        let mut buf = [0u8; 1024];
        let n = conn.receive(&mut buf).await.unwrap();
        assert!(n > 0);
        
        conn.disconnect().await;
    }
}
```


---

## 12. Anti-Patterns

### 12.1 Over-Using TypeState for Simple Flags

**❌ Anti-Pattern:**
```rust
// Don't use TypeState for simple boolean state
struct Active;
struct Inactive;

struct Service<S> {
    _state: PhantomData<S>,
}

impl Service<Active> {
    fn deactivate(self) -> Service<Inactive> { /* ... */ }
}

impl Service<Inactive> {
    fn activate(self) -> Service<Active> { /* ... */ }
}
```

**✅ Better Approach:**
```rust
// Just use a boolean
struct Service {
    active: bool,
}

impl Service {
    fn activate(&mut self) {
        self.active = true;
    }
    
    fn deactivate(&mut self) {
        self.active = false;
    }
}
```

**Why:** TypeState overhead isn't worth it for simple toggle states.

### 12.2 Ignoring Shared State Operations

**❌ Anti-Pattern:**
```rust
// Duplicating common operations across states
impl Connection<Disconnected> {
    fn address(&self) -> &str {
        &self.addr
    }
}

impl Connection<Connected> {
    fn address(&self) -> &str {
        &self.addr
    }
}

impl Connection<Error> {
    fn address(&self) -> &str {
        &self.addr
    }
}
```

**✅ Better Approach:**
```rust
// Use generic impl for shared operations
impl<S> Connection<S> {
    fn address(&self) -> &str {
        &self.addr
    }
}
```

**Why:** DRY (Don't Repeat Yourself) principle applies to TypeState too.

### 12.3 Creating Too Many States

**❌ Anti-Pattern:**
```rust
// Excessive state granularity
struct Initial;
struct ValidationStarted;
struct ValidationStep1Complete;
struct ValidationStep2Complete;
struct ValidationStep3Complete;
struct ValidationComplete;
struct ProcessingStarted;
struct ProcessingHalfway;
struct ProcessingAlmostDone;
struct ProcessingComplete;

// 10+ states for a simple process
```

**✅ Better Approach:**
```rust
// Logical grouping of states
struct Unvalidated;
struct Validated;
struct Processing { progress: f32 }
struct Complete;

// 4 meaningful states with internal progress tracking
```

**Why:** Too many states creates maintenance burden and confusing APIs.

### 12.4 Forcing Non-Linear Workflows

**❌ Anti-Pattern:**
```rust
// Using TypeState for workflows with many branches
struct State1;
struct State2;
struct State3;
// ... State20

// Where any state can transition to any other state
// Requires 20 x 20 = 400 potential impl blocks!
```

**✅ Better Approach:**
```rust
// Use runtime state machine for complex graphs
enum WorkflowState {
    State1,
    State2,
    State3,
    // ...
}

struct Workflow {
    state: WorkflowState,
}

impl Workflow {
    fn transition(&mut self, event: Event) -> Result<(), Error> {
        self.state = match (&self.state, event) {
            (WorkflowState::State1, Event::E1) => WorkflowState::State2,
            (WorkflowState::State2, Event::E2) => WorkflowState::State3,
            // Centralized transition logic
            _ => return Err(Error::InvalidTransition),
        };
        Ok(())
    }
}
```

**Why:** TypeState is best for linear or lightly-branching flows.

### 12.5 Neglecting Error States

**❌ Anti-Pattern:**
```rust
// No error state handling
impl Connection<Connecting> {
    fn complete_handshake(self) -> Connection<Connected> {
        // What if handshake fails?
        let result = do_handshake();
        result.unwrap(); // Panic on error!
        
        Connection {
            _state: PhantomData,
        }
    }
}
```

**✅ Better Approach:**
```rust
// Explicit error state transitions
struct Failed {
    error: Error,
}

impl Connection<Connecting> {
    fn complete_handshake(self) -> Result<Connection<Connected>, Connection<Failed>> {
        match do_handshake() {
            Ok(_) => Ok(Connection { _state: PhantomData }),
            Err(e) => Err(Connection {
                _state: PhantomData,
                error: e,
            }),
        }
    }
}

impl Connection<Failed> {
    fn error(&self) -> &Error {
        &self.error
    }
    
    fn retry(self) -> Connection<Connecting> {
        Connection { _state: PhantomData }
    }
}
```

**Why:** Errors are valid states that should be modeled explicitly.

### 12.6 Mixing Runtime and Compile-Time State

**❌ Anti-Pattern:**
```rust
// Confusing mix of TypeState and runtime state
struct Connection<S> {
    state_flag: ConnectionState, // Runtime state
    _type_state: PhantomData<S>, // Compile-time state
}

enum ConnectionState {
    Active,
    Inactive,
}

// Now we have two sources of truth!
```

**✅ Better Approach:**
```rust
// Pick one: either TypeState OR runtime state
struct Connection<S> {
    // TypeState only - no runtime flag
    _state: PhantomData<S>,
}

// OR

struct Connection {
    state: ConnectionState, // Runtime state only
}
```

**Why:** Multiple sources of truth lead to inconsistencies and bugs.

### 12.7 Forgetting PhantomData Can Be Elided

**❌ Anti-Pattern:**
```rust
// Always using PhantomData even when state has data
struct Open {
    file_handle: File,
}

struct Connection<S> {
    state: S,
    _phantom: PhantomData<S>, // Redundant!
}
```

**✅ Better Approach:**
```rust
// PhantomData only needed when S isn't stored
struct Connection<S> {
    state: S, // S is used directly, no PhantomData needed
}
```

**Why:** PhantomData is only for "phantom" type parameters.

### 12.8 Not Using Sealed Traits for Public APIs

**❌ Anti-Pattern:**
```rust
// Public trait allows external states
pub trait State {}

pub struct MyType<S: State> {
    _state: PhantomData<S>,
}

// Users can now define invalid states:
// struct UserDefinedInvalidState;
// impl State for UserDefinedInvalidState {}
```

**✅ Better Approach:**
```rust
mod private {
    pub trait Sealed {}
}

pub trait State: private::Sealed {}

pub struct ValidState;
impl private::Sealed for ValidState {}
impl State for ValidState {}

// Users cannot implement State for their types
```

**Why:** Sealed traits prevent API misuse.

### 12.9 Excessive Builder Boilerplate

**❌ Anti-Pattern:**
```rust
// Manually copying all fields in every transition
impl Builder<State1> {
    fn next(self) -> Builder<State2> {
        Builder {
            field1: self.field1,
            field2: self.field2,
            field3: self.field3,
            field4: self.field4,
            field5: self.field5,
            // ... 20 more fields
            _state: PhantomData,
        }
    }
}
```

**✅ Better Approach:**
```rust
// Extract shared data into separate struct
struct BuilderData {
    field1: Type1,
    field2: Type2,
    // ... all fields
}

struct Builder<S> {
    data: BuilderData,
    _state: PhantomData<S>,
}

impl Builder<State1> {
    fn next(self) -> Builder<State2> {
        Builder {
            data: self.data, // Single line!
            _state: PhantomData,
        }
    }
}
```

**Why:** Reduce boilerplate and maintenance burden.

### 12.10 Not Documenting State Transitions

**❌ Anti-Pattern:**
```rust
// No documentation of valid transitions
impl MyType<StateA> {
    pub fn transition1(self) -> MyType<StateB> { /* ... */ }
}

impl MyType<StateB> {
    pub fn transition2(self) -> MyType<StateC> { /* ... */ }
}
```

**✅ Better Approach:**
```rust
/// State machine for MyType
/// 
/// # State Transitions
/// 
/// ```text
/// StateA --transition1--> StateB --transition2--> StateC
///   ^                        |
///   |                        |
///   +-------transition3------+
/// ```
/// 
/// # States
/// 
/// - `StateA`: Initial state, allows transition1
/// - `StateB`: Intermediate state, allows transition2 and transition3
/// - `StateC`: Final state, no transitions
impl MyType<StateA> {
    /// Transition from StateA to StateB
    /// 
    /// This validates the input and performs initialization.
    pub fn transition1(self) -> MyType<StateB> { /* ... */ }
}
```

**Why:** Users need to understand the state machine to use it correctly.

---

## 13. Conclusion

### 13.1 Key Takeaways

1. **TypeState Encodes State in Types**: Runtime state becomes compile-time type information
2. **Zero Runtime Cost**: State tracking via types has no performance overhead
3. **Compile-Time Safety**: Invalid operations and transitions fail at compile time
4. **Best for Linear Workflows**: Excels at sequential processes with clear steps
5. **Use PhantomData Wisely**: Only when generic parameters aren't stored
6. **Combine with Traits**: Group states and share operations across multiple states
7. **Builder Pattern Integration**: Enforce required fields at compile time
8. **Document Transitions**: Make state machines understandable to users

### 13.2 When to Use TypeState

**✅ Excellent Use Cases:**
- Sequential multi-step processes
- Resource lifecycle management (open/close, connect/disconnect)
- Builder patterns with required fields
- Protocol state machines
- Validation pipelines
- Authorization flows

**❌ Poor Use Cases:**
- Simple boolean flags
- Frequently changing states (60fps game loops)
- Complex state graphs with many transitions
- Runtime-determined state flows
- Heterogeneous collections

### 13.3 Decision Checklist

Use TypeState when you answer "yes" to most of these:

- [ ] States are known at compile time
- [ ] Invalid transitions should be compile errors
- [ ] State changes are infrequent relative to operations
- [ ] States have distinct valid operations
- [ ] Workflow is mostly linear or lightly branching
- [ ] You want to eliminate runtime state checks
- [ ] API users benefit from IDE autocomplete showing valid ops
- [ ] States can be modeled with 2-10 distinct types

### 13.4 Pattern Selection Guide

```rust
// Simple two-state: Distinct types
struct File<S> { _state: PhantomData<S> }
impl File<Open> { /* ... */ }
impl File<Closed> { /* ... */ }

// Linear workflow: State progression
struct Pipeline { /* ... */ }
impl RawData { fn parse(self) -> ParsedData }
impl ParsedData { fn validate(self) -> ValidatedData }

// Multiple dimensions: Generic with multiple parameters
struct Document<Edit, Review> { /* ... */ }
impl<R> Document<Draft, R> { /* ... */ }
impl<E> Document<E, Reviewed> { /* ... */ }

// State with data: Stateful markers
struct Server<S> { state: S }
impl Server<Running { connections: usize }> { /* ... */ }

// FSM: Event-driven transitions
impl Connection<Idle> {
    fn on_event(self, e: Event) -> ConnectionResult
}

// Builder: Required fields
struct Builder<Url, Method> { /* ... */ }
impl Builder<HasUrl, HasMethod> { fn build(self) -> Request }
```

### 13.5 Performance Characteristics

| Aspect | TypeState | Runtime Enum |
|--------|-----------|--------------|
| **State Storage** | 0 bytes (ZST) | 1-8 bytes (discriminant) |
| **State Checks** | 0 (compile-time) | Small branch cost |
| **Transition Cost** | Move | Assignment |
| **Binary Size** | Larger (monomorphization) | Smaller |
| **Compile Time** | Slower | Faster |
| **Runtime Speed** | Fastest | Very fast |

### 13.6 Common Patterns Summary

| Pattern | Use When | Example |
|---------|----------|---------|
| **Two-State** | Open/closed, on/off | `File<Open>`, `File<Closed>` |
| **Linear** | Sequential steps | `Raw -> Parsed -> Validated` |
| **Generic** | Many states, shared ops | `Machine<S: State>` |
| **Stateful Markers** | States carry data | `Running { uptime: Duration }` |
| **FSM** | Event-driven transitions | `State.on_event() -> NextState` |
| **Builder** | Required configuration | `Builder<HasUrl, HasMethod>` |
| **Branching** | Conditional transitions | `Result<StateA, StateB>` |
| **Async** | Async operations | `async fn connect() -> Connected` |

### 13.7 Ecosystem Examples

**Real-world crates using TypeState:**

- **serde**: Serializer state machine with typestate
- **tokio::fs::File**: File handle with open/closed states
- **hyper::Request**: Builder pattern with typestate
- **diesel**: Query builder with compile-time SQL validation
- **embedded-hal**: Hardware peripheral state machines

### 13.8 Further Reading

**Rust Resources:**
- [The Rust Book - Advanced Types](https://doc.rust-lang.org/book/ch19-04-advanced-types.html)
- [PhantomData Documentation](https://doc.rust-lang.org/std/marker/struct.PhantomData.html)
- [Zero-Sized Types](https://doc.rust-lang.org/nomicon/exotic-sizes.html#zero-sized-types-zsts)

**TypeState Papers:**
- Strom, R.E. and Yemini, S. (1986) "Typestate: A programming language concept for enhancing software reliability"
- Aldrich, J. et al. (2009) "Typestate-oriented programming"

**Related Patterns:**
- [Rust Builder Pattern Guide](./rust-builder-pattern-guide.md)
- [Rust Static and Dynamic Dispatch Guide](./rust-dispatch-guide.md)
- [Rust ADT Implementation Guide](./rust-adt-implementation-guide.md)

### 13.9 Final Recommendations

1. **Start Simple**: Begin with two-state pattern, add complexity only when needed
2. **Benchmark If Concerned**: TypeState's zero-cost is true, but measure your use case
3. **Document Well**: State machines can be complex; good docs are essential
4. **Test Thoroughly**: Compile-time safety doesn't eliminate need for tests
5. **Consider Ergonomics**: Balance safety with usability for your API users
6. **Use Sealed Traits**: Prevent API misuse in public libraries
7. **Combine Patterns**: TypeState works well with Builder, Strategy, and RAII
8. **Profile Compile Times**: Many states can slow compilation
9. **Leverage IDE**: Type-driven development shines with TypeState
10. **Share Knowledge**: TypeState is powerful but less well-known; teach your team

### 13.10 Verification Checklist

Before shipping TypeState APIs, verify:

- [ ] All state transitions are documented
- [ ] Invalid transitions fail to compile
- [ ] Shared operations use generic impls
- [ ] States are sealed (for public APIs)
- [ ] Error states are modeled explicitly
- [ ] Examples show common workflows
- [ ] Tests cover all valid transition paths
- [ ] rustdoc renders state machine clearly
- [ ] Compile errors are understandable
- [ ] No runtime state checks remain

---

**Document Version:** 1.0  
**Last Updated:** 2026-01-25  
**Maintainer:** AirsSpec Development Team

