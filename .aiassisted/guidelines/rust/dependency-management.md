# ⚠️ THIS DOCUMENT HAS BEEN SUPERSEDED

**New Guide:** See `rust-dependency-injection-dip-guide.md` for comprehensive, Rust-specific DI/DIP patterns.

**Why the Change:**
- This document provides high-level SOLID theory
- The new guide provides practical, Rust-specific DI patterns based on community best practices
- The new guide includes working code examples for: Builder, Factory, Constructor Injection, Mocking

**For New Work:**
- Use `.aiassisted/guidelines/rust/rust-dependency-injection-dip-guide.md` as the authority document
- This document is retained for historical reference only

---

# Dependency Management Guidelines

## Overview

This document provides comprehensive guidelines for managing dependencies in software projects, following **SOLID** principles, **Dependency Inversion Principle (DIP)**, and **Dependency Injection (DI)** patterns.

These guidelines are designed to be project-agnostic and applicable to any software project, particularly those using statically-typed languages.

---

## Core Concepts

### 1. Dependency Inversion Principle (DIP)

The **Dependency Inversion Principle** is the "D" in SOLID principles and states:

> **High-level modules should not depend on low-level modules. Both should depend on abstractions.**
> 
> **Abstractions should not depend on details. Details should depend on abstractions.**

#### Key Points:
- Traditional layered architecture: High-level modules directly depend on low-level modules
- Dependency inversion: Both high-level and low-level modules depend on abstractions (interfaces)
- Inverts the dependency direction from top-to-bottom to bottom-to-top
- Enables loose coupling and better testability

#### Traditional Layers Pattern (Without DIP):

```
┌─────────────────────────────┐
│   High-Level Module (Policy)│
│                             │
│  ──depends on──►            │
└─────────────────────────────┘
          │
          │ depends on
          ↓
┌─────────────────────────────┐
│   Low-Level Module (Detail) │
└─────────────────────────────┘
```

**Problem:** High-level module is tightly coupled to low-level implementation details.

#### Dependency Inversion Pattern (With DIP):

```
┌─────────────────────────────┐
│   High-Level Module (Policy)│
│                             │
│  ──depends on──►            │
└─────────────────────────────┘
          │
          │ depends on
          ↓
┌─────────────────────────────┐
│      Abstraction (Trait)    │
│          Interface          │
└─────────────────────────────┘
          ▲
          │ implements
          │
┌─────────────────────────────┐
│   Low-Level Module (Detail) │
└─────────────────────────────┘
```

**Solution:** Both modules depend on the same abstraction, enabling flexibility.

---

### 2. Dependency Injection (DI)

**Dependency Injection** is a technique where an object receives other objects that it requires, rather than creating them internally.

#### Roles in Dependency Injection:

1. **Service**: A class/object containing useful functionality
2. **Client**: A class that uses services (has dependencies)
3. **Interface/Abstraction**: Defines the contract between service and client
4. **Injector/Container**: Creates and connects services to clients

#### Types of Dependency Injection:

| Type | Description | When to Use |
|------|-------------|-------------|
| **Constructor Injection** | Dependencies passed through constructor | Required dependencies, ensures object is always in valid state |
| **Method Injection** | Dependencies passed as method arguments | Temporary dependencies, optional functionality |
| **Setter Injection** | Dependencies set via setter method | Optional dependencies, may change during object lifetime |
| **Interface Injection** | Dependency provides injection method | Complex dependency graphs, specialized scenarios |

#### Constructor Injection (Recommended):

```rust
// GOOD: Constructor injection
struct Client {
    service: Arc<dyn Service>,  // ← Abstraction, not concrete type
}

impl Client {
    pub fn new(service: Arc<dyn Service>) -> Self {
        Self { service }
    }
}
```

**Benefits:**
- Ensures dependencies are always valid
- Makes dependencies explicit and required
- Easier to understand and maintain

---

## Problem: Circular Dependencies

### The Issue

Circular dependencies occur when modules depend directly on **REAL IMPLEMENTATIONS** of other modules, creating tight coupling and making code difficult to test and maintain.

### Example of Circular Dependency

```rust
// BAD: Circular dependency
// Module A depends on Module B's real implementation
mod module_a {
    use crate::module_b::RealServiceB;
    
    pub struct ComponentA {
        service: RealServiceB,  // ← Direct dependency on real impl
    }
}

// Module B knows about Module A
mod module_b {
    use crate::module_a::ComponentA;
    
    pub struct RealServiceB { /* real implementation */ }
}
```

**Problems:**
1. **Tight coupling:** ComponentA knows about RealServiceB implementation details
2. **Hard to test:** Can't easily mock RealServiceB
3. **Hard to swap:** Must change ComponentA if RealServiceB changes
4. **No flexibility:** Can't use different implementations (mock, test, production)

---

## Solution: Dependency Inversion + Dependency Injection

### The Pattern

```rust
// GOOD: Dependency inversion via abstraction
mod core {
    // ABSTRACTION (no external deps, no implementation logic)
    pub trait Service: Send + Sync {
        fn execute(&self, input: &str) -> Result<String, Error>;
    }
}

// Implementation in separate module (HAS external deps, HAS async code)
mod service_impl {
    use crate::core::Service;
    use tokio::sync::Mutex;  // External crate: OK for implementation
    
    pub struct RealService {
        state: Mutex<InternalState>,  // Real implementation with external deps
    }
    
    impl Service for RealService {
        fn execute(&self, input: &str) -> Result<String, Error> {
            // Implementation with external dependencies
            todo!()
        }
    }
}

// Component depends on ABSTRACTION, not implementation
mod client {
    use crate::core::Service;  // ← Depends on trait, not implementation
    
    pub struct ComponentA {
        service: Arc<dyn Service>,  // ← Abstraction as type parameter
    }
    
    impl ComponentA {
        pub fn new(service: Arc<dyn Service>) -> Self {
            Self { service }
        }
        
        fn do_something(&self) {
            self.service.execute("input");  // ← Works with ANY implementation
        }
    }
}
```

**Benefits:**
1. **Loose coupling:** ComponentA depends on trait, not implementation
2. **Testable:** Easy to mock Service trait in unit tests
3. **Flexible:** Can pass different implementations (MockService, RealService, etc.)
4. **Swappable:** Can change implementations without changing ComponentA

---

## Rules for Applying DIP and DI

### Rule 1: Abstractions Should Be Dependency-Free

**ALLOWED:**
- Trait definitions (method signatures, no implementation logic)
- Associated types (types without external dependencies)
- Enum definitions (variants without external dependencies)
- Error types (custom errors without external dependencies)

**FORBIDDEN:**
- Implementation logic in traits
- External crate dependencies (tokio, serde, etc.) in abstractions
- Async runtime code in abstractions
- Concrete type implementations

**Rationale:**
- Abstractions should be importable by any module
- Keeping abstractions dependency-free prevents transitive dependencies
- Traits have no external deps, so they're safe to import anywhere

### Rule 2: Dependency Injection Pattern

When passing dependencies, **MUST** follow this pattern:

```rust
// GOOD: Dependency injection
struct MyComponent {
    dependency: Arc<dyn SomeTrait>,  // ← Abstraction as type parameter
}

impl MyComponent {
    pub fn new(dependency: Arc<dyn SomeTrait>) -> Self {
        Self { dependency }
    }
    
    pub fn use_dependency(&self) {
        self.dependency.do_something();  // Use through abstraction
    }
}

// BAD: Direct creation
struct MyComponent {
    dependency: SomeConcreteType,  // ← Concrete implementation created here
}

impl MyComponent {
    pub fn new() -> Self {
        Self { dependency: SomeConcreteType::new() }  // ← Tight coupling
    }
}
```

**Rationale:**
- Abstractions allow swapping implementations (testing, different configs)
- Direct creation creates tight coupling
- Follows Dependency Inversion Principle

### Rule 3: Dependency Direction

Dependencies should flow in one direction:

```
High-Level Modules
      │
      │ depends on
      ↓
   Abstractions
      ▲
      │ implements
      │
Low-Level Modules
```

**Key Points:**
- Both high-level and low-level depend on abstractions
- No module depends on another module's implementation directly
- Eliminates circular dependencies
- Enables independent development

---

## Testing Strategy

### Unit Tests: Use Mocks

Unit tests should use **MOCK IMPLEMENTATIONS** of abstractions to test logic in isolation.

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Service;
    use std::collections::VecDeque;
    
    // MOCK: Implements abstraction for testing
    struct MockService {
        calls: VecDeque<String>,
    }
    
    impl MockService {
        fn new() -> Self {
            Self { calls: VecDeque::new() }
        }
    }
    
    impl Service for MockService {
        fn execute(&self, input: &str) -> Result<String, Error> {
            // Track calls for verification
            Ok(format!("mocked: {}", input))
        }
    }
    
    #[test]
    fn test_component_with_mock_service() {
        let mock_service = Arc::new(MockService::new());
        let component = ComponentA::new(mock_service);
        
        let result = component.do_something();
        
        // Verify mock was used
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "expected value");
    }
}
```

### Integration Tests: Use Real Implementations

Integration tests should use **REAL IMPLEMENTATIONS** to test end-to-end interactions between components.

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    use crate::service_impl::RealService;
    
    #[tokio::test]
    async fn test_component_with_real_service() {
        let real_service = Arc::new(RealService::new().await.unwrap());
        let component = ComponentA::new(real_service);
        
        let result = component.do_something().await;
        
        // Verify real implementation was used
        assert!(result.is_ok());
    }
}
```

---

## Examples: Good vs Bad Patterns

### Example 1: Direct Dependency on Implementation (BAD)

```rust
// BAD: Direct dependency on real implementation
pub struct ComponentA {
    service: RealService,  // ← Concrete implementation (tight coupling)
}

impl ComponentA {
    pub fn new() -> Self {
        Self { 
            service: RealService::new()  // ← Creates dependency internally
        }
    }
    
    fn do_something(&mut self) {
        self.service.execute("input");  // → Creates tight coupling
    }
}
```

**Problems:**
- ❌ Tight coupling: ComponentA knows about RealService
- ❌ Hard to test: Can't easily mock RealService
- ❌ Hard to swap: Must change ComponentA to use different implementation
- ❌ Violates DIP: High-level depends on low-level implementation

### Example 2: Dependency Injection via Traits (GOOD)

```rust
// GOOD: Depends on trait from core
pub struct ComponentA {
    service: Arc<dyn Service>,  // ← Abstraction as type parameter
}

impl ComponentA {
    pub fn new(service: Arc<dyn Service>) -> Self {
        Self { service }  // ← Dependency injection
    }
    
    fn do_something(&self) {
        self.service.execute("input");  // ← Works with ANY implementation
    }
}
```

**Benefits:**
- ✅ Loose coupling: ComponentA depends on trait, not implementation
- ✅ Testable: Easy to mock Service trait in unit tests
- ✅ Flexible: Can pass RealService or MockService
- ✅ Follows DIP: Depends on abstraction

### Example 3: Assembly (Wiring Dependencies)

```rust
// Manual assembly at application root
pub struct Application {
    component_a: Arc<ComponentA>,
}

impl Application {
    pub async fn new() -> Result<Self, Error> {
        // Create dependencies
        let real_service = Arc::new(RealService::new().await?);
        
        // Inject into components
        let component_a = Arc::new(ComponentA::new(real_service));
        
        Ok(Self { component_a })
    }
}
```

**Benefits:**
- ✅ Clear dependency graph
- ✅ All wiring in one place
- ✅ Easy to swap implementations for testing

---

## Advantages of DIP and DI

### 1. Loose Coupling
- Modules depend on abstractions, not concrete implementations
- Changes to implementations don't affect depending modules
- Easier to understand relationships between modules

### 2. Testability
- Easy to mock dependencies for unit testing
- Test logic in isolation without external systems
- Faster test execution

### 3. Flexibility
- Can swap implementations at runtime or compile time
- Support for multiple configurations (development, testing, production)
- Enables plugin architecture

### 4. Reusability
- High-level modules can be reused with different implementations
- Low-level modules can be reused by different high-level modules
- Promotes library over application code

### 5. Maintainability
- Clear separation of concerns
- Easier to locate and fix bugs
- Reduces code duplication

### 6. Concurrent Development
- Teams can work on modules independently
- Only need to agree on interfaces
- Enables third-party plugins and extensions

---

## Disadvantages and Considerations

### Complexity
- Increased number of interfaces/traits
- More boilerplate code
- Steeper learning curve for new developers

### Debugging Difficulty
- Separates behavior from construction
- Harder to trace through code
- May require debugger tools

### Framework Dependency
- Often requires dependency injection frameworks
- Adds external dependencies
- May over-complicate simple projects

### Upfront Effort
- Requires more initial planning
- Takes longer to set up initially
- May seem like over-engineering for small projects

---

## When to Apply DIP and DI

### Apply When:
- ✅ Building complex systems with multiple layers
- ✅ Writing libraries that need to support multiple implementations
- ✅ Developing systems requiring extensive testing
- ✅ Building plugin or extension systems
- ✅ Working with teams developing in parallel
- ✅ Needing to support multiple configurations/environments

### May Skip When:
- ⚠️ Small, simple applications
- ⚠️ Prototype or proof-of-concept code
- ⚠️ Single-developer projects
- ⚠️ Applications unlikely to change or grow
- ⚠️ Performance-critical code with known implementation

---

## Best Practices Summary

### 1. Always Prefer Constructor Injection
- Ensures dependencies are always valid
- Makes dependencies explicit
- Easiest to understand and maintain

### 2. Keep Abstractions Simple
- Define minimal interfaces
- Avoid leaking implementation details
- Focus on what, not how

### 3. Use Dependency Injection Frameworks Judiciously
- Manual DI is sufficient for small projects
- Consider frameworks for large projects
- Don't let framework dictate design

### 4. Test Both With Mocks and Real Implementations
- Unit tests: Use mocks for fast, isolated tests
- Integration tests: Use real implementations for end-to-end verification

### 5. Document Abstractions Clearly
- Explain the purpose and contract
- Provide examples of expected behavior
- Document any invariants or constraints

### 6. Avoid Over-Abstraction
- Don't create interfaces for everything
- Focus on stable boundaries
- Refactor as needed, don't over-engineer upfront

---

## Verification Checklist

Before considering code complete, verify:

- [ ] No module depends directly on another module's concrete implementation
- [ ] All dependencies are injected, not created internally
- [ ] Abstractions have no external dependencies
- [ ] Constructor injection is used for required dependencies
- [ ] Unit tests use mock implementations
- [ ] Integration tests use real implementations
- [ ] Dependency direction follows DIP (high-level → abstraction ← low-level)
- [ ] All tests pass
- [ ] Code compiles without warnings

---

## References

### Core Principles
1. **Dependency Inversion Principle** - Wikipedia
2. **Dependency Injection** - Wikipedia
3. **SOLID Principles** - Wikipedia
4. Robert C. Martin - Agile Software Development, Principles, Patterns, and Practices

### Further Reading
- Martin Fowler - Inversion of Control Containers and the Dependency Injection Pattern
- Mark Seemann - Dependency Injection Principles, Practices, and Patterns
- The Dependency Injection Design Pattern - MSDN Magazine

---

## Summary

This guideline provides a comprehensive approach to managing dependencies using:

1. ✅ **Dependency Inversion Principle:** High-level and low-level modules both depend on abstractions
2. ✅ **Dependency Injection:** Pass dependencies rather than create them internally
3. ✅ **Constructor Injection:** Preferred method for dependency injection
4. ✅ **Abstraction-First Design:** Define interfaces before implementations
5. ✅ **Testing Strategy:** Unit tests use mocks, Integration tests use real implementations
6. ✅ **Verification:** Check dependencies before completing tasks

**Key Takeaway:** Depend on abstractions, not concretions. This principle leads to loosely coupled, testable, and maintainable software.
