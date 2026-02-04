# Rust Documentation Guide

This guide establishes standards for writing documentation in Rust codebases, covering doc comments, doctests, and API documentation.

## Core Principles

1. **Completeness**: All public API items must be documented
2. **Clarity**: Write for both experts and novices
3. **Accuracy**: Keep documentation synchronized with code
4. **Testability**: Use doctests to ensure examples remain valid

## Documentation Enforcement

Enable lint enforcement at the crate level:

```rust
// lib.rs or main.rs
#![warn(missing_docs)]           // Warn on missing docs
#![deny(rustdoc::broken_intra_doc_links)]  // Error on broken links
```

For stricter enforcement:

```rust
#![deny(missing_docs)]           // Error on missing docs
#![deny(missing_crate_level_docs)]  // Error on missing crate docs
```

## Comment Syntax

### Item Documentation (`///`)

Use triple slashes for items that follow:

```rust
/// A brief description of the function.
///
/// More detailed explanation of what this function does,
/// its behavior, and any important details.
pub fn my_function() {}
```

### Module/Crate Documentation (`//!`)

Use `//!` at the top of a file for module or crate documentation:

```rust
//! # My Crate
//!
//! A brief description of what this crate provides.
//!
//! ## Getting Started
//!
//! ```rust
//! use my_crate::do_something;
//! do_something();
//! ```
```

## Documentation Structure

### Standard Structure for All Items

```markdown
[Brief one-line summary]

[Detailed description - optional but recommended]

# Examples

[Working code examples]

# Errors

[Error conditions and types - for Result-returning functions]

# Panics

[Panic conditions - if function can panic]

# Safety

[Safety requirements - for unsafe functions]
```

### Crate-Level Documentation

Place in `lib.rs` or `main.rs`:

```rust
//! # Crate Name
//!
//! Brief description of what this crate does.
//!
//! ## Features
//!
//! - Feature 1: description
//! - Feature 2: description
//!
//! ## Getting Started
//!
//! ```rust
//! use my_crate::Config;
//!
//! let config = Config::new();
//! ```
//!
//! ## Examples
//!
//! [More detailed examples]
```

### Module Documentation

```rust
//! Authentication module.
//!
//! This module provides types and functions for user authentication,
//! including token generation and verification.
//!
//! ## Types
//!
//! - [`Token`]: Represents an authentication token
//! - [`Claims`]: JWT claims structure
//!
//! ## Example
//!
//! ```rust
//! use my_crate::auth::{Token, Claims};
//!
//! let token = Token::new(Claims::default());
//! ```
```

### Function Documentation

```rust
/// Validates user credentials and returns an authentication token.
///
/// This function checks the provided username and password against
/// the configured authentication backend.
///
/// # Examples
///
/// ```rust
/// use my_crate::auth::authenticate;
///
/// let result = authenticate("user", "password");
/// assert!(result.is_ok());
/// ```
///
/// # Errors
///
/// Returns [`AuthError::InvalidCredentials`] if the username/password
/// combination is invalid.
///
/// Returns [`AuthError::ConnectionFailed`] if the auth backend is
/// unreachable.
///
/// # Panics
///
/// Panics if the auth backend is not configured.
pub fn authenticate(username: &str, password: &str) -> Result<Token, AuthError> {
    // ...
}
```

### Struct Documentation

```rust
/// Configuration for the HTTP client.
///
/// This struct holds all settings needed to establish connections
/// to remote servers.
///
/// # Examples
///
/// ```rust
/// use my_crate::HttpConfig;
///
/// let config = HttpConfig::builder()
///     .timeout(Duration::from_secs(30))
///     .build();
/// ```
#[derive(Debug, Clone)]
pub struct HttpConfig {
    /// Connection timeout in seconds.
    pub timeout: Duration,

    /// Maximum number of retry attempts.
    pub max_retries: u32,

    /// Base URL for all requests.
    base_url: String,
}
```

### Enum Documentation

```rust
/// Represents possible error conditions during file operations.
///
/// # Examples
///
/// ```rust
/// use my_crate::FileError;
///
/// fn handle_error(error: FileError) {
///     match error {
///         FileError::NotFound(path) => eprintln!("File not found: {}", path),
///         FileError::PermissionDenied => eprintln!("Access denied"),
///         FileError::IoError(e) => eprintln!("IO error: {}", e),
///     }
/// }
/// ```
#[derive(Debug)]
pub enum FileError {
    /// The specified file was not found at the given path.
    NotFound(PathBuf),

    /// Permission was denied when accessing the file.
    PermissionDenied,

    /// An underlying I/O error occurred.
    IoError(std::io::Error),
}
```

### Trait Documentation

```rust
/// A type that can be serialized to a byte stream.
///
/// Implement this trait for types that need to be written to files,
/// sent over the network, or stored in databases.
///
/// # Examples
///
/// ```rust
/// use my_crate::Serialize;
///
/// struct Point { x: i32, y: i32 }
///
/// impl Serialize for Point {
///     fn serialize(&self) -> Vec<u8> {
///         let mut bytes = Vec::new();
///         bytes.extend_from_slice(&self.x.to_le_bytes());
///         bytes.extend_from_slice(&self.y.to_le_bytes());
///         bytes
///     }
/// }
/// ```
pub trait Serialize {
    /// Converts the value to a byte vector.
    fn serialize(&self) -> Vec<u8>;
}
```

### Unsafe Function Documentation

```rust
/// Dereferences a raw pointer and returns the value.
///
/// # Safety
///
/// The caller must ensure that:
/// - `ptr` is non-null
/// - `ptr` is properly aligned for type `T`
/// - `ptr` points to a valid, initialized value of type `T`
/// - The memory at `ptr` is not being mutated by another thread
///
/// # Examples
///
/// ```rust
/// use my_crate::read_ptr;
///
/// let value = 42i32;
/// let ptr = &value as *const i32;
///
/// // SAFETY: ptr is derived from a valid reference and value is initialized
/// let result = unsafe { read_ptr(ptr) };
/// assert_eq!(result, 42);
/// ```
pub unsafe fn read_ptr<T: Copy>(ptr: *const T) -> T {
    *ptr
}
```

## Documentation Tests (Doctests)

### Basic Doctests

Code blocks in documentation are automatically tested:

```rust
/// Adds two numbers.
///
/// ```rust
/// use my_crate::add;
///
/// assert_eq!(add(2, 3), 5);
/// assert_eq!(add(0, 0), 0);
/// ```
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

### Hiding Boilerplate

Use `#` to hide lines from rendered docs while keeping them in tests:

```rust
/// Parses a configuration file.
///
/// ```rust
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use my_crate::parse_config;
///
/// let config = parse_config("config.toml")?;
/// # Ok(())
/// # }
/// ```
pub fn parse_config(path: &str) -> Result<Config, ConfigError> {
    // ...
}
```

### Doctest Attributes

| Attribute | Purpose |
|-----------|---------|
| `ignore` | Skip this test |
| `should_panic` | Test must panic to pass |
| `no_run` | Compile but don't execute |
| `compile_fail` | Must fail to compile |
| `edition2021` | Use specific Rust edition |

Examples:

```rust
/// This example panics intentionally.
///
/// ```should_panic
/// panic!("This is expected");
/// ```
pub fn panicking_function() {}

/// This code won't compile (by design).
///
/// ```compile_fail
/// let x: i32 = "not a number";
/// ```
pub fn type_safe_function() {}

/// This example requires network access.
///
/// ```no_run
/// use my_crate::fetch_data;
/// let data = fetch_data("https://example.com");
/// ```
pub fn fetch_data(url: &str) -> Data {}
```

### Async Doctests

For async functions, wrap in a runtime:

```rust
/// Fetches data from a URL.
///
/// ```rust
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use my_crate::fetch;
///
/// let response = fetch("https://example.com").await?;
/// assert!(response.status().is_success());
/// # Ok(())
/// # }
/// ```
pub async fn fetch(url: &str) -> Result<Response, Error> {
    // ...
}
```

## Intra-Doc Links

Link to other items using their paths:

```rust
/// Creates a new [`Config`] with default values.
///
/// See [`Config::builder`] for more customization options.
///
/// This returns a [`Result<Config, ConfigError>`] that may contain
/// a [`ConfigError::InvalidPath`] if the default path is inaccessible.
pub fn default_config() -> Result<Config, ConfigError> {
    // ...
}
```

### Link Disambiguation

When names are ambiguous, use disambiguators:

```rust
/// See the [`foo`](fn@foo) function and [`Foo`](struct@Foo) struct.
pub fn example() {}

/// See the [`bar!`] macro.  // Trailing ! for macros
pub fn another() {}

/// See the [`baz()`] function.  // Trailing () for functions
pub fn yet_another() {}
```

### Available Disambiguators

| Disambiguator | Target |
|--------------|--------|
| `struct@Name` | Struct |
| `enum@Name` | Enum |
| `trait@Name` | Trait |
| `fn@Name` | Function |
| `mod@Name` | Module |
| `const@Name` | Constant |
| `type@Name` | Type alias |
| `macro@Name` | Macro |

## Doc Attributes

### Hiding Items

```rust
/// Internal implementation detail.
#[doc(hidden)]
pub struct InternalType;
```

### Adding Search Aliases

```rust
/// HTTP client for making web requests.
#[doc(alias = "web")]
#[doc(alias = "request")]
pub struct HttpClient;
```

### Controlling Inlining

```rust
// Force inline from private module
mod private {
    pub struct Public;
}
#[doc(inline)]
pub use private::Public;

// Prevent inline, show as re-export
#[doc(no_inline)]
pub use other_crate::ExternalType;
```

### Crate-Level Attributes

```rust
//! My crate documentation

#![doc(html_logo_url = "https://example.com/logo.png")]
#![doc(html_favicon_url = "https://example.com/favicon.ico")]
#![doc(html_playground_url = "https://play.rust-lang.org/")]
```

## Rustdoc Lints

### Default Warnings

| Lint | Purpose |
|------|---------|
| `broken_intra_doc_links` | Detects broken internal links |
| `private_intra_doc_links` | Links from public to private items |
| `invalid_codeblock_attributes` | Wrong doctest attributes |
| `invalid_html_tags` | Unclosed HTML tags |
| `bare_urls` | URLs not formatted as links |

### Optional Lints

Enable these for stricter checking:

```rust
#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]
#![warn(rustdoc::missing_doc_code_examples)]  // nightly only
```

## Style Guidelines

### Line Length

Keep documentation lines under 100 characters for readability.

### Markdown Formatting

Use standard Markdown:

```rust
/// # Header
///
/// Regular paragraph with **bold** and *italic* text.
///
/// - Bullet point 1
/// - Bullet point 2
///
/// ```rust
/// // Code block
/// let x = 42;
/// ```
///
/// | Column 1 | Column 2 |
/// |----------|----------|
/// | Value 1  | Value 2  |
```

### First Line Rule

The first line becomes the summary in listings. Keep it:
- One sentence
- Under 80 characters
- Descriptive and actionable

```rust
/// Creates a new configuration with sensible defaults.  // Good
///
/// More details follow after the blank line.
pub fn new() -> Config {}

/// This function creates a new configuration object that can be
/// used to configure the application with various settings.  // Bad - too long
pub fn new() -> Config {}
```

### What Not to Document

Don't describe obvious type information:

```rust
/// Bad: Takes a string slice and returns an Option<i32>.
pub fn parse(s: &str) -> Option<i32> {}

/// Good: Parses an integer from the input, returning None if invalid.
pub fn parse(s: &str) -> Option<i32> {}
```

## Generating Documentation

```bash
# Generate documentation
cargo doc

# Generate and open in browser
cargo doc --open

# Include private items
cargo doc --document-private-items

# Run doctests
cargo test --doc
```

## References

- [Rustdoc Book](https://doc.rust-lang.org/stable/rustdoc/)
- [How to Write Documentation](https://doc.rust-lang.org/stable/rustdoc/how-to-write-documentation.html)
- [API Guidelines - Documentation](https://rust-lang.github.io/api-guidelines/documentation.html)
- [RFC 1574 - API Documentation Conventions](https://rust-lang.github.io/rfcs/1574-more-api-documentation-conventions.html)
