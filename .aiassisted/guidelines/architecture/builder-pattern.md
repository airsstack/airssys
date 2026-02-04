# Builder Design Pattern Guidelines

> **Purpose**: This guideline provides comprehensive principles, patterns, and best practices for implementing the Builder Design Pattern in any programming language and technology stack.

---

## Table of Contents

1. [Introduction](#introduction)
2. [What is the Builder Pattern?](#what-is-the-builder-pattern)
3. [When to Use the Builder Pattern](#when-to-use-the-builder-pattern)
4. [Core Components](#core-components)
5. [Implementation Patterns](#implementation-patterns)
6. [Advantages and Trade-offs](#advantages-and-trade-offs)
7. [Best Practices](#best-practices)
8. [Anti-Patterns to Avoid](#anti-patterns-to-avoid)
9. [Comparison with Related Patterns](#comparison-with-related-patterns)
10. [Real-World Applications](#real-world-applications)

---

## Introduction

### Definition

The **Builder Pattern** is a creational design pattern that separates the construction of a complex object from its representation. It provides a step-by-step approach to constructing objects, allowing the same construction process to create different variations of the object.

### Key Characteristics

- **Step-by-Step Construction**: Build complex objects incrementally
- **Separation of Concerns**: Construction logic is separate from the object itself
- **Flexible Representations**: Same process creates different variations
- **Improved Readability**: Fluent interfaces make code self-documenting
- **Validation Support**: Validate object state before creation

---

## What is the Builder Pattern?

### The Problem

When creating complex objects, you often face several challenges:

**1. Telescoping Constructors**

```pseudocode
// ❌ BAD: Constructor with many parameters
class Pizza {
    constructor(
        size,
        cheese,
        pepperoni,
        bacon,
        mushrooms,
        onions,
        olives,
        tomatoes,
        extraCheese
    ) {
        // Initialize all fields
    }
}

// Usage becomes confusing
pizza = new Pizza(12, true, false, true, null, null, null, false, true)
// What do these boolean values mean?
```

**2. Multiple Constructor Overloads**

```pseudocode
// ❌ BAD: Multiple constructors for different combinations
class Pizza {
    constructor(size) { }
    constructor(size, cheese) { }
    constructor(size, cheese, pepperoni) { }
    constructor(size, cheese, pepperoni, bacon) { }
    // ... many more combinations
}
```

**3. Incomplete or Invalid Objects**

```pseudocode
// ❌ BAD: Object created in invalid state
pizza = new Pizza()
pizza.setSize(12)
// Missing required toppings, but object exists
```

### The Solution

The Builder Pattern addresses these issues by:

1. **Extracting Construction Logic**: Move object creation to a dedicated Builder class
2. **Providing Fluent Interface**: Chain method calls for readability
3. **Enforcing Validation**: Validate object state before creation
4. **Supporting Optional Parameters**: Set only what's needed

```pseudocode
// ✅ GOOD: Using Builder Pattern
pizza = PizzaBuilder.create()
    .setSize(12)
    .addCheese()
    .addPepperoni()
    .addBacon()
    .addExtraCheese()
    .build()
```

---

## When to Use the Builder Pattern

### Use Cases

#### ✅ Ideal For:

1. **Objects with Many Optional Parameters**
   - More than 4-5 constructor parameters
   - Many combinations of optional fields
   - Need for default values

2. **Complex Object Creation**
   - Multi-step construction process
   - Objects with nested structures
   - Composite trees or hierarchies

3. **Immutable Objects**
   - Once created, object cannot be modified
   - All fields must be set during construction
   - Thread-safe objects

4. **Different Representations**
   - Same construction process, different outputs
   - Multiple variants of the same product
   - Configuration-based creation

5. **Validation Requirements**
   - Need to validate before object creation
   - Cross-field validation needed
   - Business rules must be enforced

#### ❌ Avoid When:

1. **Simple Objects**
   - 3 or fewer required parameters
   - No optional fields
   - No validation needed

2. **Frequently Changed Objects**
   - Object needs setters after creation
   - Mutable state is acceptable
   - Simple POJO/DTO classes

3. **Performance-Critical Code**
   - Extra object creation overhead unacceptable
   - Direct construction is faster
   - Memory constraints exist

---

## Core Components

### 1. Product

The complex object being built.

```pseudocode
// Product: The final object we want to create
class Computer {
    private cpu
    private ram
    private storage
    private gpu
    private cooling
    
    // Constructor typically private or package-private
    constructor(cpu, ram, storage, gpu, cooling) {
        this.cpu = cpu
        this.ram = ram
        this.storage = storage
        this.gpu = gpu
        this.cooling = cooling
    }
    
    function displaySpecs() {
        print("CPU: " + this.cpu)
        print("RAM: " + this.ram)
        print("Storage: " + this.storage)
        print("GPU: " + this.gpu)
        print("Cooling: " + this.cooling)
    }
}
```

### 2. Builder Interface

Defines the construction steps.

```pseudocode
// Builder Interface: Defines steps for building
interface ComputerBuilder {
    function setCPU(cpu)
    function setRAM(ram)
    function setStorage(storage)
    function setGPU(gpu)
    function setCooling(cooling)
    function build(): Computer
}
```

### 3. Concrete Builder

Implements the builder with specific logic.

```pseudocode
// Concrete Builder: Implements construction steps
class GamingComputerBuilder implements ComputerBuilder {
    private cpu
    private ram
    private storage
    private gpu
    private cooling
    
    function setCPU(cpu) {
        this.cpu = cpu
        return this  // Return self for method chaining
    }
    
    function setRAM(ram) {
        this.ram = ram
        return this
    }
    
    function setStorage(storage) {
        this.storage = storage
        return this
    }
    
    function setGPU(gpu) {
        this.gpu = gpu
        return this
    }
    
    function setCooling(cooling) {
        this.cooling = cooling
        return this
    }
    
    function build(): Computer {
        // Validate before creating
        this.validate()
        
        // Create and return the product
        return new Computer(
            this.cpu,
            this.ram,
            this.storage,
            this.gpu,
            this.cooling
        )
    }
    
    private function validate() {
        if (this.cpu == null) {
            throw Error("CPU is required")
        }
        if (this.ram == null) {
            throw Error("RAM is required")
        }
        if (this.storage == null) {
            throw Error("Storage is required")
        }
    }
}
```

### 4. Director (Optional)

Controls the building process.

```pseudocode
// Director: Manages the construction process
class ComputerDirector {
    private builder
    
    constructor(builder: ComputerBuilder) {
        this.builder = builder
    }
    
    function constructGamingPC() {
        return this.builder
            .setCPU("Intel i9-13900K")
            .setRAM("32GB DDR5")
            .setStorage("2TB NVMe SSD")
            .setGPU("NVIDIA RTX 4090")
            .setCooling("Liquid Cooling")
            .build()
    }
    
    function constructOfficePC() {
        return this.builder
            .setCPU("Intel i5-13400")
            .setRAM("16GB DDR4")
            .setStorage("512GB SSD")
            .setGPU("Integrated Graphics")
            .setCooling("Air Cooling")
            .build()
    }
}
```

### 5. Client

Uses the builder to create objects.

```pseudocode
// Client: Uses the builder
function main() {
    // Direct builder usage
    gamingPC = new GamingComputerBuilder()
        .setCPU("Intel i9-13900K")
        .setRAM("32GB DDR5")
        .setStorage("2TB NVMe SSD")
        .setGPU("NVIDIA RTX 4090")
        .setCooling("Liquid Cooling")
        .build()
    
    gamingPC.displaySpecs()
    
    // Using director
    builder = new GamingComputerBuilder()
    director = new ComputerDirector(builder)
    
    officePC = director.constructOfficePC()
    officePC.displaySpecs()
}
```

---

## Implementation Patterns

### Pattern 1: Classic Builder (GoF)

The traditional Gang of Four implementation with separate Builder interface.

```pseudocode
// Product
class House {
    private walls
    private roof
    private doors
    private windows
    private garage
    
    constructor(walls, roof, doors, windows, garage) {
        this.walls = walls
        this.roof = roof
        this.doors = doors
        this.windows = windows
        this.garage = garage
    }
}

// Abstract Builder
interface HouseBuilder {
    function buildWalls()
    function buildRoof()
    function buildDoors()
    function buildWindows()
    function buildGarage()
    function getHouse(): House
}

// Concrete Builder
class WoodenHouseBuilder implements HouseBuilder {
    private house
    
    constructor() {
        this.reset()
    }
    
    function reset() {
        this.house = new House(null, null, null, null, null)
    }
    
    function buildWalls() {
        this.house.walls = "Wooden Walls"
    }
    
    function buildRoof() {
        this.house.roof = "Wooden Roof"
    }
    
    function buildDoors() {
        this.house.doors = "Wooden Doors"
    }
    
    function buildWindows() {
        this.house.windows = "Wooden Windows"
    }
    
    function buildGarage() {
        this.house.garage = "Wooden Garage"
    }
    
    function getHouse(): House {
        result = this.house
        this.reset()
        return result
    }
}

// Director
class ConstructionDirector {
    function constructSimpleHouse(builder: HouseBuilder) {
        builder.buildWalls()
        builder.buildRoof()
        builder.buildDoors()
        return builder.getHouse()
    }
    
    function constructFullHouse(builder: HouseBuilder) {
        builder.buildWalls()
        builder.buildRoof()
        builder.buildDoors()
        builder.buildWindows()
        builder.buildGarage()
        return builder.getHouse()
    }
}
```

### Pattern 2: Fluent Builder (Modern)

The modern approach with method chaining and no separate interface.

```pseudocode
// Product
class User {
    private username
    private email
    private firstName
    private lastName
    private age
    private phoneNumber
    private address
    
    // Private constructor - only builder can create
    private constructor(builder: UserBuilder) {
        this.username = builder.username
        this.email = builder.email
        this.firstName = builder.firstName
        this.lastName = builder.lastName
        this.age = builder.age
        this.phoneNumber = builder.phoneNumber
        this.address = builder.address
    }
    
    // Static method to start building
    static function builder(): UserBuilder {
        return new UserBuilder()
    }
    
    // Nested Builder class
    class UserBuilder {
        private username
        private email
        private firstName
        private lastName
        private age
        private phoneNumber
        private address
        
        // Required fields in constructor
        constructor(username, email) {
            this.username = username
            this.email = email
        }
        
        function setFirstName(firstName) {
            this.firstName = firstName
            return this
        }
        
        function setLastName(lastName) {
            this.lastName = lastName
            return this
        }
        
        function setAge(age) {
            this.age = age
            return this
        }
        
        function setPhoneNumber(phoneNumber) {
            this.phoneNumber = phoneNumber
            return this
        }
        
        function setAddress(address) {
            this.address = address
            return this
        }
        
        function build(): User {
            this.validate()
            return new User(this)
        }
        
        private function validate() {
            if (this.username == null || this.username.isEmpty()) {
                throw Error("Username is required")
            }
            if (this.email == null || !this.email.isValidEmail()) {
                throw Error("Valid email is required")
            }
            if (this.age != null && this.age < 0) {
                throw Error("Age must be positive")
            }
        }
    }
}

// Usage
user = User.builder("johndoe", "john@example.com")
    .setFirstName("John")
    .setLastName("Doe")
    .setAge(30)
    .setPhoneNumber("+1-555-0123")
    .build()
```

### Pattern 3: Step Builder

Enforces construction order using type system.

```pseudocode
// Product
class DatabaseConnection {
    private host
    private port
    private username
    private password
    private database
    
    private constructor(host, port, username, password, database) {
        this.host = host
        this.port = port
        this.username = username
        this.password = password
        this.database = database
    }
}

// Builder with steps enforced through interfaces
interface HostStep {
    function host(host): PortStep
}

interface PortStep {
    function port(port): UsernameStep
}

interface UsernameStep {
    function username(username): PasswordStep
}

interface PasswordStep {
    function password(password): DatabaseStep
}

interface DatabaseStep {
    function database(database): BuildStep
}

interface BuildStep {
    function build(): DatabaseConnection
}

// Concrete builder implementing all steps
class DatabaseConnectionBuilder implements 
    HostStep, PortStep, UsernameStep, PasswordStep, DatabaseStep, BuildStep {
    
    private host
    private port
    private username
    private password
    private database
    
    static function create(): HostStep {
        return new DatabaseConnectionBuilder()
    }
    
    function host(host): PortStep {
        this.host = host
        return this
    }
    
    function port(port): UsernameStep {
        this.port = port
        return this
    }
    
    function username(username): PasswordStep {
        this.username = username
        return this
    }
    
    function password(password): DatabaseStep {
        this.password = password
        return this
    }
    
    function database(database): BuildStep {
        this.database = database
        return this
    }
    
    function build(): DatabaseConnection {
        return new DatabaseConnection(
            this.host,
            this.port,
            this.username,
            this.password,
            this.database
        )
    }
}

// Usage: Type system enforces order
connection = DatabaseConnectionBuilder.create()
    .host("localhost")
    .port(5432)
    .username("admin")
    .password("secret")
    .database("mydb")
    .build()

// This won't compile/run:
// connection = DatabaseConnectionBuilder.create()
//     .port(5432)  // ERROR: must set host first
```

### Pattern 4: Builder with Defaults

Provides sensible defaults for optional fields.

```pseudocode
// Product
class HttpRequest {
    private method
    private url
    private headers
    private body
    private timeout
    private retries
    
    constructor(builder: HttpRequestBuilder) {
        this.method = builder.method
        this.url = builder.url
        this.headers = builder.headers
        this.body = builder.body
        this.timeout = builder.timeout
        this.retries = builder.retries
    }
}

// Builder with defaults
class HttpRequestBuilder {
    private method = "GET"  // Default
    private url
    private headers = {}    // Default
    private body = null     // Default
    private timeout = 30    // Default: 30 seconds
    private retries = 3     // Default: 3 retries
    
    constructor(url) {
        this.url = url  // Required field
    }
    
    function setMethod(method) {
        this.method = method
        return this
    }
    
    function setHeaders(headers) {
        this.headers = headers
        return this
    }
    
    function setBody(body) {
        this.body = body
        return this
    }
    
    function setTimeout(timeout) {
        this.timeout = timeout
        return this
    }
    
    function setRetries(retries) {
        this.retries = retries
        return this
    }
    
    function build(): HttpRequest {
        return new HttpRequest(this)
    }
}

// Usage: Only set what differs from defaults
request = new HttpRequestBuilder("https://api.example.com/users")
    .setMethod("POST")
    .setBody({"name": "John"})
    .build()
// Uses default headers, timeout, and retries
```

---

## Advantages and Trade-offs

### Advantages

#### 1. Improved Readability

```pseudocode
// Before: Hard to understand
order = new Order(1, "John", "123 Main St", true, false, "credit", 
                  null, 10.5, true, "2024-01-25")

// After: Self-documenting
order = OrderBuilder.create()
    .setCustomerId(1)
    .setCustomerName("John")
    .setShippingAddress("123 Main St")
    .enableExpressShipping()
    .setPaymentMethod("credit")
    .setDiscount(10.5)
    .requireSignature()
    .setDeliveryDate("2024-01-25")
    .build()
```

#### 2. Flexible Object Construction

```pseudocode
// Create different configurations easily
basicPhone = PhoneBuilder.create()
    .setModel("Basic-100")
    .setStorage("64GB")
    .build()

premiumPhone = PhoneBuilder.create()
    .setModel("Premium-Pro")
    .setStorage("512GB")
    .addCamera("48MP Main")
    .addCamera("12MP Ultra-wide")
    .addCamera("8MP Telephoto")
    .enableWirelessCharging()
    .enableWaterResistance()
    .build()
```

#### 3. Immutability Support

```pseudocode
// Object is immutable after creation
class Person {
    private final name
    private final age
    
    private constructor(builder) {
        this.name = builder.name
        this.age = builder.age
    }
    
    // No setters - object cannot be modified
}
```

#### 4. Validation Before Creation

```pseudocode
class PasswordBuilder {
    private password
    
    function setPassword(password) {
        this.password = password
        return this
    }
    
    function build(): Password {
        // Validate before creating
        if (this.password.length < 8) {
            throw Error("Password must be at least 8 characters")
        }
        if (!this.password.hasUpperCase()) {
            throw Error("Password must contain uppercase letter")
        }
        if (!this.password.hasNumber()) {
            throw Error("Password must contain number")
        }
        
        return new Password(this.password)
    }
}
```

#### 5. Step-by-Step Construction

```pseudocode
// Build complex objects incrementally
documentBuilder = DocumentBuilder.create()

// Add sections step by step
documentBuilder
    .addTitle("Annual Report")
    .addSection("Introduction")
    .addParagraph("This report summarizes...")
    .addSection("Financial Overview")
    .addTable(financialData)
    .addSection("Conclusion")
    .addParagraph("In conclusion...")

document = documentBuilder.build()
```

### Disadvantages

#### 1. Increased Code Complexity

```pseudocode
// Simple object doesn't need builder
// ❌ Over-engineering
point = PointBuilder.create()
    .setX(10)
    .setY(20)
    .build()

// ✅ Direct construction is simpler
point = new Point(10, 20)
```

#### 2. Additional Classes

- Need to create and maintain builder classes
- More files and code to manage
- Increased memory footprint

#### 3. Performance Overhead

- Extra object creation (builder instance)
- Method call overhead for chaining
- Not suitable for performance-critical paths

#### 4. Verbosity

```pseudocode
// Builder can be verbose for simple cases
user = new UserBuilder()
    .setName("John")
    .build()

// vs simple constructor
user = new User("John")
```

---

## Best Practices

### 1. Return `this` for Method Chaining

```pseudocode
// ✅ GOOD: Return this for fluent API
class Builder {
    function setValue(value) {
        this.value = value
        return this  // Enable chaining
    }
}

// ❌ BAD: Void return prevents chaining
class Builder {
    function setValue(value) {
        this.value = value
        // No return
    }
}
```

### 2. Make Product Constructor Private

```pseudocode
// ✅ GOOD: Private constructor enforces builder usage
class User {
    private constructor(builder) {
        // Only builder can create User
    }
}

// ❌ BAD: Public constructor allows bypassing builder
class User {
    public constructor(name, email, age) {
        // Direct instantiation possible
    }
}
```

### 3. Validate in build() Method

```pseudocode
// ✅ GOOD: Validate before object creation
class Builder {
    function build() {
        this.validate()
        return new Product(this)
    }
    
    private function validate() {
        if (this.requiredField == null) {
            throw Error("Required field missing")
        }
    }
}

// ❌ BAD: No validation
class Builder {
    function build() {
        return new Product(this)  // May create invalid object
    }
}
```

### 4. Use Descriptive Method Names

```pseudocode
// ✅ GOOD: Clear, expressive names
builder
    .enableAutoSave()
    .disableNotifications()
    .withDarkTheme()
    .addFeature("spell-check")

// ❌ BAD: Unclear names
builder
    .set(true)
    .option(false)
    .add("dark")
    .put("spell-check")
```

### 5. Group Related setters

```pseudocode
// ✅ GOOD: Logical grouping
builder
    // Authentication
    .setUsername("john")
    .setPassword("secret")
    // Profile
    .setFirstName("John")
    .setLastName("Doe")
    // Preferences
    .setTheme("dark")
    .setLanguage("en")
    .build()
```

### 6. Provide Static Factory Method

```pseudocode
// ✅ GOOD: Convenient static method
class User {
    static function builder(): UserBuilder {
        return new UserBuilder()
    }
}

// Usage
user = User.builder()
    .setName("John")
    .build()

// ❌ BAD: Need to instantiate builder manually
builder = new UserBuilder()
user = builder.setName("John").build()
```

### 7. Reset Builder After build()

```pseudocode
// ✅ GOOD: Reset state after building
class Builder {
    function build() {
        result = new Product(this)
        this.reset()  // Ready for next build
        return result
    }
    
    private function reset() {
        this.field1 = null
        this.field2 = null
    }
}
```

### 8. Document Required vs Optional Fields

```pseudocode
// ✅ GOOD: Clear documentation
class UserBuilder {
    /**
     * Required fields:
     * - username
     * - email
     * 
     * Optional fields:
     * - firstName
     * - lastName
     * - phoneNumber
     */
    
    function build() {
        this.validateRequiredFields()
        return new User(this)
    }
}
```

---

## Anti-Patterns to Avoid

### 1. ❌ Builder for Simple Objects

```pseudocode
// ❌ BAD: Over-engineering simple object
class Point {
    private x
    private y
}

class PointBuilder {
    private x
    private y
    
    function setX(x) {
        this.x = x
        return this
    }
    
    function setY(y) {
        this.y = y
        return this
    }
    
    function build() {
        return new Point(this.x, this.y)
    }
}

// ✅ GOOD: Direct construction for simple objects
point = new Point(10, 20)
```

### 2. ❌ Mixing Builder with Setters

```pseudocode
// ❌ BAD: Allows modification after creation
class User {
    private name
    
    private constructor(builder) {
        this.name = builder.name
    }
    
    // Setter defeats immutability
    function setName(name) {
        this.name = name
    }
}

// ✅ GOOD: Immutable object
class User {
    private final name
    
    private constructor(builder) {
        this.name = builder.name
    }
    
    // No setters - immutable
}
```

### 3. ❌ No Validation in Builder

```pseudocode
// ❌ BAD: No validation
class EmailBuilder {
    private email
    
    function setEmail(email) {
        this.email = email
        return this
    }
    
    function build() {
        return new Email(this.email)
    }
}

// ✅ GOOD: Validate before creation
class EmailBuilder {
    private email
    
    function setEmail(email) {
        this.email = email
        return this
    }
    
    function build() {
        if (!this.email.isValid()) {
            throw Error("Invalid email format")
        }
        return new Email(this.email)
    }
}
```

### 4. ❌ Not Returning `this`

```pseudocode
// ❌ BAD: Cannot chain methods
class Builder {
    function setValue(value) {
        this.value = value
        // No return
    }
}

// Must call methods separately
builder = new Builder()
builder.setValue(10)
builder.setName("test")
result = builder.build()

// ✅ GOOD: Fluent interface
class Builder {
    function setValue(value) {
        this.value = value
        return this
    }
}

// Can chain calls
result = new Builder()
    .setValue(10)
    .setName("test")
    .build()
```

### 5. ❌ Exposing Builder State

```pseudocode
// ❌ BAD: Exposing internal state
class Builder {
    public value  // Public field
    
    function build() {
        return new Product(this.value)
    }
}

// Can be modified externally
builder = new Builder()
builder.value = 100  // Direct access
product = builder.build()

// ✅ GOOD: Encapsulated state
class Builder {
    private value
    
    function setValue(value) {
        this.value = value
        return this
    }
    
    function build() {
        return new Product(this.value)
    }
}
```

### 6. ❌ Mutable Product from Builder

```pseudocode
// ❌ BAD: Builder returns mutable object
class Builder {
    private data = []
    
    function addItem(item) {
        this.data.add(item)
        return this
    }
    
    function build() {
        return this.data  // Returns mutable reference
    }
}

// Can modify after building
builder = new Builder()
list = builder.addItem("a").build()
list.add("b")  // Mutates the built object

// ✅ GOOD: Builder returns immutable copy
class Builder {
    private data = []
    
    function addItem(item) {
        this.data.add(item)
        return this
    }
    
    function build() {
        return this.data.copy()  // Returns copy
    }
}
```

---

## Comparison with Related Patterns

### Builder vs Factory Pattern

| Aspect | Builder | Factory |
|--------|---------|---------|
| **Purpose** | Step-by-step complex object construction | Create objects based on type/criteria |
| **Complexity** | Handles complex objects with many fields | Handles creation of related object families |
| **Configuration** | Highly configurable with many options | Limited configuration, predefined variants |
| **Return Type** | Usually single product type | Can return different types (polymorphism) |
| **Usage** | `builder.setA().setB().build()` | `factory.create(type)` |

```pseudocode
// Factory Pattern
interface ShapeFactory {
    function createShape(type): Shape
}

shape = factory.createShape("circle")

// Builder Pattern
shape = CircleBuilder.create()
    .setRadius(10)
    .setColor("red")
    .setPosition(100, 100)
    .build()
```

### Builder vs Prototype Pattern

| Aspect | Builder | Prototype |
|--------|---------|-----------|
| **Creation** | Constructs new object from scratch | Clones existing object |
| **Customization** | Build with different configurations | Copy and modify |
| **Complexity** | Good for complex construction logic | Good for avoiding expensive initialization |
| **Usage** | Step-by-step configuration | Clone and customize |

```pseudocode
// Prototype Pattern
original = new Configuration()
copy = original.clone()
copy.setSetting("value")

// Builder Pattern
config = ConfigurationBuilder.create()
    .setSetting("value")
    .setOption("enabled")
    .build()
```

### Builder vs Abstract Factory

| Aspect | Builder | Abstract Factory |
|--------|---------|------------------|
| **Focus** | Construction process | Product families |
| **Result** | Single complex object | Set of related objects |
| **When** | Many construction steps | Need consistent families |
| **Flexibility** | Very flexible configuration | Less configuration, more selection |

```pseudocode
// Abstract Factory
uiFactory = new WindowsUIFactory()
button = uiFactory.createButton()
checkbox = uiFactory.createCheckbox()

// Builder
dialog = DialogBuilder.create()
    .addButton("OK")
    .addButton("Cancel")
    .addCheckbox("Remember me")
    .setTitle("Login")
    .build()
```

---

## Real-World Applications

### 1. Query Builders (SQL/Database)

```pseudocode
// SQL Query Builder
query = QueryBuilder.create()
    .select("users.name", "users.email", "orders.total")
    .from("users")
    .join("orders", "users.id = orders.user_id")
    .where("users.active", "=", true)
    .where("orders.total", ">", 100)
    .orderBy("orders.total", "DESC")
    .limit(10)
    .build()

// Generates: 
// SELECT users.name, users.email, orders.total
// FROM users
// JOIN orders ON users.id = orders.user_id
// WHERE users.active = true AND orders.total > 100
// ORDER BY orders.total DESC
// LIMIT 10
```

### 2. HTTP Request Builders

```pseudocode
// HTTP Request Builder
request = HttpRequestBuilder.create()
    .setMethod("POST")
    .setUrl("https://api.example.com/users")
    .addHeader("Content-Type", "application/json")
    .addHeader("Authorization", "Bearer " + token)
    .setBody({
        "name": "John Doe",
        "email": "john@example.com"
    })
    .setTimeout(30000)
    .setRetries(3)
    .build()

response = client.send(request)
```

### 3. UI Component Builders

```pseudocode
// Dialog Builder
dialog = DialogBuilder.create()
    .setTitle("Confirm Delete")
    .setMessage("Are you sure you want to delete this item?")
    .addButton("Delete", "destructive", onDelete)
    .addButton("Cancel", "default", onCancel)
    .setIcon("warning")
    .setCancelable(true)
    .build()

dialog.show()
```

### 4. Configuration Builders

```pseudocode
// Application Configuration Builder
config = AppConfigBuilder.create()
    .setEnvironment("production")
    .enableFeature("analytics")
    .enableFeature("notifications")
    .disableFeature("debug-mode")
    .setDatabaseUrl("postgres://localhost:5432/mydb")
    .setLogLevel("info")
    .setMaxConnections(100)
    .setTimeout(30)
    .build()

app = new Application(config)
```

### 5. Document Builders

```pseudocode
// PDF Document Builder
pdf = PdfDocumentBuilder.create()
    .setTitle("Monthly Report")
    .setAuthor("John Doe")
    .addPage()
        .setOrientation("portrait")
        .addHeading("Financial Summary", level: 1)
        .addParagraph("This report shows...")
        .addTable(financialData)
    .addPage()
        .setOrientation("landscape")
        .addChart(chartData, type: "bar")
    .setFooter("Page {page} of {total}")
    .build()

pdf.save("report.pdf")
```

### 6. Test Data Builders

```pseudocode
// Test User Builder
testUser = UserTestBuilder.create()
    .withValidEmail()
    .withValidPassword()
    .withRole("admin")
    .withVerifiedAccount()
    .withActiveSubscription()
    .build()

// Test Order Builder
testOrder = OrderTestBuilder.create()
    .withRandomItems(5)
    .withShippingAddress(testUser.address)
    .withPaymentMethod("credit-card")
    .withStatus("pending")
    .build()
```

### 7. Fluent Validation Builders

```pseudocode
// Validation Rules Builder
validator = ValidationBuilder.forType(User)
    .field("email")
        .isRequired()
        .isEmail()
        .maxLength(255)
    .field("password")
        .isRequired()
        .minLength(8)
        .mustContainUppercase()
        .mustContainNumber()
    .field("age")
        .isOptional()
        .min(18)
        .max(120)
    .build()

errors = validator.validate(user)
```

---

## Decision Framework

### When to Use Builder Pattern

```
Is object complex?
├─ Yes
│  ├─ Many parameters (>4)?
│  │  ├─ Yes → Use Builder ✅
│  │  └─ No → Consider simple constructor
│  │
│  ├─ Many optional parameters?
│  │  ├─ Yes → Use Builder ✅
│  │  └─ No → Consider simple constructor
│  │
│  ├─ Need immutability?
│  │  ├─ Yes → Use Builder ✅
│  │  └─ No → Consider simple constructor
│  │
│  ├─ Complex validation needed?
│  │  ├─ Yes → Use Builder ✅
│  │  └─ No → Consider simple constructor
│  │
│  └─ Multiple representations?
│     ├─ Yes → Use Builder ✅
│     └─ No → Consider simple constructor
│
└─ No (Simple object)
   └─ Use simple constructor ✅
```

---

## Conclusion

### Key Takeaways

1. **Purpose**: Builder Pattern separates complex object construction from its representation
2. **When to Use**: Objects with many parameters, optional fields, or validation needs
3. **Benefits**: Improved readability, flexibility, immutability support, and validation
4. **Trade-offs**: More code, additional classes, performance overhead
5. **Best Practices**: Return `this`, validate in build(), use descriptive names, provide defaults
6. **Avoid**: Over-engineering simple objects, mixing with setters, exposing builder state

### Quick Reference

**Use Builder Pattern When:**
- 4+ constructor parameters
- Many optional parameters
- Need for immutability
- Complex validation required
- Multiple object representations needed

**Avoid Builder Pattern When:**
- Simple objects (2-3 fields)
- All fields are required
- No validation needed
- Performance is critical
- Object needs to be mutable

---

## Further Reading

### Books
- *Design Patterns: Elements of Reusable Object-Oriented Software* by Gang of Four
- *Effective Java* by Joshua Bloch (Item 2: Consider a builder when faced with many constructor parameters)
- *Head First Design Patterns* by Freeman & Freeman

### Articles
- [Builder Pattern - Refactoring Guru](https://refactoring.guru/design-patterns/builder)
- [Builder Pattern - Wikipedia](https://en.wikipedia.org/wiki/Builder_pattern)
- [When to Use Builder Pattern](https://blogs.oracle.com/javamagazine/post/exploring-joshua-blochs-builder-design-pattern-in-java)

### Related Patterns
- Factory Method Pattern
- Abstract Factory Pattern
- Prototype Pattern
- Fluent Interface Pattern

---

**Document Version**: 1.0  
**Last Updated**: 2026-01-25  
**Status**: Active
