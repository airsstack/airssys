# Algebraic Data Types (ADT) Guidelines

> **Purpose**: This guideline provides comprehensive principles, patterns, and best practices for using Algebraic Data Types in software design and implementation across any programming language.

---

## Table of Contents

1. [Introduction](#introduction)
2. [What are Algebraic Data Types?](#what-are-algebraic-data-types)
3. [Product Types](#product-types)
4. [Sum Types](#sum-types)
5. [Pattern Matching](#pattern-matching)
6. [Type Algebra](#type-algebra)
7. [Benefits and Trade-offs](#benefits-and-trade-offs)
8. [Best Practices](#best-practices)
9. [Anti-Patterns to Avoid](#anti-patterns-to-avoid)
10. [Real-World Applications](#real-world-applications)
11. [Language-Specific Implementations](#language-specific-implementations)

---

## Introduction

### Definition

**Algebraic Data Types (ADTs)** are composite types formed by combining other types using algebraic operations. They are fundamental constructs in functional programming and type theory that enable precise modeling of data structures and business domains.

**Not to be confused with**: Abstract Data Types (also abbreviated as ADT), which refer to data structures defined by their operations and behavior rather than their implementation.

### Key Characteristics

- **Type Safety**: Encode business rules at the type level
- **Exhaustiveness**: Compiler-enforced handling of all cases
- **Composability**: Build complex types from simple ones
- **Immutability**: Typically immutable by default
- **Mathematical Foundation**: Based on algebraic operations (sum and product)

### Historical Context

- Introduced in the **Hope** programming language (1970s, University of Edinburgh)
- Core feature in functional languages (Haskell, ML, OCaml)
- Increasingly adopted in mainstream languages (Rust, Scala, TypeScript, Java, Kotlin)

---

## What are Algebraic Data Types?

### The Algebra of Types

Algebraic Data Types get their name from the algebraic operations used to combine types:

**1. Addition (Sum Types)**: `A + B` - A value is **either** type A **or** type B  
**2. Multiplication (Product Types)**: `A × B` - A value contains **both** type A **and** type B

### Why "Algebraic"?

The number of possible values in a composite type can be calculated using arithmetic:

```pseudocode
// Product Type: Point has x AND y
type Point = Integer × Integer

// If Integer has 2^32 possible values:
// Point has 2^32 × 2^32 = 2^64 possible values

// Sum Type: Direction is North OR South OR East OR West
type Direction = North | South | East | West

// Direction has 1 + 1 + 1 + 1 = 4 possible values
```

### The Two Fundamental Constructions

#### Sum Types (Tagged Unions / Variant Types)

A value must be **one of** a fixed set of alternatives.

```pseudocode
type PaymentMethod = 
    | CreditCard(cardNumber, cvv, expiryDate)
    | PayPal(email)
    | BankTransfer(accountNumber, routingNumber)
    | Cash
```

#### Product Types (Records / Tuples)

A value contains **all of** the specified fields.

```pseudocode
type Address = {
    street: String,
    city: String,
    postalCode: String,
    country: String
}
```

---

## Product Types

### Definition

A **Product Type** combines multiple values into a single composite value. All constituent values are present simultaneously.

### Mathematical Representation

```
A × B × C

If A has m values, B has n values, C has p values,
then A × B × C has m × n × p values.
```

### Common Forms

#### 1. Records (Named Product Types)

```pseudocode
type Person = {
    firstName: String,
    lastName: String,
    age: Integer,
    email: String
}

person = Person {
    firstName: "Alice",
    lastName: "Smith",
    age: 30,
    email: "alice@example.com"
}
```

#### 2. Tuples (Anonymous Product Types)

```pseudocode
type Coordinate = (Float, Float)
type RGB = (Integer, Integer, Integer)

point = (10.5, 20.3)
color = (255, 128, 0)
```

#### 3. Structs (Low-Level Product Types)

```pseudocode
struct Point3D {
    x: Float
    y: Float
    z: Float
}
```

### Nested Product Types

Products can contain other products:

```pseudocode
type Employee = {
    id: Integer,
    name: String,
    address: Address,  // Product type inside product type
    salary: Money
}

type Address = {
    street: String,
    city: String,
    country: String
}

type Money = {
    amount: Decimal,
    currency: String
}
```

### Counting Values in Product Types

```pseudocode
// Example: Two booleans
type Flags = {
    isActive: Boolean,    // 2 values: true, false
    isVerified: Boolean   // 2 values: true, false
}

// Total possible values = 2 × 2 = 4:
// (true, true), (true, false), (false, true), (false, false)

// Example: Boolean and Digit
type BoolAndDigit = {
    flag: Boolean,    // 2 values
    digit: Digit      // 10 values (0-9)
}

// Total possible values = 2 × 10 = 20
```

---

## Sum Types

### Definition

A **Sum Type** (also called **Tagged Union**, **Variant Type**, or **Discriminated Union**) represents a value that can be one of several alternatives. Each alternative is tagged with a unique identifier (constructor).

### Mathematical Representation

```
A + B + C

If A has m values, B has n values, C has p values,
then A + B + C has m + n + p values.
```

### Basic Sum Types

#### 1. Enumerations (Sum Types Without Data)

```pseudocode
type TrafficLight = Red | Yellow | Green

type DayOfWeek = 
    | Monday 
    | Tuesday 
    | Wednesday 
    | Thursday 
    | Friday 
    | Saturday 
    | Sunday
```

#### 2. Sum Types With Data

```pseudocode
type Shape =
    | Circle(radius: Float)
    | Rectangle(width: Float, height: Float)
    | Triangle(base: Float, height: Float)

circle = Circle(5.0)
rectangle = Rectangle(10.0, 20.0)
triangle = Triangle(8.0, 6.0)
```

### Classic Examples

#### 1. Option/Maybe Type

Represents a value that might be absent:

```pseudocode
type Option<T> = 
    | Some(value: T)
    | None

// Safe representation of nullable values
findUser(id: Integer): Option<User> {
    user = database.query(id)
    if (user exists) {
        return Some(user)
    } else {
        return None
    }
}

// Usage
result = findUser(42)
match result {
    Some(user) => print("Found: " + user.name)
    None => print("User not found")
}
```

#### 2. Either/Result Type

Represents success or failure:

```pseudocode
type Result<T, E> = 
    | Ok(value: T)
    | Err(error: E)

// Error handling without exceptions
divide(a: Float, b: Float): Result<Float, String> {
    if (b == 0) {
        return Err("Division by zero")
    } else {
        return Ok(a / b)
    }
}

// Usage
result = divide(10.0, 2.0)
match result {
    Ok(value) => print("Result: " + value)
    Err(error) => print("Error: " + error)
}
```

#### 3. Tree Structures

```pseudocode
type BinaryTree<T> =
    | Empty
    | Node(value: T, left: BinaryTree<T>, right: BinaryTree<T>)

// Example tree
tree = Node(
    value: 5,
    left: Node(
        value: 3,
        left: Empty,
        right: Empty
    ),
    right: Node(
        value: 7,
        left: Empty,
        right: Empty
    )
)
```

### Counting Values in Sum Types

```pseudocode
// Boolean type is a sum type
type Boolean = True | False
// Total values = 1 + 1 = 2

// Boolean or Digit
type BoolOrDigit = 
    | BoolValue(Boolean)
    | DigitValue(Digit)

// Total values = 2 + 10 = 12

// Option type
type Option<T> = Some(T) | None
// If T has n values, Option<T> has n + 1 values
```

---

## Pattern Matching

### Definition

**Pattern Matching** is a mechanism for checking a value against a pattern and deconstructing data. It's the primary way to work with algebraic data types, especially sum types.

### Basic Pattern Matching

```pseudocode
function describe(shape: Shape): String {
    match shape {
        Circle(radius) => 
            "Circle with radius " + radius
        
        Rectangle(width, height) => 
            "Rectangle " + width + " × " + height
        
        Triangle(base, height) => 
            "Triangle with base " + base
    }
}
```

### Exhaustiveness Checking

The compiler ensures all cases are handled:

```pseudocode
// ❌ Compile error: Missing pattern for Triangle
function area(shape: Shape): Float {
    match shape {
        Circle(r) => 3.14159 * r * r
        Rectangle(w, h) => w * h
        // ERROR: Non-exhaustive pattern match!
    }
}

// ✅ All cases handled
function area(shape: Shape): Float {
    match shape {
        Circle(r) => 3.14159 * r * r
        Rectangle(w, h) => w * h
        Triangle(b, h) => 0.5 * b * h
    }
}
```

### Nested Pattern Matching

```pseudocode
type Expression =
    | Number(value: Integer)
    | Add(left: Expression, right: Expression)
    | Multiply(left: Expression, right: Expression)

function evaluate(expr: Expression): Integer {
    match expr {
        Number(n) => n
        
        Add(left, right) => 
            evaluate(left) + evaluate(right)
        
        Multiply(left, right) => 
            evaluate(left) * evaluate(right)
    }
}

// Nested pattern matching
function simplify(expr: Expression): Expression {
    match expr {
        Add(Number(0), right) => right
        Add(left, Number(0)) => left
        Multiply(Number(1), right) => right
        Multiply(left, Number(1)) => left
        Multiply(Number(0), _) => Number(0)
        Multiply(_, Number(0)) => Number(0)
        _ => expr
    }
}
```

### Guards and Conditions

```pseudocode
function categorize(age: Integer): String {
    match age {
        n if n < 0 => "Invalid age"
        n if n < 13 => "Child"
        n if n < 20 => "Teenager"
        n if n < 65 => "Adult"
        _ => "Senior"
    }
}
```

### Wildcard Patterns

```pseudocode
// Underscore (_) matches anything and discards the value
function isCircle(shape: Shape): Boolean {
    match shape {
        Circle(_) => true
        _ => false
    }
}

// Partial deconstruction
function hasRadius(shape: Shape): Boolean {
    match shape {
        Circle(r) if r > 0 => true
        _ => false
    }
}
```

---

## Type Algebra

### Algebraic Laws

Algebraic Data Types follow mathematical laws:

#### 1. Addition is Commutative

```pseudocode
A + B ≡ B + A

type Result1 = Left(A) | Right(B)
type Result2 = Right(B) | Left(A)
// Semantically equivalent (though constructors differ)
```

#### 2. Multiplication is Commutative

```pseudocode
A × B ≡ B × A

type Pair1 = (A, B)
type Pair2 = (B, A)
// Same number of possible values
```

#### 3. Multiplication Distributes Over Addition

```pseudocode
A × (B + C) ≡ (A × B) + (A × C)

type Left = A × (B | C)
type Right = (A × B) | (A × C)
// Isomorphic (can convert between them without loss)
```

#### 4. Identity Elements

```pseudocode
// Addition identity: 0 (void/never type)
A + 0 ≡ A

type Void  // Type with no values

type WithVoid<A> = A | Void
// Equivalent to just A

// Multiplication identity: 1 (unit type)
A × 1 ≡ A

type Unit = ()  // Type with exactly one value

type WithUnit<A> = (A, Unit)
// Equivalent to just A
```

### Recursive Types

Algebraic Data Types can be defined recursively:

```pseudocode
// Linked List
type List<T> = 
    | Empty
    | Cons(head: T, tail: List<T>)

// μ (mu) represents recursive type
// List<T> ≡ 1 + T × List<T>
// List<T> ≡ μX. 1 + T × X

// Natural numbers (Peano arithmetic)
type Nat = 
    | Zero
    | Succ(Nat)

// Nat ≡ μX. 1 + X
// Nat has infinite values: Zero, Succ(Zero), Succ(Succ(Zero)), ...
```

### Type Isomorphisms

Two types are isomorphic if you can convert between them without losing information:

```pseudocode
// A × B ≅ B × A
function swap<A, B>(pair: (A, B)): (B, A) {
    match pair {
        (a, b) => (b, a)
    }
}

// A + (B + C) ≅ (A + B) + C
type Nested = A | (B | C)
type Flat = (A | B) | C

function flatten(nested: Nested): Flat {
    match nested {
        A(a) => Left(Left(a))
        B(b) => Left(Right(b))
        C(c) => Right(c)
    }
}
```

---

## Benefits and Trade-offs

### Benefits

#### 1. Make Illegal States Unrepresentable

```pseudocode
// ❌ BAD: Using booleans and nulls
class Order {
    isDelivered: Boolean
    deliveryDate: Date?  // Nullable
    trackingNumber: String?  // Nullable
}

// Problems:
// - isDelivered=true but deliveryDate=null (inconsistent)
// - isDelivered=false but trackingNumber exists (impossible state)

// ✅ GOOD: Using sum types
type Order =
    | Pending(orderDate: Date)
    | Shipped(orderDate: Date, trackingNumber: String)
    | Delivered(orderDate: Date, trackingNumber: String, deliveryDate: Date)

// Impossible to create invalid states
```

#### 2. Exhaustive Pattern Matching

```pseudocode
// Compiler forces you to handle all cases
function processPayment(method: PaymentMethod): Result {
    match method {
        CreditCard(num, cvv, exp) => processCreditCard(num, cvv, exp)
        PayPal(email) => processPayPal(email)
        BankTransfer(acc, routing) => processBankTransfer(acc, routing)
        Cash => processCash()
        // If we add a new payment method, compiler will error here
    }
}
```

#### 3. Self-Documenting Code

```pseudocode
// Type tells you exactly what's possible
type LoadingState<T> =
    | NotStarted
    | Loading
    | Success(data: T)
    | Failure(error: String)

// No need to guess what states are valid
```

#### 4. Refactoring Safety

```pseudocode
// Change type definition
type Shape =
    | Circle(radius: Float)
    | Rectangle(width: Float, height: Float)
    | Triangle(base: Float, height: Float)
    | Ellipse(majorAxis: Float, minorAxis: Float)  // NEW

// Compiler will find every place that needs updating
// Error: Non-exhaustive pattern match in function area()
// Error: Non-exhaustive pattern match in function perimeter()
```

#### 5. Precise Domain Modeling

```pseudocode
// Model business rules exactly
type EmailValidation =
    | Unvalidated(rawEmail: String)
    | Validated(email: EmailAddress)
    | Rejected(rawEmail: String, reason: String)

// Prevents using unvalidated emails
function sendEmail(to: Validated): Result {
    match to {
        Validated(email) => send(email)
        // Compiler prevents Unvalidated or Rejected
    }
}
```

### Trade-offs

#### 1. Language Support Variability

Not all languages have native support for ADTs:

```pseudocode
// Languages with strong ADT support:
// - Haskell, OCaml, F#, Rust, Elm, PureScript

// Languages with partial support:
// - TypeScript, Kotlin, Swift, Scala

// Languages requiring workarounds:
// - Java (sealed classes + records since Java 17)
// - Python (using classes or enums)
// - JavaScript (using objects/classes)
```

#### 2. Verbosity in Some Languages

```pseudocode
// Haskell (concise)
data Maybe a = Nothing | Just a

// Java (verbose)
sealed interface Maybe<T> {
    record Nothing<T>() implements Maybe<T> {}
    record Just<T>(T value) implements Maybe<T> {}
}
```

#### 3. Learning Curve

Requires understanding:
- Type theory concepts
- Pattern matching
- Functional programming principles

#### 4. Performance Considerations

```pseudocode
// Sum types may have memory overhead for tags
type Result<T, E> = Ok(T) | Err(E)
// Stores tag + larger of T or E (plus padding)

// More memory than just T
```

---

## Best Practices

### 1. Model Business Domain with Sum Types

```pseudocode
// ✅ GOOD: Precise domain model
type UserAccount =
    | Active(user: User, subscription: Subscription)
    | Suspended(user: User, reason: String, until: Date)
    | Closed(user: User, closedDate: Date)

// ❌ BAD: Boolean flags
class UserAccount {
    user: User
    isActive: Boolean
    isSuspended: Boolean
    isClosed: Boolean
    suspensionReason: String?
    suspensionEndDate: Date?
    closedDate: Date?
}
```

### 2. Use Option/Maybe Instead of Null

```pseudocode
// ✅ GOOD: Explicit optionality
function findUserById(id: Integer): Option<User> {
    user = database.query(id)
    return user ? Some(user) : None
}

// ❌ BAD: Null reference
function findUserById(id: Integer): User? {
    return database.query(id)  // May return null
}
```

### 3. Use Result/Either for Error Handling

```pseudocode
// ✅ GOOD: Explicit error types
function parseJson(input: String): Result<JsonValue, ParseError> {
    try {
        value = parse(input)
        return Ok(value)
    } catch (error) {
        return Err(ParseError(error.message))
    }
}

// ❌ BAD: Exceptions
function parseJson(input: String): JsonValue {
    return parse(input)  // May throw exception
}
```

### 4. Make Variants Constructors Clear

```pseudocode
// ✅ GOOD: Clear constructor names
type ApiResponse<T> =
    | Success(data: T, timestamp: DateTime)
    | NotFound(message: String)
    | Unauthorized(message: String)
    | ServerError(message: String, code: Integer)

// ❌ BAD: Unclear names
type ApiResponse<T> =
    | Type1(T, DateTime)
    | Type2(String)
    | Type3(String)
    | Type4(String, Integer)
```

### 5. Keep Variants Small and Focused

```pseudocode
// ✅ GOOD: Focused variants
type PaymentResult =
    | Approved(transactionId: String)
    | Declined(reason: DeclineReason)
    | PendingReview(reviewId: String)

type DeclineReason =
    | InsufficientFunds
    | ExpiredCard
    | InvalidCvv
    | SuspectedFraud

// ❌ BAD: Kitchen sink variant
type PaymentResult =
    | Success(transactionId, amount, currency, timestamp, fee, ...)
    | Failure(reason, code, message, timestamp, attemptNumber, ...)
```

### 6. Use Pattern Matching Over Conditionals

```pseudocode
// ✅ GOOD: Pattern matching
function handleResponse(response: ApiResponse<Data>): String {
    match response {
        Success(data, _) => "Loaded: " + data.title
        NotFound(msg) => "Not found: " + msg
        Unauthorized(msg) => "Unauthorized: " + msg
        ServerError(msg, code) => "Error " + code + ": " + msg
    }
}

// ❌ BAD: Conditional chains
function handleResponse(response: ApiResponse<Data>): String {
    if (response.isSuccess()) {
        return "Loaded: " + response.getData().title
    } else if (response.isNotFound()) {
        return "Not found: " + response.getMessage()
    } else if (response.isUnauthorized()) {
        return "Unauthorized: " + response.getMessage()
    } else if (response.isServerError()) {
        return "Error " + response.getCode() + ": " + response.getMessage()
    }
}
```

### 7. Leverage Exhaustiveness Checking

```pseudocode
// ✅ GOOD: Let compiler verify completeness
function render(state: LoadingState<Data>): View {
    match state {
        NotStarted => renderPlaceholder()
        Loading => renderSpinner()
        Success(data) => renderData(data)
        Failure(error) => renderError(error)
    }
    // Compiler ensures all cases handled
}

// ❌ BAD: Default case hides missing patterns
function render(state: LoadingState<Data>): View {
    match state {
        Success(data) => renderData(data)
        _ => renderPlaceholder()
    }
    // Loading and Failure treated the same (probably wrong)
}
```

### 8. Encode State Machines as Sum Types

```pseudocode
// ✅ GOOD: Explicit state machine
type ConnectionState =
    | Disconnected
    | Connecting(startTime: DateTime)
    | Connected(socket: Socket, connectedAt: DateTime)
    | Reconnecting(socket: Socket, attempt: Integer)
    | Failed(error: String)

function transition(
    state: ConnectionState, 
    event: Event
): ConnectionState {
    match (state, event) {
        (Disconnected, Connect) => 
            Connecting(currentTime())
        
        (Connecting(_), Success(socket)) => 
            Connected(socket, currentTime())
        
        (Connecting(_), Failure(error)) => 
            Failed(error)
        
        (Connected(socket, _), Disconnect) => 
            Disconnected
        
        // Invalid transitions caught by compiler
        _ => state
    }
}
```

---

## Anti-Patterns to Avoid

### 1. ❌ Using Booleans Instead of Sum Types

```pseudocode
// ❌ BAD: Boolean hell
class Task {
    isComplete: Boolean
    isCancelled: Boolean
    isInProgress: Boolean
    isFailed: Boolean
}

// What if isComplete=true AND isCancelled=true?
// What if all are false?

// ✅ GOOD: Sum type
type TaskStatus =
    | Pending
    | InProgress(startedAt: DateTime)
    | Completed(finishedAt: DateTime)
    | Cancelled(reason: String)
    | Failed(error: String)
```

### 2. ❌ Nullable Fields Instead of Option Types

```pseudocode
// ❌ BAD: Null values
class User {
    name: String
    email: String
    phoneNumber: String?  // May be null
    address: String?      // May be null
}

// Null checks scattered everywhere
if (user.phoneNumber != null) {
    sendSms(user.phoneNumber)
}

// ✅ GOOD: Option type
class User {
    name: String
    email: String
    phoneNumber: Option<PhoneNumber>
    address: Option<Address>
}

// Explicit handling
match user.phoneNumber {
    Some(phone) => sendSms(phone)
    None => print("No phone number")
}
```

### 3. ❌ Using Strings for Enumerations

```pseudocode
// ❌ BAD: Stringly-typed
function processStatus(status: String) {
    if (status == "pending") {
        // ...
    } else if (status == "approved") {
        // ...
    } else if (status == "rejected") {
        // ...
    }
    // Typos not caught: "aproved", "Pending", etc.
}

// ✅ GOOD: Sum type
type Status = Pending | Approved | Rejected

function processStatus(status: Status) {
    match status {
        Pending => // ...
        Approved => // ...
        Rejected => // ...
    }
}
```

### 4. ❌ Overly Generic Types

```pseudocode
// ❌ BAD: Too generic
type Response<T> = 
    | Success(T)
    | Error(String)

// Everything is a string error?

// ✅ GOOD: Specific error types
type LoginResponse =
    | Success(user: User, token: Token)
    | InvalidCredentials
    | AccountLocked(until: DateTime)
    | NetworkError(message: String)
```

### 5. ❌ Mixing Concerns in Variants

```pseudocode
// ❌ BAD: Mixed concerns
type UserAction =
    | Login(email, password)
    | Logout
    | UpdateProfile(data)
    | FetchData(url)  // Data fetching not user action
    | LogError(error)  // Logging not user action

// ✅ GOOD: Focused types
type UserAction =
    | Login(email, password)
    | Logout
    | UpdateProfile(data)

type SystemEvent =
    | FetchData(url)
    | LogError(error)
```

### 6. ❌ Not Using Exhaustiveness

```pseudocode
// ❌ BAD: Wildcard hides missing cases
function handle(event: Event) {
    match event {
        Click(x, y) => handleClick(x, y)
        _ => print("Ignored")
    }
    // KeyPress, Scroll, etc. all ignored silently
}

// ✅ GOOD: Explicit handling
function handle(event: Event) {
    match event {
        Click(x, y) => handleClick(x, y)
        KeyPress(key) => handleKeyPress(key)
        Scroll(delta) => handleScroll(delta)
        MouseMove(x, y) => handleMouseMove(x, y)
    }
}
```

---

## Real-World Applications

### 1. UI Component State Management

```pseudocode
type ComponentState<T> =
    | Idle
    | Loading
    | Loaded(data: T)
    | Refreshing(data: T)
    | Error(message: String)

function render(state: ComponentState<UserList>): View {
    match state {
        Idle => 
            <EmptyState />
        
        Loading => 
            <Spinner />
        
        Loaded(users) => 
            <UserList users={users} />
        
        Refreshing(users) => 
            <UserList users={users} showRefreshIndicator={true} />
        
        Error(msg) => 
            <ErrorView message={msg} retry={loadData} />
    }
}
```

### 2. API Response Modeling

```pseudocode
type HttpResponse<T> =
    | Success200(body: T, headers: Headers)
    | NoContent204
    | BadRequest400(errors: ValidationErrors)
    | Unauthorized401
    | Forbidden403
    | NotFound404
    | ServerError500(message: String)
    | NetworkError(error: NetworkException)

function handleUserFetch(response: HttpResponse<User>): Effect {
    match response {
        Success200(user, _) => 
            showUserProfile(user)
        
        NotFound404 => 
            showMessage("User not found")
        
        Unauthorized401 => 
            redirectToLogin()
        
        ServerError500(msg) => 
            showError("Server error: " + msg)
        
        NetworkError(err) => 
            showError("Network error: " + err.message)
        
        _ => 
            showError("Unexpected response")
    }
}
```

### 3. Form Validation

```pseudocode
type ValidationResult<T> =
    | Valid(value: T)
    | Invalid(errors: List<ValidationError>)

type ValidationError = {
    field: String,
    message: String
}

function validateRegistration(form: RegistrationForm): ValidationResult<ValidatedForm> {
    errors = []
    
    if (form.email.isEmpty()) {
        errors.add(ValidationError("email", "Email is required"))
    } else if (!form.email.isValidEmail()) {
        errors.add(ValidationError("email", "Invalid email format"))
    }
    
    if (form.password.length < 8) {
        errors.add(ValidationError("password", "Password must be at least 8 characters"))
    }
    
    if (errors.isEmpty()) {
        return Valid(ValidatedForm(form.email, form.password))
    } else {
        return Invalid(errors)
    }
}
```

### 4. Domain-Driven Design

```pseudocode
// Order lifecycle
type Order =
    | Draft(items: List<Item>, createdAt: DateTime)
    | Submitted(items: List<Item>, submittedAt: DateTime, orderId: OrderId)
    | Confirmed(items: List<Item>, orderId: OrderId, confirmedAt: DateTime)
    | Shipped(orderId: OrderId, trackingNumber: String, shippedAt: DateTime)
    | Delivered(orderId: OrderId, deliveredAt: DateTime)
    | Cancelled(orderId: OrderId, reason: String, cancelledAt: DateTime)

// Business rules encoded in types
function ship(order: Order): Result<Order, ShippingError> {
    match order {
        Confirmed(items, orderId, _) =>
            // Can only ship confirmed orders
            tracking = generateTrackingNumber()
            Ok(Shipped(orderId, tracking, currentTime()))
        
        _ =>
            Err(ShippingError("Can only ship confirmed orders"))
    }
}
```

### 5. Expression Evaluator

```pseudocode
type Expression =
    | Number(value: Integer)
    | Variable(name: String)
    | Add(left: Expression, right: Expression)
    | Subtract(left: Expression, right: Expression)
    | Multiply(left: Expression, right: Expression)
    | Divide(left: Expression, right: Expression)

function evaluate(expr: Expression, vars: Map<String, Integer>): Result<Integer, EvalError> {
    match expr {
        Number(n) => 
            Ok(n)
        
        Variable(name) =>
            vars.get(name)
                .map(Ok)
                .getOrElse(Err(UndefinedVariable(name)))
        
        Add(left, right) => {
            leftVal = evaluate(left, vars)?
            rightVal = evaluate(right, vars)?
            Ok(leftVal + rightVal)
        }
        
        Divide(left, right) => {
            leftVal = evaluate(left, vars)?
            rightVal = evaluate(right, vars)?
            
            if (rightVal == 0) {
                Err(DivisionByZero)
            } else {
                Ok(leftVal / rightVal)
            }
        }
        
        // ... other operations
    }
}
```

### 6. File System Representation

```pseudocode
type FileSystemNode =
    | File(name: String, size: Integer, content: Bytes)
    | Directory(name: String, children: List<FileSystemNode>)
    | SymLink(name: String, target: Path)

function totalSize(node: FileSystemNode): Integer {
    match node {
        File(_, size, _) => 
            size
        
        Directory(_, children) => 
            children.map(totalSize).sum()
        
        SymLink(_, target) => 
            // Follow symlink and calculate
            resolveAndCalculate(target)
    }
}
```

### 7. Command Pattern with ADTs

```pseudocode
type Command =
    | CreateUser(name: String, email: String)
    | DeleteUser(userId: UserId)
    | UpdateUserEmail(userId: UserId, newEmail: String)
    | SendNotification(userId: UserId, message: String)

function execute(command: Command): Result<Success, CommandError> {
    match command {
        CreateUser(name, email) =>
            userRepository.create(name, email)
        
        DeleteUser(userId) =>
            userRepository.delete(userId)
        
        UpdateUserEmail(userId, newEmail) =>
            userRepository.updateEmail(userId, newEmail)
        
        SendNotification(userId, message) =>
            notificationService.send(userId, message)
    }
}
```

### 8. Parser Combinators

```pseudocode
type ParseResult<T> =
    | Success(value: T, remaining: String)
    | Failure(error: String, position: Integer)

type Parser<T> = function(input: String): ParseResult<T>

function parseNumber(input: String): ParseResult<Integer> {
    digits = input.takeWhile(isDigit)
    
    if (digits.isEmpty()) {
        return Failure("Expected number", 0)
    }
    
    value = parseInt(digits)
    remaining = input.drop(digits.length)
    
    return Success(value, remaining)
}
```

---

## Language-Specific Implementations

### Haskell

```haskell
-- Sum type
data Shape = Circle Float
           | Rectangle Float Float
           | Triangle Float Float Float

-- Pattern matching
area :: Shape -> Float
area (Circle r) = pi * r * r
area (Rectangle w h) = w * h
area (Triangle b h) = 0.5 * b * h

-- Polymorphic ADT
data Maybe a = Nothing | Just a

-- Recursive ADT
data List a = Nil | Cons a (List a)
```

### Rust

```rust
// Sum type (enum)
enum Shape {
    Circle { radius: f64 },
    Rectangle { width: f64, height: f64 },
    Triangle { base: f64, height: f64 },
}

// Pattern matching
fn area(shape: &Shape) -> f64 {
    match shape {
        Shape::Circle { radius } => 
            std::f64::consts::PI * radius * radius,
        Shape::Rectangle { width, height } => 
            width * height,
        Shape::Triangle { base, height } => 
            0.5 * base * height,
    }
}

// Option type (built-in)
enum Option<T> {
    None,
    Some(T),
}

// Result type (built-in)
enum Result<T, E> {
    Ok(T),
    Err(E),
}
```

### TypeScript

```typescript
// Sum type (discriminated union)
type Shape =
    | { kind: 'circle'; radius: number }
    | { kind: 'rectangle'; width: number; height: number }
    | { kind: 'triangle'; base: number; height: number };

// Pattern matching (using discriminant)
function area(shape: Shape): number {
    switch (shape.kind) {
        case 'circle':
            return Math.PI * shape.radius * shape.radius;
        case 'rectangle':
            return shape.width * shape.height;
        case 'triangle':
            return 0.5 * shape.base * shape.height;
    }
}

// Generic sum type
type Option<T> = 
    | { kind: 'none' }
    | { kind: 'some'; value: T };

type Result<T, E> =
    | { kind: 'ok'; value: T }
    | { kind: 'error'; error: E };
```

### Scala

```scala
// Sum type (sealed trait)
sealed trait Shape
case class Circle(radius: Double) extends Shape
case class Rectangle(width: Double, height: Double) extends Shape
case class Triangle(base: Double, height: Double) extends Shape

// Pattern matching
def area(shape: Shape): Double = shape match {
  case Circle(r) => math.Pi * r * r
  case Rectangle(w, h) => w * h
  case Triangle(b, h) => 0.5 * b * h
}

// Option type (built-in)
sealed trait Option[+A]
case class Some[+A](value: A) extends Option[A]
case object None extends Option[Nothing]

// Either type (built-in)
sealed trait Either[+A, +B]
case class Left[+A](value: A) extends Either[A, Nothing]
case class Right[+B](value: B) extends Either[Nothing, B]
```

### Kotlin

```kotlin
// Sum type (sealed class)
sealed class Shape {
    data class Circle(val radius: Double) : Shape()
    data class Rectangle(val width: Double, val height: Double) : Shape()
    data class Triangle(val base: Double, val height: Double) : Shape()
}

// Pattern matching (when expression)
fun area(shape: Shape): Double = when (shape) {
    is Shape.Circle -> Math.PI * shape.radius * shape.radius
    is Shape.Rectangle -> shape.width * shape.height
    is Shape.Triangle -> 0.5 * shape.base * shape.height
}

// Result type (built-in)
sealed class Result<out T, out E> {
    data class Ok<T>(val value: T) : Result<T, Nothing>()
    data class Err<E>(val error: E) : Result<Nothing, E>()
}
```

### Java (17+)

```java
// Sum type (sealed interface + records)
sealed interface Shape 
    permits Circle, Rectangle, Triangle {
}

record Circle(double radius) implements Shape {}
record Rectangle(double width, double height) implements Shape {}
record Triangle(double base, double height) implements Shape {}

// Pattern matching (switch expression)
static double area(Shape shape) {
    return switch (shape) {
        case Circle(var r) -> 
            Math.PI * r * r;
        case Rectangle(var w, var h) -> 
            w * h;
        case Triangle(var b, var h) -> 
            0.5 * b * h;
    };
}

// Optional type (built-in)
Optional<String> maybeValue = Optional.of("hello");
Optional<String> empty = Optional.empty();

// Custom Result type
sealed interface Result<T, E> 
    permits Result.Ok, Result.Err {
    
    record Ok<T, E>(T value) implements Result<T, E> {}
    record Err<T, E>(E error) implements Result<T, E> {}
}
```

### Swift

```swift
// Sum type (enum)
enum Shape {
    case circle(radius: Double)
    case rectangle(width: Double, height: Double)
    case triangle(base: Double, height: Double)
}

// Pattern matching
func area(shape: Shape) -> Double {
    switch shape {
    case .circle(let r):
        return Double.pi * r * r
    case .rectangle(let w, let h):
        return w * h
    case .triangle(let b, let h):
        return 0.5 * b * h
    }
}

// Optional type (built-in)
enum Optional<Wrapped> {
    case none
    case some(Wrapped)
}

// Result type (built-in)
enum Result<Success, Failure: Error> {
    case success(Success)
    case failure(Failure)
}
```

### F#

```fsharp
// Sum type (discriminated union)
type Shape =
    | Circle of radius: float
    | Rectangle of width: float * height: float
    | Triangle of base: float * height: float

// Pattern matching
let area shape =
    match shape with
    | Circle r -> System.Math.PI * r * r
    | Rectangle (w, h) -> w * h
    | Triangle (b, h) -> 0.5 * b * h

// Option type (built-in)
type Option<'T> =
    | Some of 'T
    | None

// Result type (built-in)
type Result<'T, 'TError> =
    | Ok of 'T
    | Error of 'TError
```

---

## Conclusion

### Key Takeaways

1. **Algebraic Data Types** combine types using algebraic operations (sum and product)
2. **Product Types** (records/tuples) contain multiple values simultaneously
3. **Sum Types** (tagged unions) represent one of several alternatives
4. **Pattern Matching** provides safe and exhaustive handling of ADTs
5. **Type Algebra** follows mathematical laws (commutative, associative, distributive)
6. **Make Illegal States Unrepresentable** - encode business rules in types
7. **Language Support** varies, but many modern languages now support ADTs

### When to Use ADTs

✅ **Use ADTs When:**
- Modeling domain states with mutual exclusivity
- Replacing boolean flags and null values
- Implementing type-safe error handling
- Building parsers, compilers, or interpreters
- Creating complex state machines
- Encoding business rules at the type level

❌ **Avoid ADTs When:**
- Language lacks proper support (use workarounds instead)
- Working with dynamic, unpredictable data shapes
- Performance is absolutely critical (measure first)
- Team lacks functional programming experience (provide training)

### Evolution Path

```
1. Start with: Replace nulls with Option/Maybe types
2. Then: Replace exceptions with Result/Either types
3. Next: Model domain states with custom sum types
4. Finally: Encode entire business domain with ADTs
```

---

## Further Reading

### Books
- *Types and Programming Languages* by Benjamin C. Pierce
- *Programming in Haskell* by Graham Hutton
- *Domain Modeling Made Functional* by Scott Wlaschin
- *Functional Programming in Scala* by Paul Chiusano & Rúnar Bjarnason

### Articles
- [Algebraic Data Types - Wikipedia](https://en.wikipedia.org/wiki/Algebraic_data_type)
- [Making Illegal States Unrepresentable](https://fsharpforfunandprofit.com/posts/designing-with-types-making-illegal-states-unrepresentable/)
- [Parse, Don't Validate](https://lexi-lambda.github.io/blog/2019/11/05/parse-don-t-validate/)

### Academic Papers
- *A History of Haskell: Being Lazy With Class* (Hudak et al., 2007)
- *Algebraic Data Types* (Hope Language, 1980)

### Related Concepts
- Abstract Data Types (ADT - different meaning)
- Type Theory
- Category Theory
- Dependent Types
- Generalized Algebraic Data Types (GADTs)

---

**Document Version**: 1.0  
**Last Updated**: 2026-01-25  
**Status**: Active  
**Applicable To**: All programming languages (with language-specific adaptations)
