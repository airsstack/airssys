# Modular Monolith Architecture Guidelines

> **Purpose**: This guideline provides comprehensive principles, patterns, and best practices for implementing Modular Monolith architecture in any programming language and technology stack.

---

## Table of Contents

1. [Introduction](#introduction)
2. [What is a Modular Monolith?](#what-is-a-modular-monolith)
3. [Core Principles](#core-principles)
4. [Module Design](#module-design)
5. [Module Communication](#module-communication)
6. [Benefits and Trade-offs](#benefits-and-trade-offs)
7. [Implementation Patterns](#implementation-patterns)
8. [Best Practices](#best-practices)
9. [Anti-Patterns to Avoid](#anti-patterns-to-avoid)
10. [Migration Path](#migration-path)

---

## Introduction

### Definition

A **Modular Monolith** is an architectural style that structures the application into **independent modules** with well-defined boundaries, while maintaining a **single deployment unit**. It combines the simplicity of monolithic architecture with the modularity benefits typically associated with microservices.

### Key Characteristics

- **Single Deployment Unit**: All modules are deployed together as one application
- **Logical Module Boundaries**: Clear separation between modules based on business domains
- **High Cohesion**: Related functionality grouped within modules
- **Low Coupling**: Minimal dependencies between modules
- **Explicit Interfaces**: Modules communicate through well-defined contracts
- **Data Isolation**: Each module owns and manages its own data

---

## What is a Modular Monolith?

### The Monolith Foundation

A **monolith** is a system with exactly **one deployment unit**. This doesn't imply poor design or lack of modularity - it's simply a deployment characteristic.

**Traditional Monolith Issues:**
- Intertwined code with no clear boundaries
- High coupling between components
- Difficult to understand and maintain
- Changes ripple across the entire codebase

### The Modular Enhancement

**Modular Monolith = Monolith + Modularity**

A Modular Monolith addresses traditional monolith issues by:

1. **Organizing by Business Capability** (Vertical Slices)
   - NOT by technical layers (UI, Business Logic, Data Access)
   - Group by features/domains (User Management, Orders, Payments)

2. **Enforcing Module Boundaries**
   - Modules cannot directly access internals of other modules
   - Communication only through explicit public APIs

3. **Enabling Independent Evolution**
   - Modules can be developed, tested, and modified independently
   - Changes in one module have minimal impact on others

---

## Core Principles

### 1. Module Independence and Interchangeability

**Goal**: Minimize dependencies between modules.

**Measures of Independence:**

```
Independence = f(
    number_of_dependencies,
    strength_of_dependencies,
    stability_of_dependencies
)
```

**Guidelines:**

- **Minimize Dependency Count**: Keep the number of module dependencies low
- **Weaken Dependency Strength**: Use occasional, thin interfaces rather than frequent, wide APIs
- **Depend on Stable Modules**: Prefer modules that change infrequently

**Example:**

```pseudocode
// ❌ BAD: Strong coupling - module uses many methods from another module
class OrderModule {
    function createOrder(userId) {
        // Calls 10+ methods on UserModule
        user = userModule.getUser(userId)
        preferences = userModule.getPreferences(userId)
        paymentInfo = userModule.getPaymentInfo(userId)
        address = userModule.getAddress(userId)
        // ... many more calls
    }
}

// ✅ GOOD: Weak coupling - minimal, focused interface
class OrderModule {
    function createOrder(userId) {
        // Single, focused query
        userContext = userModule.getOrderContext(userId)
        // userContext contains only what's needed for orders
    }
}
```

### 2. Vertical Slice Organization

**Principle**: Organize modules by **business capability**, not technical concerns.

**Technical Layers (❌ Avoid This):**
```
├── UI Layer (all features)
├── Business Logic Layer (all features)
└── Data Access Layer (all features)
```

When adding a feature, you must change ALL layers.

**Business Modules (✅ Use This):**
```
├── User Management Module (complete vertical slice)
├── Order Processing Module (complete vertical slice)
└── Payment Module (complete vertical slice)
```

When adding a feature, you change ONE module.

**Implementation Pattern:**

```pseudocode
// Each module is self-contained with all layers
module UserManagement {
    
    // Domain (business logic)
    namespace Domain {
        class User {
            private id
            private email
            private name
            
            function create(email, name) {
                // Business rules and validation
            }
        }
    }
    
    // Application (use cases)
    namespace Application {
        class CreateUserUseCase {
            function execute(command) {
                // Orchestrate domain logic
            }
        }
    }
    
    // Infrastructure (data access, external services)
    namespace Infrastructure {
        class UserRepository {
            function save(user) {
                // Persistence logic
            }
        }
    }
    
    // Public API (only this is exposed)
    namespace API {
        // Export only what other modules need
        export CreateUserUseCase
        export UserDTO
    }
}
```

### 3. Well-Defined Module Interface (Contract)

**Principle**: Each module MUST expose a clear, stable public API.

**Without Contract (❌):**
```
Module A → directly accesses → Module B internals
```
This leads to tight coupling and brittle code.

**With Contract (✅):**
```
Module A → calls public API → Module B (via contract)
```

**Implementation:**

```pseudocode
// Module Interface (Public Contract)
interface UserAPI {
    function createUser(command: CreateUserCommand): UserId
    function getUser(id: UserId): UserDTO
    function listUsers(query: ListUsersQuery): List<UserDTO>
}

// Module Implementation (Internal)
class UserModule implements UserAPI {
    // Internal components - NOT exposed
    private repository: UserRepository
    private eventBus: EventBus
    
    function createUser(command: CreateUserCommand): UserId {
        // Implementation details hidden
    }
}

// Other modules only depend on the interface
class OrderModule {
    private userAPI: UserAPI  // ✅ Depends on abstraction, not concrete type
    
    function createOrder(userId) {
        user = this.userAPI.getUser(userId)
    }
}
```

### 4. Encapsulation and Information Hiding

**Principle**: Hide implementation details; expose only what's necessary.

**Visibility Rules:**

```pseudocode
module UserManagement {
    // Private - not visible outside module
    private namespace Domain { }
    private namespace Infrastructure { }
    
    // Public - this is the module's interface
    public namespace API { }
    
    // Export only what's necessary
    export UserAPI, CreateUserCommand, UserDTO
    
    // Everything else stays internal
}
```

### 5. Loose Coupling, High Cohesion (GRASP Principle)

**High Cohesion**: Related functionality grouped together
**Low Coupling**: Minimal dependencies between modules

**Example:**

```pseudocode
// ✅ HIGH COHESION: User module contains all user-related logic
module UserManagement {
    class User { }
    class UserRepository { }
    class UserValidator { }
    // All user-related code in one place
}

// ❌ LOW COHESION: User logic scattered
module Validators {
    class UserValidator { }
    class OrderValidator { }
}
module Entities {
    class User { }
    class Order { }
}
```

---

## Module Design

### Module Structure

Each module follows this structure:

```
module_name/
├── api/                    # Public API (interfaces, DTOs, commands, queries)
│   ├── interfaces
│   ├── dtos
│   └── commands
├── domain/                 # Business logic (entities, value objects, domain events)
│   ├── entities
│   ├── value_objects
│   └── events
├── application/            # Use cases, application services
│   ├── commands
│   └── queries
├── infrastructure/         # Data access, external integrations
│   ├── repositories
│   └── persistence
└── tests/                  # Module-specific tests
    ├── unit_tests
    └── integration_tests
```

### Module Boundaries

**Rules for Defining Module Boundaries:**

1. **Bounded Context Alignment**: Each module represents a bounded context from Domain-Driven Design
2. **Business Capability**: Module boundaries follow business capabilities, not technical concerns
3. **Team Ownership**: Ideally, one team owns one module
4. **Change Frequency**: Code that changes together stays together

**Example:**

```
application/
├── user_management/       # User bounded context
├── order_processing/      # Order bounded context
├── payment/               # Payment bounded context
├── inventory/             # Inventory bounded context
└── shared/                # Shared kernel (minimal!)
```

### Module Dependencies

**Dependency Rules:**

```pseudocode
// ✅ ALLOWED: Module depends on shared kernel
import shared.Result
import shared.Error
import shared.EventBus

// ✅ ALLOWED: Module depends on another module's public API
import user_management.api.UserAPI
import user_management.api.UserDTO

// ❌ FORBIDDEN: Module depends on another module's internals
import user_management.domain.User          // ❌ Don't reach into internal domain!
import user_management.infrastructure.UserRepository  // ❌ Don't use internal components!
```

**Visualizing Dependencies:**

```
┌─────────────┐
│    User     │
│ Management  │
└──────┬──────┘
       │ uses public API
       ▼
┌─────────────┐
│    Order    │
│ Processing  │
└──────┬──────┘
       │ uses public API
       ▼
┌─────────────┐
│   Payment   │
└─────────────┘

All modules depend on:
┌─────────────┐
│   Shared    │
│   Kernel    │
└─────────────┘
```

---

## Module Communication

### Communication Patterns

#### 1. Synchronous Communication (Request-Response)

**Use When:**
- Immediate response needed
- Strong consistency required
- Simple query operations

**Implementation:**

```pseudocode
// Define contract (interface)
interface UserAPI {
    async function getUser(id: UserId): UserDTO
}

// Consumer uses the contract
class OrderService {
    private userAPI: UserAPI
    
    async function createOrder(userId: UserId): Order {
        // Synchronous call to another module
        user = await this.userAPI.getUser(userId)
        // Use user data...
    }
}
```

#### 2. Asynchronous Communication (Events)

**Use When:**
- Loose coupling desired
- Eventual consistency acceptable
- One-to-many communication
- Cross-module workflows

**Implementation:**

```pseudocode
// Define domain event
class UserCreatedEvent {
    userId: UserId
    email: String
    createdAt: DateTime
}

// Publisher (User module)
class UserModule {
    async function createUser(command: CreateUserCommand): UserId {
        user = User.create(command)
        await repository.save(user)
        
        // Publish event
        event = new UserCreatedEvent(
            userId: user.id,
            email: user.email,
            createdAt: user.createdAt
        )
        await eventBus.publish(event)
        
        return user.id
    }
}

// Subscriber (Order module)
class OrderModule {
    async function handleUserCreated(event: UserCreatedEvent) {
        // React to user creation
        log("User created: " + event.userId)
        // Possibly initialize user's shopping cart
    }
}
```

#### 3. Shared Kernel (Minimal!)

**Use When:**
- Truly shared concepts across all modules
- Core domain primitives

**Keep It Minimal:**

```pseudocode
// shared/
module Shared {
    // Common types
    class Result<T> { }
    class Error { }
    
    // Common interfaces
    interface DomainEvent { }
    interface EventBus { }
    
    // ⚠️ WARNING: Don't turn shared kernel into a dumping ground!
    // Only include what's TRULY shared across ALL modules
}
```

### Communication Guidelines

| Pattern | Use When | Pros | Cons |
|---------|----------|------|------|
| **Synchronous (Interface)** | Need immediate response, strong consistency | Simple, explicit dependencies | Tighter coupling |
| **Asynchronous (Events)** | Loose coupling, eventual consistency OK | Very loose coupling, scalable | Harder to debug, eventual consistency |
| **Shared Kernel** | Core primitives needed everywhere | Reduces duplication | Can become bloated if overused |

---

## Benefits and Trade-offs

### Benefits of Modular Monolith

#### 1. Simplicity
- **Single deployment**: No distributed system complexity
- **One codebase**: Easier to navigate and understand
- **Simplified testing**: Integration tests run in-process
- **No network calls**: Communication via function calls (faster)

#### 2. Development Velocity
- **Fast iteration**: Changes compile and run immediately
- **Easy refactoring**: Type system helps with breaking changes
- **Shared tooling**: Single build system, single test runner
- **IDE support**: Better code navigation and autocomplete

#### 3. Team Scalability
- **Clear ownership**: Teams own specific modules
- **Parallel development**: Teams work independently on their modules
- **Reduced coordination**: Well-defined APIs reduce inter-team dependencies

#### 4. Operational Simplicity
- **Single deployment**: No orchestration needed
- **Single database**: No distributed transactions
- **Simple monitoring**: One application to monitor
- **Lower infrastructure cost**: One server instead of many

#### 5. Evolvability
- **Path to microservices**: Modules can be extracted later if needed
- **Gradual refactoring**: Improve module boundaries over time
- **Technology adoption**: Upgrade dependencies incrementally

### Trade-offs and Limitations

#### 1. Scalability Constraints
- **Cannot scale modules independently**: Must scale entire application
- **Resource contention**: Heavy modules impact light modules
- **Mitigation**: Profile and optimize hot paths; consider extraction if needed

#### 2. Deployment Coupling
- **All modules deploy together**: No independent module releases
- **Mitigation**: Use feature flags for gradual rollouts

#### 3. Technology Homogeneity
- **Single language/runtime**: Cannot use polyglot architecture
- **Mitigation**: Choose a versatile language and runtime

#### 4. Database Coupling
- **Shared database instance**: Modules share same DB (though different schemas)
- **Mitigation**: Enforce data isolation via module boundaries

### When to Use Modular Monolith

#### ✅ Ideal For:

- **Startups and MVPs**: Fast development, simple deployment
- **Small-to-medium teams**: < 50 engineers
- **Domain uncertainty**: Still discovering bounded contexts
- **Strict consistency needs**: ACID transactions required
- **Cost-sensitive**: Lower infrastructure costs
- **Moderate scale**: Can handle millions of requests/day with proper optimization

#### ❌ Consider Microservices When:

- **Independent scalability required**: Some modules need 100x more resources
- **Polyglot needs**: Different modules need different languages/runtimes
- **Very large teams**: > 100 engineers, need strong isolation
- **Extreme scale**: Billions of requests/day
- **Regulatory isolation**: Compliance requires physical separation

---

## Implementation Patterns

### Language-Agnostic Patterns

#### Pattern 1: Module Facade

```pseudocode
// Public module interface
interface OrderAPI {
    function createOrder(command): OrderId
    function getOrder(id): OrderDTO
    function listOrders(query): List<OrderDTO>
}

// Module facade (implements public API)
class OrderModule implements OrderAPI {
    // Internal dependencies (hidden)
    private orderRepository
    private orderValidator
    private eventBus
    
    // Constructor injection
    constructor(repository, validator, eventBus) {
        this.orderRepository = repository
        this.orderValidator = validator
        this.eventBus = eventBus
    }
    
    // Public methods
    function createOrder(command): OrderId {
        // Delegate to internal components
        order = Order.create(command)
        this.orderValidator.validate(order)
        this.orderRepository.save(order)
        this.eventBus.publish(new OrderCreatedEvent(order))
        return order.id
    }
}
```

#### Pattern 2: Data Transfer Objects (DTOs)

```pseudocode
// Internal domain entity (private)
class Order {
    private id
    private customerId
    private items
    private totalAmount
    private status
    
    // Rich domain logic
    function addItem(item) { }
    function cancel() { }
    function calculateTotal() { }
}

// Public DTO (exposed via API)
class OrderDTO {
    public id
    public customerId
    public items
    public totalAmount
    public status
    
    // Only data, no behavior
    static function fromDomain(order: Order): OrderDTO {
        return new OrderDTO(
            id: order.id,
            customerId: order.customerId,
            items: order.items,
            totalAmount: order.totalAmount,
            status: order.status
        )
    }
}
```

#### Pattern 3: Dependency Injection (Composition Root)

```pseudocode
// Application composition root
class Application {
    private userModule
    private orderModule
    private paymentModule
    
    function initialize(config) {
        // Create shared infrastructure
        eventBus = new InMemoryEventBus()
        database = new Database(config.databaseUrl)
        
        // Initialize modules with dependencies
        this.userModule = new UserModule(
            repository: new UserRepository(database),
            eventBus: eventBus
        )
        
        this.orderModule = new OrderModule(
            repository: new OrderRepository(database),
            eventBus: eventBus,
            userAPI: this.userModule  // Inject as interface
        )
        
        this.paymentModule = new PaymentModule(
            repository: new PaymentRepository(database),
            eventBus: eventBus,
            orderAPI: this.orderModule
        )
    }
    
    // Expose module APIs
    function getUserAPI(): UserAPI {
        return this.userModule
    }
    
    function getOrderAPI(): OrderAPI {
        return this.orderModule
    }
}
```

#### Pattern 4: Event-Driven Communication

```pseudocode
// Event bus interface
interface EventBus {
    function publish(event: DomainEvent)
    function subscribe(eventType, handler: EventHandler)
}

// Domain event
class OrderCreatedEvent implements DomainEvent {
    orderId
    customerId
    totalAmount
    timestamp
}

// Publisher (Order module)
class OrderModule {
    function createOrder(command) {
        order = Order.create(command)
        repository.save(order)
        
        // Publish event
        event = new OrderCreatedEvent(
            orderId: order.id,
            customerId: order.customerId,
            totalAmount: order.totalAmount,
            timestamp: now()
        )
        eventBus.publish(event)
    }
}

// Subscriber (Payment module)
class PaymentModule {
    function initialize() {
        // Subscribe to events
        eventBus.subscribe(OrderCreatedEvent, this.handleOrderCreated)
    }
    
    function handleOrderCreated(event: OrderCreatedEvent) {
        // React to order creation
        log("Processing payment for order: " + event.orderId)
    }
}
```

---

## Best Practices

### 1. Start with Good Boundaries

**Do:**
- Use Domain-Driven Design to identify bounded contexts
- Map each bounded context to a module
- Validate boundaries with event storming workshops

**Don't:**
- Organize by technical layers (UI, business, data)
- Create modules for "utilities" or "helpers"
- Split modules too granularly (micro-modules)

### 2. Minimize the Shared Kernel

**Do:**
- Only include truly universal concepts (Result, Error, Event interface)
- Keep shared kernel as small as possible
- Version shared types carefully

**Don't:**
- Put business logic in shared kernel
- Use it as a dumping ground for "common" code
- Let it grow uncontrolled

### 3. Use Events for Cross-Module Workflows

**Do:**
- Publish domain events for significant state changes
- Use events for module integration
- Make events immutable and versioned

**Don't:**
- Use events for synchronous request-response
- Publish too many fine-grained events
- Forget to version event schemas

### 4. Enforce Architectural Rules with Tests

```pseudocode
// Architecture tests
test "modules only depend on public APIs" {
    // Verify no module imports internals of another module
    violations = analyzeImports()
    assert(violations.isEmpty())
}

test "shared kernel has no business logic" {
    // Verify shared kernel only has primitives
    violations = analyzeSharedKernel()
    assert(violations.isEmpty())
}

test "modules have clear public APIs" {
    // Each module must expose an API
    for each module in modules {
        assert(module.hasPublicAPI())
    }
}
```

### 5. Keep Module APIs Stable

**Do:**
- Version your APIs (V1, V2)
- Use semantic versioning
- Deprecate before removing

**Don't:**
- Make breaking changes without versioning
- Expose internal types in APIs
- Change API contracts frequently

### 6. Data Isolation

**Do:**
- Each module has its own database schema
- Modules access their own data only
- Use events to share data across modules

**Don't:**
- Share database tables between modules
- Use foreign keys across module boundaries
- Allow direct SQL joins across modules

```sql
-- ✅ GOOD: Separate schemas per module
CREATE SCHEMA user_management;
CREATE TABLE user_management.users (...);

CREATE SCHEMA order_processing;
CREATE TABLE order_processing.orders (...);

-- ❌ BAD: Foreign key across modules
CREATE TABLE order_processing.orders (
    user_id UUID REFERENCES user_management.users(id) -- ❌ Couples modules in DB!
);
```

### 7. Module Testing Strategy

```pseudocode
// Unit tests - test module internals
test "user creation validates email" {
    user = User.create(email: "invalid", name: "John")
    assert(user.hasError("invalid_email"))
}

// Integration tests - test module API
test "create user returns valid id" {
    userModule = UserModule.createForTesting()
    userId = await userModule.createUser(command)
    assert(userId.isValid())
}

// End-to-end tests - test cross-module workflows
test "create user and order workflow" {
    app = Application.createForTesting()
    
    // Test workflow across modules
    userId = await app.getUserAPI().createUser(userCommand)
    orderId = await app.getOrderAPI().createOrder(orderCommand)
    
    assert(order.customerId == userId)
}
```

### 8. Module Lifecycle Management

```pseudocode
class Module {
    private databaseConnection
    private eventSubscriptions
    
    async function initialize() {
        // Subscribe to events
        // Start background workers
    }
    
    async function shutdown() {
        // Clean up resources
        // Unsubscribe from events
        // Flush pending work
    }
}
```

---

## Anti-Patterns to Avoid

### 1. ❌ Shared Database Tables

**Problem:**
```sql
-- ❌ BAD: All modules read/write same table
CREATE TABLE users (...);
-- User module queries users
-- Order module queries users
-- Payment module queries users
```

**Solution:**
```sql
-- ✅ GOOD: Each module has its own user representation
CREATE SCHEMA user_management;
CREATE TABLE user_management.users (...);

CREATE SCHEMA order_processing;
CREATE TABLE order_processing.customers (...); -- Order's view of users
```

### 2. ❌ Circular Dependencies

**Problem:**
```pseudocode
// ❌ BAD: Circular dependency
module UserManagement {
    import OrderAPI from order_processing  // User depends on Order
}

module OrderProcessing {
    import UserAPI from user_management    // Order depends on User
}
```

**Solution:**
```pseudocode
// ✅ GOOD: Use events to break cycle
module UserManagement {
    // Publishes UserCreatedEvent
    eventBus.publish(new UserCreatedEvent(...))
}

module OrderProcessing {
    // Subscribes to UserCreatedEvent
    function handleUserCreated(event: UserCreatedEvent) { }
}
```

### 3. ❌ Anemic Modules (Just Data Transfer)

**Problem:**
```pseudocode
// ❌ BAD: Module is just CRUD operations
interface UserAPI {
    function create(data: UserData): void
    function read(id: UserId): UserData
    function update(id: UserId, data: UserData): void
    function delete(id: UserId): void
}
// No business logic!
```

**Solution:**
```pseudocode
// ✅ GOOD: Module exposes business operations
interface UserAPI {
    function registerUser(command: RegisterUserCommand): UserId
    function activateUser(id: UserId): void
    function changeEmail(id: UserId, newEmail: Email): void
}
// Business-meaningful operations!
```

### 4. ❌ God Module (Too Much Responsibility)

**Problem:**
```pseudocode
// ❌ BAD: One module does everything
module Application {
    // Handles users, orders, payments, inventory, notifications...
    // 10,000+ lines of code
}
```

**Solution:**
```pseudocode
// ✅ GOOD: Split by bounded context
module UserManagement { }
module OrderProcessing { }
module Payment { }
module Inventory { }
```

### 5. ❌ Leaky Abstractions

**Problem:**
```pseudocode
// ❌ BAD: Internal types leak into API
interface UserAPI {
    function saveUser(user: User): void
    //                       ^^^^ Internal domain type exposed!
}
```

**Solution:**
```pseudocode
// ✅ GOOD: API uses DTOs
interface UserAPI {
    function createUser(command: CreateUserCommand): UserDTO
    //                           ^^^^^^^^^^^^^^^^^    ^^^^^^^
    //                           API types only!
}
```

### 6. ❌ Bypassing Module APIs

**Problem:**
```pseudocode
// ❌ BAD: Directly accessing database from another module
module OrderProcessing {
    function createOrder() {
        // Directly queries user database!
        user = database.query("SELECT * FROM user_management.users WHERE id = ?")
    }
}
```

**Solution:**
```pseudocode
// ✅ GOOD: Use module API
module OrderProcessing {
    function createOrder() {
        // Uses User module's API
        user = userAPI.getUser(userId)
    }
}
```

---

## Migration Path

### From Layered Monolith to Modular Monolith

#### Phase 1: Identify Modules (Bounded Contexts)

1. **Conduct Event Storming**: Map business processes and events
2. **Identify Aggregates**: Group related entities
3. **Define Bounded Contexts**: Draw boundaries around cohesive domains
4. **Map to Modules**: Each bounded context becomes a module

#### Phase 2: Refactor One Module at a Time

**Step 1: Create Module Structure**
```
src/
└── modules/
    └── user_management/  # New module
        ├── api/
        ├── domain/
        ├── application/
        └── infrastructure/
```

**Step 2: Move Domain Logic**
- Extract entities, value objects, domain services
- Move to `user_management/domain/`

**Step 3: Define Public API**
- Create interface in `user_management/api/`
- Expose only what other modules need

**Step 4: Migrate Consumers**
- Update other parts of code to use new API
- Remove direct dependencies on old code

**Step 5: Isolate Data**
- Move database tables to module schema
- Update migrations

#### Phase 3: Enforce Boundaries

1. **Add Architecture Tests**: Verify no boundary violations
2. **Code Reviews**: Check for anti-patterns
3. **Documentation**: Document module APIs and contracts

### From Modular Monolith to Microservices

If you eventually need microservices, the modular monolith makes this transition smooth:

#### Step 1: Choose Module to Extract

**Criteria:**
- High resource usage (CPU/memory)
- Different scaling needs
- Independent deployment needed
- Team wants autonomy

#### Step 2: Define Service Interface

```pseudocode
// Before (in-process interface)
interface UserAPI {
    function getUser(id: UserId): UserDTO
}

// After (network service)
service UserService {
    endpoint GET /users/{id} returns UserDTO
}
```

#### Step 3: Extract Module

1. Create new service project
2. Copy module code
3. Implement network API (REST/gRPC)
4. Deploy as separate service

#### Step 4: Update Monolith

```pseudocode
// Before: In-process call
user = userModule.getUser(id)

// After: Remote call
user = await httpClient.get("/users/" + id)
// Same API shape! Only implementation changed.
```

#### Step 5: Migrate Data

1. Replicate data to new database
2. Switch reads to new service
3. Switch writes to new service
4. Decommission old tables

**Key Insight**: Because modules already have clear boundaries, extracting to microservices is a deployment change, not an architecture change!

---

## Conclusion

### Key Takeaways

1. **Modular Monolith = Simplicity + Modularity**
   - Single deployment, multiple logical modules
   - Best of both monolith and microservices worlds

2. **Design for Modules, Not Layers**
   - Vertical slices (business capabilities)
   - Not horizontal layers (technical concerns)

3. **Enforce Boundaries Rigorously**
   - Public APIs only
   - No shared database tables
   - Use events for cross-module communication

4. **Start Modular, Extract When Needed**
   - Begin with modular monolith
   - Extract to microservices only when necessary
   - Modular boundaries make extraction easy

### Decision Framework

```
Start Here → Modular Monolith

Scale Requirements?
├─ No → Stay with Modular Monolith ✅
└─ Yes → 
    Independent Scaling Needed?
    ├─ No → Stay with Modular Monolith ✅
    └─ Yes → 
        Polyglot Requirements?
        ├─ No → Horizontal scaling of monolith ✅
        └─ Yes → Consider Microservices

Team Size?
├─ < 50 engineers → Modular Monolith ✅
└─ > 100 engineers → Consider Microservices

Domain Knowledge?
├─ Uncertain → Modular Monolith ✅
└─ Crystal clear → Can start with Microservices
```

### Further Reading

- [Modular Monolith: A Primer - Kamil Grzybek](https://www.kamilgrzybek.com/blog/posts/modular-monolith-primer)
- [Modular Monolith with DDD - GitHub](https://github.com/kgrzybek/modular-monolith-with-ddd)
- [Google: Towards Modern Development of Cloud Applications](https://dl.acm.org/doi/pdf/10.1145/3593856.3595909)
- [What Is a Modular Monolith? - Milan Jovanović](https://www.milanjovanovic.tech/blog/what-is-a-modular-monolith)

---

**Document Version**: 2.0  
**Last Updated**: 2026-01-25  
**Status**: Active
