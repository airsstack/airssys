# Rust Smart Pointers Guide

Smart pointers are data structures that act like pointers but have additional metadata and capabilities. Unlike regular references (`&T`), smart pointers **own** the data they point to and manage memory automatically through Rust's ownership system.

## Core Concept

The key distinction between references and smart pointers:

| Type | Ownership | Heap/Stack | Use Case |
|------|-----------|------------|----------|
| `&T` | Borrows (no ownership) | Points to either | Temporary access to data |
| `Box<T>` | Owns | Heap | Single ownership, heap allocation |
| `Rc<T>` | Shared ownership | Heap | Multiple owners (single-threaded) |
| `Arc<T>` | Shared ownership | Heap | Multiple owners (thread-safe) |
| `RefCell<T>` | Owns with interior mutability | Either | Runtime-checked mutable borrowing |

## Box<T> - Heap Allocation with Single Ownership

### What It Does

`Box<T>` allocates data on the heap and provides a pointer to it. The box owns the data exclusively.

### When to Use

1. **Large data that would overflow the stack**
   ```rust
   // Stack overflow risk with large array
   let data = [0u8; 1_000_000]; // BAD: Too large for stack

   // Use Box to store on heap
   let data = Box::new([0u8; 1_000_000]); // GOOD: Heap-allocated
   ```

2. **Recursive types** (types that contain themselves)
   ```rust
   // Won't compile - infinite size
   struct List {
       value: i32,
       next: List,  // ERROR: Recursive type has infinite size
   }

   // Box provides indirection with known size
   struct List {
       value: i32,
       next: Option<Box<List>>,  // GOOD: Known size (pointer size)
   }
   ```

3. **Trait objects** (dynamic dispatch)
   ```rust
   trait Animal {
       fn make_sound(&self);
   }

   // Box allows storing different types implementing Animal
   let animals: Vec<Box<dyn Animal>> = vec![
       Box::new(Dog),
       Box::new(Cat),
   ];
   ```

### Performance Characteristics

- **Allocation**: One heap allocation
- **Deallocation**: Automatic when box goes out of scope
- **Overhead**: Minimal (pointer size + heap metadata)

## Rc<T> - Single-Threaded Shared Ownership

### What It Does

`Rc<T>` (Reference Counted) enables multiple owners of the same data through reference counting. When the last `Rc` is dropped, the data is freed.

### When to Use

Use `Rc<T>` when:
- Multiple parts of your program need to read the same data
- You can't determine at compile time which part will finish last
- You're in a single-threaded context

```rust
use std::rc::Rc;

struct Node {
    value: i32,
    children: Vec<Rc<Node>>,
}

let shared = Rc::new(Node { value: 5, children: vec![] });
let child1 = Rc::clone(&shared); // Increments reference count
let child2 = Rc::clone(&shared); // Now 3 owners

// All three can read the data
println!("{}", shared.value);
println!("{}", child1.value);
println!("{}", child2.value);
```

### Important Limitations

- **Immutable only**: `Rc<T>` only provides shared immutable access
- **Single-threaded**: Not thread-safe (won't compile if sent across threads)
- **Cycles**: Can create memory leaks with reference cycles

### Common Pattern: Rc<RefCell<T>>

Combine with `RefCell` for shared mutable data:

```rust
use std::rc::Rc;
use std::cell::RefCell;

let data = Rc::new(RefCell::new(vec![1, 2, 3]));
let data_clone = Rc::clone(&data);

// Mutate through one reference
data.borrow_mut().push(4);

// See changes through another reference
println!("{:?}", data_clone.borrow()); // [1, 2, 3, 4]
```

## Arc<T> - Thread-Safe Shared Ownership

### What It Does

`Arc<T>` (Atomic Reference Counted) is the thread-safe version of `Rc<T>`. It uses atomic operations for the reference count, allowing safe sharing across threads.

### When to Use

**Only use `Arc` when you need concurrent shared ownership:**

```rust
use std::sync::Arc;
use std::thread;

let data = Arc::new(vec![1, 2, 3]);

let handles: Vec<_> = (0..3).map(|i| {
    let data = Arc::clone(&data);
    thread::spawn(move || {
        println!("Thread {}: {:?}", i, data);
    })
}).collect();

for handle in handles {
    handle.join().unwrap();
}
```

### Performance Cost

`Arc` has overhead compared to `Rc`:
- Atomic operations are slower than regular operations
- Memory barriers for synchronization
- Cache line contention in multi-threaded scenarios

**Guideline**: Prefer `Rc` for single-threaded code. Only use `Arc` when crossing thread boundaries.

### Common Pattern: Arc<Mutex<T>> or Arc<RwLock<T>>

For shared mutable state across threads:

```rust
use std::sync::{Arc, Mutex};
use std::thread;

let counter = Arc::new(Mutex::new(0));
let mut handles = vec![];

for _ in 0..10 {
    let counter = Arc::clone(&counter);
    let handle = thread::spawn(move || {
        let mut num = counter.lock().unwrap();
        *num += 1;
    });
    handles.push(handle);
}

for handle in handles {
    handle.join().unwrap();
}

println!("Result: {}", *counter.lock().unwrap()); // Result: 10
```

## RefCell<T> - Interior Mutability

### What It Does

`RefCell<T>` provides interior mutability - the ability to mutate data even when there are immutable references to it. Borrowing rules are enforced at **runtime** instead of compile time.

### When to Use

1. **When you need to mutate data through a shared reference**
   ```rust
   use std::cell::RefCell;

   struct Config {
       cache: RefCell<HashMap<String, String>>,
   }

   impl Config {
       fn get(&self, key: &str) -> String {
           // Can mutate cache even though &self is immutable
           if let Some(value) = self.cache.borrow().get(key) {
               return value.clone();
           }

           let value = expensive_computation(key);
           self.cache.borrow_mut().insert(key.to_string(), value.clone());
           value
       }
   }
   ```

2. **Mock objects in tests**
   ```rust
   use std::cell::RefCell;

   struct MockLogger {
       messages: RefCell<Vec<String>>,
   }

   impl MockLogger {
       fn log(&self, msg: &str) {
           // Mutate through &self
           self.messages.borrow_mut().push(msg.to_string());
       }

       fn assert_logged(&self, msg: &str) {
           assert!(self.messages.borrow().contains(&msg.to_string()));
       }
   }
   ```

### Runtime Safety

`RefCell` enforces borrowing rules at runtime:

```rust
use std::cell::RefCell;

let data = RefCell::new(5);

let r1 = data.borrow();     // Immutable borrow
let r2 = data.borrow();     // OK: Multiple immutable borrows

// This will PANIC at runtime
let r3 = data.borrow_mut(); // ERROR: Already borrowed
```

### Performance

- **Compile time**: No overhead
- **Runtime**: Small overhead for borrow checking (typically just a counter)
- **Risk**: Can panic if borrowing rules are violated at runtime

## Weak<T> - Breaking Reference Cycles

### What It Does

`Weak<T>` is a non-owning reference that works with `Rc` or `Arc`. It doesn't prevent the data from being dropped.

### When to Use

Use `Weak` to prevent reference cycles:

```rust
use std::rc::{Rc, Weak};
use std::cell::RefCell;

struct Node {
    value: i32,
    parent: RefCell<Weak<Node>>,    // Weak to prevent cycle
    children: RefCell<Vec<Rc<Node>>>, // Strong references
}

let parent = Rc::new(Node {
    value: 1,
    parent: RefCell::new(Weak::new()),
    children: RefCell::new(vec![]),
});

let child = Rc::new(Node {
    value: 2,
    parent: RefCell::new(Rc::downgrade(&parent)), // Create weak reference
    children: RefCell::new(vec![]),
});

parent.children.borrow_mut().push(Rc::clone(&child));

// Access parent from child (if it still exists)
if let Some(p) = child.parent.borrow().upgrade() {
    println!("Parent value: {}", p.value);
}
```

## Decision Tree: Which Smart Pointer to Use?

```
Do you need to share ownership?
│
├─ No → Use Box<T> if you need heap allocation
│       Use regular ownership (T) otherwise
│
└─ Yes → Is it single-threaded?
    │
    ├─ Yes → Do you need mutability?
    │   │
    │   ├─ No  → Use Rc<T>
    │   └─ Yes → Use Rc<RefCell<T>>
    │
    └─ No (multi-threaded) → Do you need mutability?
        │
        ├─ No  → Use Arc<T>
        └─ Yes → Use Arc<Mutex<T>> or Arc<RwLock<T>>
```

## Project Guidelines

Based on this project's architecture and policies:

### 1. Prefer Owned Values

Most of this CLI tool's code should use owned values or references, not smart pointers:

```rust
// GOOD: Pass ownership or borrow
fn process(data: String) { }
fn process_ref(data: &str) { }

// AVOID: Unnecessary smart pointer
fn process_boxed(data: Box<String>) { }  // Why Box? Just use String
```

### 2. Minimal Arc Usage

From the [rust-dispatch-guide.md](rust-dispatch-guide.md), this project prefers static dispatch and owned values:

```rust
// GOOD: Static dispatch with owned/borrowed values
fn execute<F: FileSystem>(fs: &F) { }

// AVOID: Unnecessary Arc
fn execute(fs: Arc<dyn FileSystem>) { }  // Not needed for this CLI
```

**When Arc is acceptable:**
- Sharing state across actual concurrent threads
- Plugin systems requiring runtime dispatch
- Not applicable to most of this codebase

### 3. Box for Trait Objects Only When Needed

Use `Box<dyn Trait>` only when dynamic dispatch is truly required:

```rust
// GOOD: Static dispatch (preferred)
fn log<L: Logger>(logger: &L, msg: &str) {
    logger.info(msg);
}

// ACCEPTABLE: Dynamic dispatch when heterogeneous collection needed
let loggers: Vec<Box<dyn Logger>> = vec![
    Box::new(ConsoleLogger),
    Box::new(FileLogger::new("log.txt")),
];
```

### 4. RefCell for Interior Mutability Patterns

Acceptable uses in this project:
- Mock objects in tests
- Caching/memoization
- Event listeners/observers

```rust
// Test mock with RefCell
struct MockFileSystem {
    calls: RefCell<Vec<String>>,
}

impl FileSystem for MockFileSystem {
    fn read(&self, path: &Path) -> Result<String> {
        self.calls.borrow_mut().push(format!("read: {:?}", path));
        Ok("test".to_string())
    }
}
```

## Common Anti-Patterns

### 1. Unnecessary Boxing

```rust
// BAD: Pointless Box
fn get_name() -> Box<String> {
    Box::new(String::from("name"))
}

// GOOD: Just return the String
fn get_name() -> String {
    String::from("name")
}
```

### 2. Arc Without Threads

```rust
// BAD: Arc in single-threaded code
fn process_data(data: Arc<Vec<u8>>) {
    // No threads, no sharing needed
}

// GOOD: Use borrowed reference
fn process_data(data: &[u8]) {
    // Cheaper and simpler
}
```

### 3. Rc Cycles Without Weak

```rust
// BAD: Creates memory leak
struct Node {
    next: Option<Rc<RefCell<Node>>>,
    prev: Option<Rc<RefCell<Node>>>,  // Cycle!
}

// GOOD: Break cycle with Weak
struct Node {
    next: Option<Rc<RefCell<Node>>>,
    prev: Option<Weak<RefCell<Node>>>,  // No leak
}
```

## Summary Table

| Type | Thread-Safe | Mutability | Reference Count | Use Case |
|------|-------------|------------|-----------------|----------|
| `Box<T>` | ✅ | Mutable | N/A (single owner) | Heap allocation, trait objects |
| `Rc<T>` | ❌ | Immutable | Yes | Single-threaded shared ownership |
| `Arc<T>` | ✅ | Immutable | Yes (atomic) | Multi-threaded shared ownership |
| `RefCell<T>` | ❌ | Runtime-checked | N/A | Interior mutability pattern |
| `Mutex<T>` | ✅ | Mutable | N/A | Thread-safe interior mutability |
| `RwLock<T>` | ✅ | Multiple readers OR one writer | N/A | Optimized read-heavy thread-safe access |
| `Weak<T>` | Depends on Rc/Arc | Same as Rc/Arc | Non-owning | Breaking reference cycles |

## References

- [Smart Pointers - The Rust Book](https://doc.rust-lang.org/book/ch15-00-smart-pointers.html)
- [Box, Rc, and Arc Explained - Visual Guide (Jan 2026)](https://medium.com/@premchandak_11/box-rc-and-arc-explained-a-visual-guide-to-rust-smart-pointers-eb2563822a87)
- [Smart Pointers Demystified - DEV Community](https://dev.to/sgchris/smart-pointers-demystified-box-rc-and-refcell-27k)
- [Understanding Rust's Smart Pointers - Boardor](https://boardor.com/blog/understanding-rusts-smart-pointers-box-rc-and-refcell)
- [Mastering Rust Smart Pointers - Medium](https://basillica.medium.com/mastering-rust-smart-pointers-a-complete-guide-to-box-rc-arc-and-more-ccc61c9b197c)
