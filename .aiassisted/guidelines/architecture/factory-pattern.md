# Factory Pattern - A Generic Design Pattern Guideline

## Table of Contents

1. [Introduction](#introduction)
2. [What is the Factory Pattern?](#what-is-the-factory-pattern)
3. [When to Use Factory Pattern](#when-to-use-factory-pattern)
4. [When NOT to Use Factory Pattern](#when-not-to-use-factory-pattern)
5. [Core Components](#core-components)
6. [Factory Pattern Variants](#factory-pattern-variants)
7. [Implementation Patterns](#implementation-patterns)
8. [Real-World Applications](#real-world-applications)
9. [Advantages](#advantages)
10. [Trade-offs and Considerations](#trade-offs-and-considerations)
11. [Best Practices](#best-practices)
12. [Anti-Patterns](#anti-patterns)
13. [Comparison with Related Patterns](#comparison-with-related-patterns)
14. [Decision Framework](#decision-framework)

---

## Introduction

The **Factory Pattern** is a creational design pattern that provides an interface for creating objects without specifying their exact classes. It encapsulates object creation logic, promoting loose coupling and adherence to the Open/Closed Principle.

**Key Concept:** Define an interface for creating objects, but let subclasses or implementing classes decide which concrete class to instantiate.

### Pattern Family

The Factory Pattern is part of the **Creational Patterns** family, which deals with object creation mechanisms. It includes several variants:

- **Simple Factory** (not a GoF pattern, but widely used)
- **Factory Method** (GoF pattern)
- **Abstract Factory** (GoF pattern)

This guideline covers all three variants with a focus on language-agnostic principles.

---

## What is the Factory Pattern?

### Definition

> "Define an interface for creating an object, but let subclasses decide which class to instantiate. Factory Method lets a class defer instantiation to subclasses."
> — Gang of Four, *Design Patterns*

### Core Idea

Instead of directly calling constructors to create objects:

```pseudocode
// Direct instantiation (tight coupling)
product = new ConcreteProduct()
```

Use a factory method or factory class:

```pseudocode
// Factory-based creation (loose coupling)
product = factory.createProduct()
```

### Problem It Solves

**Without Factory Pattern:**
- Client code is tightly coupled to concrete classes
- Adding new product types requires modifying client code
- Violates Open/Closed Principle
- Complex initialization logic scattered across codebase

**With Factory Pattern:**
- Client code depends on abstractions, not concrete classes
- New product types can be added without modifying client code
- Centralized object creation logic
- Easy to test with mock factories

---

## When to Use Factory Pattern

### Use Case 1: Unknown Types at Compile Time

When the exact types and dependencies of objects are not known beforehand:

```pseudocode
// Configuration-driven object creation
config = readConfigFile()
if config.databaseType == "MySQL":
    connection = MySQLConnection()
else if config.databaseType == "PostgreSQL":
    connection = PostgreSQLConnection()
```

**Factory Solution:**
```pseudocode
connection = ConnectionFactory.create(config.databaseType)
```

### Use Case 2: Complex Object Creation

When object creation involves complex logic that shouldn't be in client code:

```pseudocode
// Complex setup
vehicle = new Car()
vehicle.setEngine(new V8Engine())
vehicle.setTransmission(new AutomaticTransmission())
vehicle.setWheels(4)
vehicle.configureSafetySystems()
```

**Factory Solution:**
```pseudocode
vehicle = VehicleFactory.createCar()
```

### Use Case 3: Family of Related Objects

When you need to create families of related objects:

```pseudocode
// UI components for different platforms
button = factory.createButton()
checkbox = factory.createCheckbox()
textField = factory.createTextField()
```

### Use Case 4: Framework Extension

When you want to provide extension points for libraries/frameworks:

```pseudocode
// Framework defines factory method
abstract class UIFramework:
    abstract method createButton(): Button
    
    method render():
        button = this.createButton()
        button.display()

// Users extend with custom implementations
class CustomUIFramework extends UIFramework:
    method createButton(): Button:
        return new RoundButton()
```

### Use Case 5: Resource Reuse

When you want to reuse expensive objects instead of recreating them:

```pseudocode
// Connection pooling
class ConnectionFactory:
    pool = []
    
    method getConnection():
        if pool.isEmpty():
            return new DatabaseConnection()
        else:
            return pool.remove()
```

---

## When NOT to Use Factory Pattern

### ❌ Simple Object Creation

Don't use factory for trivial object instantiation:

```pseudocode
// Overkill for simple objects
point = PointFactory.createPoint(x, y)

// Just use constructor
point = new Point(x, y)
```

### ❌ Single Product Type

If you only ever create one type of object:

```pseudocode
// Unnecessary abstraction
factory = new UserFactory()
user = factory.createUser()

// Direct instantiation is clearer
user = new User()
```

### ❌ Over-Engineering

Don't add factories "just in case" for future flexibility:

```pseudocode
// YAGNI violation - You Aren't Gonna Need It
class StringFactory:
    method createString(text):
        return new String(text)
```

---

## Core Components

### 1. Product Interface/Abstract Class

Defines the interface that all concrete products must implement:

```pseudocode
interface Transport:
    method deliver(destination: String)
    method getCost(): Decimal
```

### 2. Concrete Products

Implementations of the product interface:

```pseudocode
class Truck implements Transport:
    method deliver(destination: String):
        print("Delivering by land to " + destination)
    
    method getCost(): Decimal:
        return 50.00

class Ship implements Transport:
    method deliver(destination: String):
        print("Delivering by sea to " + destination)
    
    method getCost(): Decimal:
        return 200.00
```

### 3. Creator/Factory Interface

Declares the factory method:

```pseudocode
abstract class Logistics:
    abstract method createTransport(): Transport
    
    method planDelivery(destination: String):
        transport = this.createTransport()
        transport.deliver(destination)
```

### 4. Concrete Creators/Factories

Implement the factory method to create specific products:

```pseudocode
class RoadLogistics extends Logistics:
    method createTransport(): Transport:
        return new Truck()

class SeaLogistics extends Logistics:
    method createTransport(): Transport:
        return new Ship()
```

---

## Factory Pattern Variants

### Variant 1: Simple Factory (Static Factory)

**Structure:**
- Single factory class with a method that returns different product types
- Not a GoF pattern, but widely used
- Also known as "Static Factory Method"

**Implementation:**

```pseudocode
class VehicleFactory:
    method createVehicle(type: String): Vehicle:
        if type == "car":
            return new Car()
        else if type == "motorcycle":
            return new Motorcycle()
        else if type == "truck":
            return new Truck()
        else:
            throw new Error("Unknown vehicle type")

// Usage
factory = new VehicleFactory()
vehicle = factory.createVehicle("car")
```

**Characteristics:**
- ✅ Simple to implement
- ✅ Centralizes object creation
- ❌ Violates Open/Closed Principle (need to modify factory for new types)
- ❌ Not extensible through inheritance

### Variant 2: Factory Method Pattern

**Structure:**
- Abstract creator class with abstract factory method
- Concrete creators override factory method
- GoF creational pattern

**Implementation:**

```pseudocode
// Abstract creator
abstract class DocumentCreator:
    abstract method createDocument(): Document
    
    method openDocument():
        doc = this.createDocument()
        doc.open()
        return doc

// Concrete creators
class PDFCreator extends DocumentCreator:
    method createDocument(): Document:
        return new PDFDocument()

class WordCreator extends DocumentCreator:
    method createDocument(): Document:
        return new WordDocument()

// Usage
creator = new PDFCreator()
document = creator.openDocument()
```

**Characteristics:**
- ✅ Follows Open/Closed Principle
- ✅ Extensible through inheritance
- ✅ Subclasses control object creation
- ❌ Requires subclass for each product type

### Variant 3: Abstract Factory Pattern

**Structure:**
- Factory interface with multiple factory methods
- Creates families of related objects
- GoF creational pattern

**Implementation:**

```pseudocode
// Abstract factory
interface UIFactory:
    method createButton(): Button
    method createCheckbox(): Checkbox
    method createTextField(): TextField

// Concrete factories
class WindowsUIFactory implements UIFactory:
    method createButton(): Button:
        return new WindowsButton()
    
    method createCheckbox(): Checkbox:
        return new WindowsCheckbox()
    
    method createTextField(): TextField:
        return new WindowsTextField()

class MacUIFactory implements UIFactory:
    method createButton(): Button:
        return new MacButton()
    
    method createCheckbox(): Checkbox:
        return new MacCheckbox()
    
    method createTextField(): TextField:
        return new MacTextField()

// Usage
factory = new WindowsUIFactory()
button = factory.createButton()
checkbox = factory.createCheckbox()
```

**Characteristics:**
- ✅ Creates families of related objects
- ✅ Ensures product compatibility
- ✅ Isolates concrete classes
- ❌ More complex than Factory Method
- ❌ Adding new product types requires changing all factories

---

## Implementation Patterns

### Pattern 1: Parameterized Factory

Use parameters to determine which product to create:

```pseudocode
class PaymentFactory:
    method createPaymentProcessor(type: String): PaymentProcessor:
        switch type:
            case "credit_card":
                return new CreditCardProcessor()
            case "paypal":
                return new PayPalProcessor()
            case "bank_transfer":
                return new BankTransferProcessor()
            default:
                throw new Error("Unsupported payment type")

// Usage
factory = new PaymentFactory()
processor = factory.createPaymentProcessor("paypal")
```

### Pattern 2: Registry-Based Factory

Use a registry to map keys to product classes:

```pseudocode
class NotificationFactory:
    registry = {}
    
    static method register(type: String, creator: Function):
        registry[type] = creator
    
    method create(type: String, config: Object): Notification:
        if type not in registry:
            throw new Error("Unknown notification type")
        
        creator = registry[type]
        return creator(config)

// Registration
NotificationFactory.register("email", function(config):
    return new EmailNotification(config)
)

NotificationFactory.register("sms", function(config):
    return new SMSNotification(config)
)

// Usage
factory = new NotificationFactory()
notification = factory.create("email", {to: "user@example.com"})
```

### Pattern 3: Lazy Initialization Factory

Create products only when needed:

```pseudocode
class DatabaseConnectionFactory:
    connections = {}
    
    method getConnection(dbName: String): Connection:
        if dbName not in connections:
            connections[dbName] = this.createConnection(dbName)
        
        return connections[dbName]
    
    private method createConnection(dbName: String): Connection:
        return new DatabaseConnection(dbName)
```

### Pattern 4: Dependency Injection Factory

Inject dependencies into created objects:

```pseudocode
class ServiceFactory:
    logger: Logger
    config: Configuration
    
    constructor(logger: Logger, config: Configuration):
        this.logger = logger
        this.config = config
    
    method createUserService(): UserService:
        repository = new UserRepository(this.config)
        return new UserService(repository, this.logger)
    
    method createOrderService(): OrderService:
        repository = new OrderRepository(this.config)
        return new OrderService(repository, this.logger)
```

### Pattern 5: Builder-Enhanced Factory

Combine Factory with Builder for complex objects:

```pseudocode
class ReportFactory:
    method createSalesReport(format: String): Report:
        builder = new ReportBuilder()
        
        report = builder
            .setTitle("Sales Report")
            .setDataSource(new SalesDataSource())
            .setFormat(format)
            .addColumn("Date")
            .addColumn("Revenue")
            .addColumn("Units Sold")
            .build()
        
        return report
```

---

## Real-World Applications

### 1. Database Connections

```pseudocode
// Database connection factory
interface DatabaseConnection:
    method connect()
    method query(sql: String)
    method close()

class ConnectionFactory:
    method createConnection(type: String, config: Object): DatabaseConnection:
        switch type:
            case "mysql":
                return new MySQLConnection(config)
            case "postgresql":
                return new PostgreSQLConnection(config)
            case "mongodb":
                return new MongoDBConnection(config)

// Usage in application
factory = new ConnectionFactory()
connection = factory.createConnection("postgresql", dbConfig)
connection.connect()
```

### 2. Payment Processing

```pseudocode
interface PaymentProcessor:
    method processPayment(amount: Decimal, details: Object): Boolean
    method refund(transactionId: String): Boolean

class PaymentFactory:
    method createProcessor(gateway: String): PaymentProcessor:
        switch gateway:
            case "stripe":
                return new StripeProcessor()
            case "paypal":
                return new PayPalProcessor()
            case "square":
                return new SquareProcessor()

// E-commerce checkout
processor = PaymentFactory.createProcessor(user.preferredGateway)
success = processor.processPayment(cart.total, paymentDetails)
```

### 3. Logging Systems

```pseudocode
interface Logger:
    method log(level: String, message: String)
    method error(message: String)
    method debug(message: String)

class LoggerFactory:
    method createLogger(environment: String): Logger:
        if environment == "production":
            return new FileLogger("/var/log/app.log")
        else if environment == "development":
            return new ConsoleLogger()
        else if environment == "testing":
            return new MemoryLogger()

// Application initialization
logger = LoggerFactory.createLogger(config.environment)
logger.log("INFO", "Application started")
```

### 4. Document Parsers

```pseudocode
interface DocumentParser:
    method parse(file: File): Document
    method validate(): Boolean

class ParserFactory:
    method createParser(fileExtension: String): DocumentParser:
        switch fileExtension:
            case "pdf":
                return new PDFParser()
            case "docx":
                return new WordParser()
            case "xlsx":
                return new ExcelParser()
            case "csv":
                return new CSVParser()

// File processing system
extension = file.getExtension()
parser = ParserFactory.createParser(extension)
document = parser.parse(file)
```

### 5. Notification Systems

```pseudocode
interface Notification:
    method send(recipient: String, message: String): Boolean

class NotificationFactory:
    method createNotification(channel: String, config: Object): Notification:
        switch channel:
            case "email":
                return new EmailNotification(config.smtpServer)
            case "sms":
                return new SMSNotification(config.twilioKey)
            case "push":
                return new PushNotification(config.fcmKey)
            case "slack":
                return new SlackNotification(config.webhookUrl)

// Alert system
notifier = NotificationFactory.createNotification("email", emailConfig)
notifier.send("admin@example.com", "Critical error detected")
```

### 6. UI Component Generation

```pseudocode
interface Button:
    method render()
    method onClick(handler: Function)

class ButtonFactory:
    method createButton(platform: String): Button:
        switch platform:
            case "web":
                return new HTMLButton()
            case "mobile":
                return new NativeButton()
            case "desktop":
                return new DesktopButton()

// Cross-platform UI framework
button = ButtonFactory.createButton(currentPlatform)
button.onClick(handleClick)
button.render()
```

### 7. Game Entity Creation

```pseudocode
interface Enemy:
    method attack()
    method move()
    method takeDamage(amount: Integer)

class EnemyFactory:
    method createEnemy(type: String, level: Integer): Enemy:
        baseEnemy = switch type:
            case "zombie":
                new Zombie()
            case "skeleton":
                new Skeleton()
            case "boss":
                new BossEnemy()
        
        baseEnemy.setLevel(level)
        baseEnemy.scaleStats(level)
        return baseEnemy

// Game level manager
factory = new EnemyFactory()
enemy = factory.createEnemy("zombie", currentLevel)
spawnEnemy(enemy)
```

### 8. Data Export Formats

```pseudocode
interface DataExporter:
    method export(data: Array, filename: String)

class ExporterFactory:
    method createExporter(format: String): DataExporter:
        switch format:
            case "json":
                return new JSONExporter()
            case "xml":
                return new XMLExporter()
            case "csv":
                return new CSVExporter()
            case "excel":
                return new ExcelExporter()

// Reporting module
exporter = ExporterFactory.createExporter(user.selectedFormat)
exporter.export(reportData, "sales-report")
```

---

## Advantages

### 1. Loose Coupling

Client code depends on abstractions, not concrete implementations:

```pseudocode
// Client doesn't know about concrete classes
transport = logistics.createTransport()
transport.deliver("New York")
```

### 2. Single Responsibility Principle

Object creation logic is centralized in one place:

```pseudocode
// All vehicle creation logic in one factory
class VehicleFactory:
    method createVehicle(type: String): Vehicle:
        // Complex initialization logic here
        vehicle = instantiateVehicle(type)
        vehicle.configure()
        vehicle.validate()
        return vehicle
```

### 3. Open/Closed Principle

New product types can be added without modifying existing code:

```pseudocode
// Add new transport type by creating new subclass
class AirLogistics extends Logistics:
    method createTransport(): Transport:
        return new Airplane()

// Client code unchanged
logistics = new AirLogistics()
transport = logistics.createTransport()
```

### 4. Improved Testability

Easy to inject mock factories for testing:

```pseudocode
// Production
factory = new RealPaymentFactory()

// Testing
factory = new MockPaymentFactory()

// Same client code works with both
processor = factory.createProcessor("stripe")
```

### 5. Centralized Configuration

Object creation configuration in one location:

```pseudocode
class ServiceFactory:
    config: Configuration
    
    method createService(name: String): Service:
        service = instantiateService(name)
        service.setConfig(this.config)
        service.initialize()
        return service
```

### 6. Resource Management

Control over object lifecycle and pooling:

```pseudocode
class ConnectionFactory:
    pool: ConnectionPool
    
    method getConnection(): Connection:
        return pool.acquire()
    
    method releaseConnection(conn: Connection):
        pool.release(conn)
```

---

## Trade-offs and Considerations

### Complexity vs. Flexibility

**Trade-off:**
- Factory adds abstraction layers
- More classes and interfaces to maintain
- Steeper learning curve for new developers

**When acceptable:**
- Large codebases with multiple product types
- Frequent addition of new product variants
- Need for runtime product selection

**When problematic:**
- Small projects with few product types
- Product types are stable and rarely change
- Team prefers simplicity over flexibility

### Performance Overhead

**Consideration:**
- Factory method calls add indirection
- Virtual method dispatch (in some languages)
- Potential allocation overhead

**Mitigation:**
```pseudocode
// Cache frequently created objects
class OptimizedFactory:
    cache = {}
    
    method createProduct(type: String): Product:
        if type in cache:
            return cache[type].clone()
        
        product = actuallyCreateProduct(type)
        cache[type] = product
        return product
```

### Explosion of Classes

**Problem:**
- Each product type requires concrete factory subclass
- Can lead to class proliferation

**Solution:**
Use parameterized factory instead:

```pseudocode
// Instead of: TruckFactory, ShipFactory, PlaneFactory
class TransportFactory:
    method create(type: String): Transport:
        // Single factory handles all types
```

---

## Best Practices

### 1. Return Interfaces, Not Concrete Classes

```pseudocode
// ✅ Good - Returns interface
method createLogger(): Logger:
    return new FileLogger()

// ❌ Bad - Returns concrete class
method createLogger(): FileLogger:
    return new FileLogger()
```

### 2. Use Meaningful Factory Method Names

```pseudocode
// ✅ Good - Clear intent
method createDatabaseConnection(): Connection
method buildUserFromDTO(dto: UserDTO): User
method newInstanceOf(className: String): Object

// ❌ Bad - Vague names
method create(): Object
method make(): Thing
method get(): Something
```

### 3. Handle Unknown Types Gracefully

```pseudocode
// ✅ Good - Clear error handling
method createTransport(type: String): Transport:
    if type == "truck":
        return new Truck()
    else if type == "ship":
        return new Ship()
    else:
        throw new UnsupportedTransportTypeError(type)

// ❌ Bad - Silent failure
method createTransport(type: String): Transport:
    if type == "truck":
        return new Truck()
    return null  // What does null mean here?
```

### 4. Keep Factories Stateless When Possible

```pseudocode
// ✅ Good - Stateless factory
class StatelessFactory:
    method createService(config: Config): Service:
        return new ServiceImpl(config)

// ⚠️ Use with caution - Stateful factory
class StatefulFactory:
    lastCreated: Product
    creationCount: Integer
    
    method createProduct(): Product:
        this.lastCreated = new Product()
        this.creationCount++
        return this.lastCreated
```

### 5. Document Creation Constraints

```pseudocode
class VehicleFactory:
    /**
     * Creates a vehicle instance.
     * 
     * @param type: Must be one of: "car", "truck", "motorcycle"
     * @param config: Must contain "engineType" and "fuelCapacity"
     * @throws InvalidVehicleTypeError if type is unknown
     * @throws ConfigurationError if config is invalid
     */
    method createVehicle(type: String, config: Object): Vehicle:
        validateType(type)
        validateConfig(config)
        return instantiateVehicle(type, config)
```

### 6. Use Factory for Complex Initialization Only

```pseudocode
// ✅ Good - Complex initialization justified factory
class DatabaseFactory:
    method createConnection(config: Config): Connection:
        connection = new Connection(config.host, config.port)
        connection.authenticate(config.username, config.password)
        connection.setPoolSize(config.poolSize)
        connection.enableSSL(config.sslConfig)
        connection.setTimeouts(config.timeouts)
        return connection

// ❌ Bad - Unnecessary factory for simple object
class PointFactory:
    method createPoint(x: Integer, y: Integer): Point:
        return new Point(x, y)  // Just use constructor!
```

### 7. Consider Using Registry for Extensibility

```pseudocode
class PluginFactory:
    registry = {}
    
    static method registerPlugin(name: String, creator: Function):
        registry[name] = creator
    
    method createPlugin(name: String): Plugin:
        if name not in registry:
            throw new UnknownPluginError(name)
        
        creator = registry[name]
        return creator()

// Third-party plugins can register themselves
PluginFactory.registerPlugin("analytics", function():
    return new AnalyticsPlugin()
)
```

### 8. Validate Products After Creation

```pseudocode
class ValidatingFactory:
    method createProduct(type: String): Product:
        product = this.instantiateProduct(type)
        
        if not product.isValid():
            throw new InvalidProductError("Product failed validation")
        
        return product
```

---

## Anti-Patterns

### Anti-Pattern 1: God Factory

**Problem:** Single factory creates every type of object in the system

```pseudocode
// ❌ Bad - Factory does too much
class GodFactory:
    method create(type: String): Object:
        switch type:
            case "user": return new User()
            case "order": return new Order()
            case "product": return new Product()
            case "payment": return new Payment()
            case "shipment": return new Shipment()
            case "invoice": return new Invoice()
            // ... 50 more types
```

**Solution:** Create separate factories for different domains

```pseudocode
// ✅ Good - Separate domain factories
class UserFactory:
    method createUser(): User
    method createAdmin(): Admin

class OrderFactory:
    method createOrder(): Order
    method createQuote(): Quote

class PaymentFactory:
    method createPayment(): Payment
    method createRefund(): Refund
```

### Anti-Pattern 2: Factory Returning Null

**Problem:** Factory returns null for unknown types instead of throwing error

```pseudocode
// ❌ Bad - Silent failure
method createTransport(type: String): Transport:
    if type == "truck":
        return new Truck()
    return null  // Caller must check for null

// Client code
transport = factory.createTransport("unknown")
if transport != null:  // Easy to forget this check!
    transport.deliver()
```

**Solution:** Throw explicit exceptions

```pseudocode
// ✅ Good - Fail fast with clear error
method createTransport(type: String): Transport:
    if type == "truck":
        return new Truck()
    else if type == "ship":
        return new Ship()
    else:
        throw new UnknownTransportTypeError(
            "Unknown transport type: " + type
        )
```

### Anti-Pattern 3: Factory with Business Logic

**Problem:** Factory contains domain logic beyond object creation

```pseudocode
// ❌ Bad - Factory has business logic
class OrderFactory:
    method createOrder(items: Array): Order:
        order = new Order()
        
        // This is business logic, not factory responsibility!
        discount = this.calculateDiscount(items)
        tax = this.calculateTax(items, discount)
        shipping = this.determineShipping(items)
        
        order.setDiscount(discount)
        order.setTax(tax)
        order.setShipping(shipping)
        
        return order
```

**Solution:** Keep factories focused on object creation

```pseudocode
// ✅ Good - Factory creates, services handle logic
class OrderFactory:
    method createOrder(): Order:
        return new Order()

class OrderService:
    factory: OrderFactory
    discountService: DiscountService
    taxService: TaxService
    
    method createOrderFromItems(items: Array): Order:
        order = factory.createOrder()
        order.setItems(items)
        
        discount = discountService.calculate(items)
        tax = taxService.calculate(items, discount)
        
        order.setDiscount(discount)
        order.setTax(tax)
        
        return order
```

### Anti-Pattern 4: Unnecessary Factory Hierarchy

**Problem:** Deep inheritance hierarchy for simple factory logic

```pseudocode
// ❌ Bad - Over-engineered hierarchy
abstract class AbstractVehicleFactory:
    abstract method createVehicle(): Vehicle

abstract class AbstractLandVehicleFactory extends AbstractVehicleFactory:
    // ...

abstract class AbstractMotorizedVehicleFactory extends AbstractLandVehicleFactory:
    // ...

class CarFactory extends AbstractMotorizedVehicleFactory:
    method createVehicle(): Vehicle:
        return new Car()
```

**Solution:** Use simple factory or parameterized approach

```pseudocode
// ✅ Good - Flat, simple structure
class VehicleFactory:
    method createVehicle(type: String): Vehicle:
        switch type:
            case "car": return new Car()
            case "truck": return new Truck()
            case "motorcycle": return new Motorcycle()
```

### Anti-Pattern 5: Stateful Factory with Side Effects

**Problem:** Factory maintains state that affects creation

```pseudocode
// ❌ Bad - Factory state causes unpredictable behavior
class StatefulProductFactory:
    counter: Integer = 0
    lastProduct: Product
    
    method createProduct(): Product:
        this.counter++
        
        // Side effect: modifies global state
        Database.incrementProductCount()
        
        if this.counter % 2 == 0:
            this.lastProduct = new ProductA()
        else:
            this.lastProduct = new ProductB()
        
        return this.lastProduct

// Unpredictable behavior
product1 = factory.createProduct()  // ProductB
product2 = factory.createProduct()  // ProductA (different type!)
```

**Solution:** Keep factories stateless and side-effect free

```pseudocode
// ✅ Good - Stateless factory
class ProductFactory:
    method createProduct(type: String): Product:
        return switch type:
            case "A": new ProductA()
            case "B": new ProductB()
```

### Anti-Pattern 6: Factory Method That Doesn't Create

**Problem:** Factory method just retrieves existing objects without clear naming

```pseudocode
// ❌ Bad - Misleading name
class UserFactory:
    method createUser(id: String): User:
        // Doesn't create! Retrieves from database
        return Database.findUserById(id)
```

**Solution:** Use appropriate naming or repository pattern

```pseudocode
// ✅ Good - Clear naming
class UserRepository:
    method findById(id: String): User:
        return Database.findUserById(id)

class UserFactory:
    method createUser(name: String, email: String): User:
        // Actually creates new user
        return new User(name, email)
```

---

## Comparison with Related Patterns

### Factory Pattern vs. Builder Pattern

| Aspect | Factory Pattern | Builder Pattern |
|--------|----------------|-----------------|
| **Purpose** | Create different types of objects | Construct complex objects step-by-step |
| **When to use** | Multiple product variants | Single complex product |
| **Complexity** | Simple creation | Multi-step construction |
| **Flexibility** | Choose product type | Configure product details |

**Factory Example:**
```pseudocode
// Different types
transport = factory.createTransport("truck")  // or "ship" or "plane"
```

**Builder Example:**
```pseudocode
// Same type, different configuration
house = new HouseBuilder()
    .setWalls(4)
    .setRoof("tile")
    .setGarage(true)
    .build()
```

**When to combine:**
```pseudocode
// Use factory to get right builder for product type
builder = BuilderFactory.createBuilder("luxury_house")
house = builder
    .setWalls(6)
    .setPool(true)
    .build()
```

### Factory Pattern vs. Abstract Factory Pattern

| Aspect | Factory Method | Abstract Factory |
|--------|---------------|------------------|
| **Scope** | Creates single product | Creates families of products |
| **Structure** | Single method | Multiple methods |
| **Complexity** | Simpler | More complex |
| **Use case** | One product type varies | Multiple related products vary |

**Factory Method Example:**
```pseudocode
interface DocumentFactory:
    method createDocument(): Document

class PDFFactory implements DocumentFactory:
    method createDocument(): Document:
        return new PDFDocument()
```

**Abstract Factory Example:**
```pseudocode
interface UIFactory:
    method createButton(): Button
    method createCheckbox(): Checkbox
    method createTextField(): TextField

class WindowsUIFactory implements UIFactory:
    method createButton(): Button:
        return new WindowsButton()
    
    method createCheckbox(): Checkbox:
        return new WindowsCheckbox()
    
    method createTextField(): TextField:
        return new WindowsTextField()
```

### Factory Pattern vs. Prototype Pattern

| Aspect | Factory Pattern | Prototype Pattern |
|--------|----------------|-------------------|
| **Creation method** | Instantiates from scratch | Clones existing object |
| **Performance** | May be slower | Faster for complex objects |
| **Use case** | Creation logic varies by type | Object is expensive to create |
| **Configuration** | Constructor parameters | Modify cloned object |

**Factory Example:**
```pseudocode
class VehicleFactory:
    method createVehicle(type: String): Vehicle:
        return new Vehicle(type)  // Create from scratch
```

**Prototype Example:**
```pseudocode
class VehiclePrototype:
    prototype: Vehicle
    
    method clone(): Vehicle:
        return prototype.clone()  // Copy existing object
```

### Factory Pattern vs. Dependency Injection

| Aspect | Factory Pattern | Dependency Injection |
|--------|----------------|---------------------|
| **Control** | Client controls creation | Container controls creation |
| **When** | Runtime creation decision | Startup/configuration time |
| **Coupling** | Client knows factory | Client knows nothing |
| **Flexibility** | Programmatic | Declarative (config) |

**Factory Example:**
```pseudocode
class Service:
    method process():
        // Service decides which repository to use
        repository = RepositoryFactory.create("mysql")
        data = repository.fetch()
```

**Dependency Injection Example:**
```pseudocode
class Service:
    repository: Repository
    
    constructor(repository: Repository):
        // Container injects dependency
        this.repository = repository
    
    method process():
        data = this.repository.fetch()
```

**When to combine:**
```pseudocode
// Factory injected via DI
class Service:
    factory: RepositoryFactory
    
    constructor(factory: RepositoryFactory):
        this.factory = factory
    
    method process(dbType: String):
        // Use injected factory at runtime
        repository = this.factory.create(dbType)
        data = repository.fetch()
```

### Factory Pattern vs. Service Locator

| Aspect | Factory Pattern | Service Locator |
|--------|----------------|-----------------|
| **Purpose** | Create objects | Locate/retrieve services |
| **Scope** | Specific product types | Any service in system |
| **Coupling** | To factory interface | To service locator |
| **Testability** | Easy to mock | Harder to mock |

**Factory Example:**
```pseudocode
// Specific to product type
logger = LoggerFactory.createLogger("file")
```

**Service Locator Example:**
```pseudocode
// Generic service retrieval
logger = ServiceLocator.get("Logger")
cache = ServiceLocator.get("Cache")
database = ServiceLocator.get("Database")
```

---

## Decision Framework

### Should You Use Factory Pattern?

```pseudocode
function shouldUseFactory(situation):
    // Simple cases - NO factory needed
    if situation.productTypes == 1:
        return NO, "Use direct instantiation"
    
    if situation.creationLogic == "simple":
        return NO, "Constructor is sufficient"
    
    // Complex cases - YES factory recommended
    if situation.productTypes > 3:
        return YES, "Use Factory Method or Abstract Factory"
    
    if situation.creationLogic == "complex":
        return YES, "Centralize creation logic in factory"
    
    if situation.typeDeterminedAt == "runtime":
        return YES, "Use parameterized factory"
    
    if situation.needsExtensibility == true:
        return YES, "Use Factory Method pattern"
    
    // Default
    return MAYBE, "Consider trade-offs carefully"
```

### Which Factory Variant?

```
START
  |
  v
Do you have multiple product types?
  |
  |--NO--> Use direct instantiation (no factory)
  |
  |--YES--> Is creation logic complex?
      |
      |--NO--> Use Simple Factory
      |
      |--YES--> Do you need extensibility via inheritance?
          |
          |--NO--> Use Simple Factory with registry
          |
          |--YES--> Do you create families of related products?
              |
              |--NO--> Use Factory Method Pattern
              |
              |--YES--> Use Abstract Factory Pattern
```

### Decision Table

| Scenario | Recommended Pattern | Reason |
|----------|-------------------|--------|
| 2-3 product types, simple creation | Simple Factory | Minimizes complexity |
| 5+ product types, complex creation | Factory Method | Extensible, maintainable |
| Multiple related product families | Abstract Factory | Ensures consistency |
| Runtime type determination | Parameterized Factory | Flexible selection |
| Object pooling/reuse | Factory with cache | Resource efficiency |
| Plugin architecture | Registry-based Factory | Third-party extensibility |
| Single product type | No factory | YAGNI principle |
| Simple objects (x, y coordinates) | Direct instantiation | No abstraction needed |

### Questions to Ask

1. **How many product types exist?**
   - 1 type → No factory
   - 2-3 types → Simple factory
   - 4+ types → Factory Method

2. **Will new product types be added frequently?**
   - Yes → Factory Method (extensible)
   - No → Simple Factory (simpler)

3. **Is creation logic complex?**
   - Yes → Factory (centralize complexity)
   - No → Direct instantiation

4. **Do products have related families?**
   - Yes → Abstract Factory
   - No → Factory Method

5. **Is runtime selection needed?**
   - Yes → Parameterized Factory
   - No → May not need factory

6. **Is the team comfortable with patterns?**
   - Yes → Use appropriate factory variant
   - No → Start with Simple Factory, refactor later

---

## Summary

### Key Takeaways

1. **Purpose**: Factory Pattern provides an interface for creating objects without specifying exact classes
2. **Variants**: Simple Factory, Factory Method, Abstract Factory
3. **Benefits**: Loose coupling, extensibility, testability, centralized creation logic
4. **Trade-offs**: Added complexity, more classes, potential performance overhead
5. **Use when**: Multiple product types, complex creation, runtime selection needed
6. **Avoid when**: Single product type, simple creation, over-engineering

### Quick Reference

**Simple Factory:**
```pseudocode
class Factory:
    method create(type): Product:
        switch type:
            case "A": return new ProductA()
            case "B": return new ProductB()
```

**Factory Method:**
```pseudocode
abstract class Creator:
    abstract method createProduct(): Product
    
class ConcreteCreator extends Creator:
    method createProduct(): Product:
        return new ConcreteProduct()
```

**Abstract Factory:**
```pseudocode
interface AbstractFactory:
    method createProductA(): ProductA
    method createProductB(): ProductB
```

### When to Use This Guideline

- Designing new systems with multiple product types
- Refactoring tightly coupled object creation code
- Building extensible frameworks or libraries
- Implementing plugin architectures
- Need to decouple client code from concrete implementations

---

## References

This guideline synthesized knowledge from:

- Gang of Four: *Design Patterns: Elements of Reusable Object-Oriented Software*
- GeeksforGeeks: Factory Method for Designing Pattern
- Wikipedia: Factory Method Pattern
- Refactoring.Guru: Factory Method Design Pattern
- Industry best practices and real-world applications

---

**Version:** 1.0  
**Last Updated:** 2026-01-25  
**Maintainer:** Architecture Guidelines Team
