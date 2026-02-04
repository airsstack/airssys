# Dependency Management: DI & DIP

> Generic guidelines for Dependency Injection and Dependency Inversion Principle applicable across programming languages

---

## Table of Contents

1. [Overview](#overview)
2. [Dependency Inversion Principle (DIP)](#dependency-inversion-principle-dip)
3. [Dependency Injection (DI)](#dependency-injection-di)
4. [Patterns & Techniques](#patterns--techniques)
5. [Anti-Patterns](#anti-patterns)
6. [Testing Strategies](#testing-strategies)
7. [Language-Specific Considerations](#language-specific-considerations)

---

## Overview

**Purpose:** Decouple modules to improve testability, maintainability, and flexibility.

**Core Concepts:**
- **Dependency Inversion Principle (DIP):** High-level modules should not depend on low-level modules. Both should depend on abstractions.
- **Dependency Injection (DI):** A technique where dependencies are provided (injected) rather than created internally.

**Benefits:**
- Loose coupling between components
- Easy mocking/stubbing for tests
- Swap implementations without changing client code
- Clear dependency graph

---

## Dependency Inversion Principle (DIP)

### Definition

**SOLID's "D":** 
1. High-level modules should not depend on low-level modules. Both should depend on abstractions.
2. Abstractions should not depend on details. Details should depend on abstractions.

### Without DIP (Tightly Coupled)

```pseudocode
// Low-level module
class MySQLDatabase {
    function save(data) {
        // MySQL-specific implementation
    }
}

// High-level module depends on concrete low-level module
class UserService {
    private database: MySQLDatabase
    
    constructor() {
        this.database = new MySQLDatabase()  // ❌ Tight coupling
    }
    
    function createUser(user) {
        this.database.save(user)
    }
}
```

**Problems:**
- `UserService` cannot use PostgreSQL without modification
- Cannot test `UserService` without real MySQL database
- Violates Open/Closed Principle

### With DIP (Loosely Coupled)

```pseudocode
// Abstraction (interface/trait/protocol)
interface Database {
    function save(data)
    function find(id)
}

// Low-level modules implement abstraction
class MySQLDatabase implements Database {
    function save(data) {
        // MySQL implementation
    }
    
    function find(id) {
        // MySQL implementation
    }
}

class PostgreSQLDatabase implements Database {
    function save(data) {
        // PostgreSQL implementation
    }
    
    function find(id) {
        // PostgreSQL implementation
    }
}

// High-level module depends on abstraction
class UserService {
    private database: Database  // ✅ Depends on abstraction
    
    constructor(database: Database) {
        this.database = database
    }
    
    function createUser(user) {
        this.database.save(user)
    }
}
```

**Benefits:**
- `UserService` works with any `Database` implementation
- Easy to test with mock database
- Can swap databases at runtime

---

## Dependency Injection (DI)

### Definition

**Technique:** Providing dependencies to a component from the outside rather than creating them internally.

### Three Types of DI

#### 1. Constructor Injection (Recommended)

```pseudocode
class EmailService {
    private mailer: Mailer
    private logger: Logger
    
    // Dependencies injected via constructor
    constructor(mailer: Mailer, logger: Logger) {
        this.mailer = mailer
        this.logger = logger
    }
    
    function sendEmail(to, subject, body) {
        this.logger.info("Sending email to: " + to)
        this.mailer.send(to, subject, body)
    }
}

// Usage
mailer = new SMTPMailer()
logger = new FileLogger()
emailService = new EmailService(mailer, logger)
```

**Pros:**
- Dependencies are explicit and immutable
- Impossible to create object in invalid state
- Easy to test

**Cons:**
- Constructor can become large with many dependencies

#### 2. Setter Injection

```pseudocode
class EmailService {
    private mailer: Mailer
    private logger: Logger
    
    function setMailer(mailer: Mailer) {
        this.mailer = mailer
    }
    
    function setLogger(logger: Logger) {
        this.logger = logger
    }
    
    function sendEmail(to, subject, body) {
        if this.logger != null {
            this.logger.info("Sending email to: " + to)
        }
        this.mailer.send(to, subject, body)
    }
}

// Usage
emailService = new EmailService()
emailService.setMailer(new SMTPMailer())
emailService.setLogger(new FileLogger())
```

**Pros:**
- Flexible - can change dependencies after construction
- Optional dependencies

**Cons:**
- Object can be in invalid state
- Dependencies are mutable
- Less explicit

#### 3. Interface/Method Injection

```pseudocode
class EmailService {
    // Dependency injected per method call
    function sendEmail(to, subject, body, mailer: Mailer, logger: Logger) {
        logger.info("Sending email to: " + to)
        mailer.send(to, subject, body)
    }
}

// Usage
emailService = new EmailService()
emailService.sendEmail("user@example.com", "Hello", "...", 
                       new SMTPMailer(), new FileLogger())
```

**Pros:**
- Most flexible
- No state management

**Cons:**
- Verbose method signatures
- Repetitive if same dependency used across methods

### Recommendation Hierarchy

1. **Constructor Injection** - Default choice for required dependencies
2. **Setter Injection** - Optional dependencies or framework constraints
3. **Method Injection** - Context-specific dependencies or one-off needs

---

## Patterns & Techniques

### 1. Manual DI (Poor Man's DI)

**Concept:** Wire dependencies manually at composition root.

```pseudocode
// Composition root (main function)
function main() {
    // Create leaf dependencies first
    config = loadConfig()
    logger = new FileLogger(config.logPath)
    
    // Create intermediate dependencies
    database = new PostgreSQLDatabase(config.dbUrl, logger)
    cache = new RedisCache(config.redisUrl, logger)
    
    // Create high-level services
    userRepository = new UserRepository(database, cache)
    emailService = new EmailService(new SMTPMailer(), logger)
    
    // Create application service
    userService = new UserService(userRepository, emailService, logger)
    
    // Create API/UI layer
    userController = new UserController(userService)
    
    // Start application
    server = new Server(userController)
    server.start()
}
```

**Pros:**
- No framework dependency
- Explicit and traceable
- Simple to understand

**Cons:**
- Manual wiring for large projects
- No lifecycle management

### 2. Service Locator Pattern (Anti-Pattern)

**Concept:** Global registry for dependencies.

```pseudocode
// Service Locator (❌ Anti-Pattern)
class ServiceLocator {
    private static services: Map<String, Object> = {}
    
    static function register(name: String, service: Object) {
        services[name] = service
    }
    
    static function get(name: String): Object {
        return services[name]
    }
}

// Usage
class UserService {
    function createUser(user) {
        // Hidden dependency - not visible in constructor
        database = ServiceLocator.get("database")  // ❌ Bad
        database.save(user)
    }
}
```

**Problems:**
- Hidden dependencies (not in constructor)
- Tight coupling to ServiceLocator
- Hard to test
- Runtime errors if dependency not registered

**Avoid Service Locator.** Use DI instead.

### 3. DI Container/Framework

**Concept:** Automate dependency wiring and lifecycle management.

```pseudocode
// Container registration
container = new DIContainer()

// Register abstractions to concrete types
container.register(Interface: Database, 
                   Implementation: PostgreSQLDatabase, 
                   Lifecycle: Singleton)

container.register(Interface: Logger, 
                   Implementation: FileLogger, 
                   Lifecycle: Transient)

container.register(Interface: UserService, 
                   Implementation: UserService, 
                   Lifecycle: Scoped)

// Resolve dependencies automatically
userService = container.resolve(UserService)
// Container automatically injects Database and Logger

// Usage
userService.createUser(user)
```

**Lifecycles:**
- **Singleton:** One instance for entire application
- **Transient:** New instance per request
- **Scoped:** One instance per scope (e.g., HTTP request)

**Pros:**
- Automatic dependency resolution
- Lifecycle management
- Reduces boilerplate

**Cons:**
- Framework dependency
- Magic behavior (less explicit)
- Learning curve

### 4. Factory Pattern with DI

**Concept:** Use factories to create complex objects with dependencies.

```pseudocode
interface ReportGenerator {
    function generate(): Report
}

class PDFReportGenerator implements ReportGenerator {
    private database: Database
    private logger: Logger
    
    constructor(database: Database, logger: Logger) {
        this.database = database
        this.logger = logger
    }
    
    function generate(): Report {
        // PDF generation logic
    }
}

class ExcelReportGenerator implements ReportGenerator {
    private database: Database
    private logger: Logger
    
    constructor(database: Database, logger: Logger) {
        this.database = database
        this.logger = logger
    }
    
    function generate(): Report {
        // Excel generation logic
    }
}

// Factory with injected dependencies
class ReportGeneratorFactory {
    private database: Database
    private logger: Logger
    
    constructor(database: Database, logger: Logger) {
        this.database = database
        this.logger = logger
    }
    
    function create(type: String): ReportGenerator {
        if type == "PDF" {
            return new PDFReportGenerator(this.database, this.logger)
        } else if type == "Excel" {
            return new ExcelReportGenerator(this.database, this.logger)
        }
        throw new Error("Unknown report type")
    }
}

// Usage
factory = new ReportGeneratorFactory(database, logger)
generator = factory.create("PDF")
report = generator.generate()
```

### 5. Interface Segregation for Dependencies

**Concept:** Depend on minimal interfaces, not fat interfaces.

```pseudocode
// ❌ Fat interface
interface Database {
    function save(data)
    function find(id)
    function delete(id)
    function transaction(callback)
    function migrate()
    function backup()
}

// UserService only needs save/find
class UserService {
    private database: Database  // ❌ Depends on more than needed
    
    function createUser(user) {
        this.database.save(user)
    }
}

// ✅ Segregated interfaces
interface Reader {
    function find(id)
}

interface Writer {
    function save(data)
}

interface TransactionManager {
    function transaction(callback)
}

// UserService depends on what it needs
class UserService {
    private reader: Reader
    private writer: Writer
    
    constructor(reader: Reader, writer: Writer) {
        this.reader = reader
        this.writer = writer
    }
    
    function createUser(user) {
        this.writer.save(user)
    }
    
    function getUser(id) {
        return this.reader.find(id)
    }
}
```

**Benefits:**
- Smaller interfaces are easier to mock
- Clear about what component actually uses
- Easier to maintain

---

## Anti-Patterns

### 1. New Keyword Abuse

```pseudocode
// ❌ Creating dependencies internally
class UserService {
    private database: Database
    
    constructor() {
        this.database = new MySQLDatabase()  // ❌ Tight coupling
    }
}
```

**Fix:** Inject dependency via constructor.

### 2. Static Dependencies

```pseudocode
// ❌ Static method calls
class UserService {
    function createUser(user) {
        DatabaseConnection.getInstance().save(user)  // ❌ Hidden dependency
    }
}
```

**Fix:** Inject instance via constructor.

### 3. God Object/Container Passed Around

```pseudocode
// ❌ Passing entire container
class UserService {
    private container: DIContainer
    
    constructor(container: DIContainer) {
        this.container = container  // ❌ Too much access
    }
    
    function createUser(user) {
        database = this.container.get("database")
        database.save(user)
    }
}
```

**Fix:** Inject specific dependencies, not container.

### 4. Temporal Coupling

```pseudocode
// ❌ Order matters - temporal coupling
class EmailService {
    private mailer: Mailer
    
    function setMailer(mailer: Mailer) {
        this.mailer = mailer
    }
    
    function sendEmail(to, subject, body) {
        this.mailer.send(to, subject, body)  // ❌ Fails if setMailer not called
    }
}
```

**Fix:** Use constructor injection for required dependencies.

### 5. Over-Abstraction

```pseudocode
// ❌ Unnecessary abstraction
interface StringConcatenator {
    function concat(a: String, b: String): String
}

class DefaultStringConcatenator implements StringConcatenator {
    function concat(a: String, b: String): String {
        return a + b  // ❌ No value added
    }
}
```

**Fix:** Only abstract when there are multiple implementations or test seams needed.

---

## Testing Strategies

### 1. Mock Objects

```pseudocode
// Production code
interface Database {
    function save(data)
    function find(id)
}

class UserService {
    private database: Database
    
    constructor(database: Database) {
        this.database = database
    }
    
    function createUser(user) {
        this.database.save(user)
        return user.id
    }
}

// Test code
class MockDatabase implements Database {
    savedData: List = []
    
    function save(data) {
        this.savedData.append(data)
    }
    
    function find(id) {
        // Mock implementation
    }
}

// Test
test "createUser saves to database" {
    mockDb = new MockDatabase()
    service = new UserService(mockDb)
    
    user = { id: 1, name: "Alice" }
    service.createUser(user)
    
    assert mockDb.savedData.length == 1
    assert mockDb.savedData[0] == user
}
```

### 2. Spy Objects

```pseudocode
class SpyLogger implements Logger {
    loggedMessages: List = []
    
    function info(message: String) {
        this.loggedMessages.append(message)
    }
    
    function wasCalled(): Boolean {
        return this.loggedMessages.length > 0
    }
    
    function wasCalledWith(message: String): Boolean {
        return this.loggedMessages.contains(message)
    }
}

// Test
test "createUser logs activity" {
    spyLogger = new SpyLogger()
    service = new UserService(database, spyLogger)
    
    service.createUser(user)
    
    assert spyLogger.wasCalled()
    assert spyLogger.wasCalledWith("Creating user: Alice")
}
```

### 3. Stub Objects

```pseudocode
class StubDatabase implements Database {
    function save(data) {
        // Do nothing
    }
    
    function find(id) {
        // Return predefined data
        return { id: 1, name: "Alice" }
    }
}

// Test
test "getUser returns user from database" {
    stubDb = new StubDatabase()
    service = new UserService(stubDb)
    
    user = service.getUser(1)
    
    assert user.name == "Alice"
}
```

### 4. Test Doubles with DI Containers

```pseudocode
// Test-specific DI configuration
test "user service integration test" {
    container = new DIContainer()
    
    // Override production dependencies with test doubles
    container.register(Database, InMemoryDatabase)  // ✅ Test double
    container.register(Logger, SpyLogger)           // ✅ Test double
    
    // Resolve service with test dependencies
    service = container.resolve(UserService)
    
    service.createUser(user)
    
    // Assertions...
}
```

---

## Language-Specific Considerations

### Statically-Typed Languages (Rust, Java, C#, TypeScript)

**Abstractions:**
- **Interfaces** (Java, C#, TypeScript)
- **Traits** (Rust)
- **Abstract Classes** (Java, C#)

**Example (Generic):**
```pseudocode
// Define trait/interface
trait Database {
    function save(data: Data) -> Result
}

// Implement for concrete type
class PostgreSQL implements Database {
    function save(data: Data) -> Result {
        // Implementation
    }
}

// Generic struct depends on trait bound
class UserService<D: Database> {
    private database: D
    
    constructor(database: D) {
        this.database = database
    }
}
```

**Benefits:**
- Compile-time safety
- IDE autocomplete
- Explicit contracts

### Dynamically-Typed Languages (Python, JavaScript, Ruby)

**Abstractions:**
- **Duck Typing** (implicit interfaces)
- **Abstract Base Classes** (Python)
- **Protocols** (Python 3.8+)

**Example (Generic):**
```pseudocode
// No explicit interface - duck typing
class PostgreSQL {
    function save(data) {
        // Implementation
    }
}

class UserService {
    constructor(database) {
        this.database = database  // Any object with save() method
    }
    
    function createUser(user) {
        this.database.save(user)
    }
}
```

**Considerations:**
- Runtime errors if dependency missing methods
- Use type hints/docstrings to document expected interface
- Unit tests critical for catching contract violations

### Functional Languages (Haskell, Elixir, Clojure)

**Abstractions:**
- **Typeclasses** (Haskell)
- **Behaviours** (Elixir)
- **Protocols** (Clojure)

**Example (Generic):**
```pseudocode
// Typeclass/protocol
protocol Database {
    function save(db, data)
}

// Implementation for concrete type
implement Database for PostgreSQL {
    function save(db, data) {
        // Implementation
    }
}

// Function depends on protocol
function createUser(database: Database, user) {
    Database.save(database, user)
}
```

**DI via:**
- **Higher-order functions** (pass dependencies as parameters)
- **Reader monad** (Haskell)
- **Application environment** (Elixir)

---

## Summary

### Key Principles

1. **Depend on abstractions, not concretions** (DIP)
2. **Inject dependencies, don't create them** (DI)
3. **Prefer constructor injection** for required dependencies
4. **Keep interfaces small** (Interface Segregation)
5. **Avoid Service Locator** (anti-pattern)

### Decision Tree

```
Do I need to swap implementations?
├─ No → Direct instantiation OK (no abstraction needed)
└─ Yes → Create abstraction
    │
    ├─ Required dependency?
    │  └─ Yes → Constructor injection
    │
    ├─ Optional dependency?
    │  └─ Yes → Setter injection or default value
    │
    └─ Context-specific dependency?
       └─ Yes → Method injection
```

### Testing Mindset

**If it's hard to test, it's poorly designed.**

Good DI makes testing trivial:
- Mock external dependencies
- Test in isolation
- No setup/teardown overhead

---

## References

- **SOLID Principles** by Robert C. Martin
- **Dependency Injection Principles, Practices, and Patterns** by Steven van Deursen & Mark Seemann
- **Clean Architecture** by Robert C. Martin
- **Design Patterns** by Gang of Four

---

*Last Updated: 2026-01-25*
