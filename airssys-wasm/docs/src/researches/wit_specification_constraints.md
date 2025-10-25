# WIT Specification Constraints - Evidence-Based Summary

**Date:** 2025-10-25  
**Task:** WASM-TASK-003 Phase 1 Task 1.1  
**Sources:** WASI Preview 2 WIT files, wasm-tools examples, practical testing  
**Purpose:** Document WIT specification rules and constraints for ADR-WASM-015 implementation

---

## Package Declaration Requirements

### Format
```wit
package {namespace}:{name}@{version};
```

### Constraints (Evidence-Based)

**Namespace:**
- Format: Lowercase identifier
- Examples: `wasi`, `airssys`, `my-org`
- Pattern: `[a-z][a-z0-9-]*`
- Can contain hyphens (observed in WASI)

**Name:**
- Format: Lowercase identifier with optional hyphens
- Examples: `io`, `filesystem`, `core-types`, `ext-network`
- Pattern: `[a-z][a-z0-9-]*`
- Hyphens allowed for multi-word names

**Version:**
- Format: Semantic versioning `MAJOR.MINOR.PATCH`
- Examples: `0.2.8`, `1.0.0`, `2.1.3`
- Required: Must include all three version components
- Prefix: `@` symbol before version

**Terminator:**
- Required: Semicolon (`;`) at end of declaration

### Validated Examples from WASI

```wit
package wasi:io@0.2.8;
package wasi:filesystem@0.2.8;
package wasi:clocks@0.2.8;
package wasi:sockets@0.2.8;
```

### AIrsSys Pattern Validation

```wit
// ✅ VALID - Matches WASI pattern
package airssys:core-types@1.0.0;
package airssys:core-component@1.0.0;
package airssys:ext-filesystem@1.0.0;

// ❌ INVALID - Wrong separator
package airssys-core-types@1.0.0;

// ❌ INVALID - Missing version
package airssys:core-types;

// ❌ INVALID - Missing semicolon
package airssys:core-types@1.0.0
```

---

## Interface Definitions

### Syntax
```wit
interface {name} {
    // ... interface content
}
```

### Constraints

**Interface Names:**
- Format: Lowercase identifiers with optional hyphens
- Examples: `error`, `poll`, `streams`, `wall-clock`
- Pattern: `[a-z][a-z0-9-]*`
- Must be unique within package

**Visibility:**
- All interfaces in package are exported
- Can be imported by other packages via `use` statements

### WASI Examples

```wit
interface error { ... }
interface poll { ... }
interface streams { ... }
interface instance-network { ... }
interface ip-name-lookup { ... }
```

---

## World Definitions

### Syntax
```wit
world {name} {
    import {interface-name};
    export {interface-name};
}
```

### Purpose
- Defines component contract (imports and exports)
- Specifies what a component requires and provides
- Used for component type checking

### WASI Example

```wit
package wasi:io@0.2.8;

world imports {
    import streams;
    import poll;
}
```

### Constraints
- World names follow same pattern as interfaces
- Can import and/or export interfaces
- Can be empty (observed in minimal examples)

---

## Type System

### Built-in Types

**Primitive Types:**
- `bool` - Boolean value
- `u8`, `u16`, `u32`, `u64` - Unsigned integers
- `s8`, `s16`, `s32`, `s64` - Signed integers
- `f32`, `f64` - Floating point
- `char` - Unicode scalar value
- `string` - UTF-8 string

**Container Types:**
- `list<T>` - Ordered collection
- `option<T>` - Optional value
- `result<T, E>` - Success or error
- `tuple<T1, T2, ...>` - Fixed-size heterogeneous collection

### Type Definitions

**Record (Struct):**
```wit
record descriptor-stat {
    %type: descriptor-type,
    link-count: link-count,
    size: filesize,
    data-access-timestamp: option<datetime>,
}
```

**Constraints:**
- Field names: lowercase with hyphens
- Use `%` prefix for reserved keywords (e.g., `%type`)
- Trailing comma allowed after last field

**Enum (Tagged Union with No Data):**
```wit
enum descriptor-type {
    unknown,
    block-device,
    character-device,
    directory,
    fifo,
    symbolic-link,
    regular-file,
    socket,
}
```

**Constraints:**
- Variant names: lowercase with hyphens
- No associated data (use `variant` for that)
- Trailing comma allowed

**Variant (Tagged Union with Data):**
```wit
variant error-code {
    access(string),
    not-found,
    already-exists(u32),
}
```

**Constraints:**
- Can mix variants with and without data
- Data types must be valid WIT types

**Flags (Bitflags):**
```wit
flags descriptor-flags {
    read,
    write,
    file-integrity-sync,
    data-integrity-sync,
    requested-write-sync,
    mutate-directory,
}
```

**Constraints:**
- Represents bit flags that can be combined
- Each flag is a boolean value
- No associated data

**Type Alias:**
```wit
type filesize = u64;
```

**Resource (Component Model):**
```wit
resource pollable {
    ready: func() -> bool;
    block: func();
}
```

**Constraints:**
- Defines opaque handle with methods
- Cannot be copied or compared
- Lifetime managed by host or guest

---

## Function Definitions

### Syntax
```wit
{function-name}: func({params}) -> {return-type};
```

### Examples from WASI

```wit
// No parameters, returns bool
ready: func() -> bool;

// Parameters, no return
block: func();

// Multiple parameters, return type
poll: func(in: list<borrow<pollable>>) -> list<u32>;

// Result return type
open-at: func(
    path-flags: path-flags,
    path: string,
    open-flags: open-flags,
    flags: descriptor-flags,
) -> result<descriptor, error-code>;
```

### Constraints

**Function Names:**
- Lowercase with hyphens
- Pattern: `[a-z][a-z0-9-]*`

**Parameters:**
- Format: `{name}: {type}`
- Multiple parameters comma-separated
- Trailing comma allowed

**Return Types:**
- Can be any valid WIT type
- Empty return: `func()` (no `->`)
- Result pattern: `result<ok-type, err-type>`

**Ownership Qualifiers:**
- `borrow<T>` - Borrowed resource (read-only reference)
- No qualifier - Owned value

---

## Import/Export Syntax

### Use Statement (Import Types)
```wit
use wasi:io/streams@0.2.8.{input-stream, output-stream, error};
use wasi:clocks/wall-clock@0.2.8.{datetime};
```

### Format
```wit
use {package}.{interface-name}.{{type-list}};
```

### Constraints

**Package Reference:**
- Full package identifier: `namespace:name@version`
- Slash separator before interface: `/`
- Example: `wasi:io/streams@0.2.8`

**Type List:**
- Braced list: `{type1, type2, type3}`
- Can import specific types or interfaces
- Must be exported by referenced package

**Multiple Imports:**
```wit
use wasi:io/streams@0.2.8.{input-stream, output-stream};
use wasi:io/poll@0.2.8.{pollable};
```

---

## Documentation Comments

### Syntax
```wit
/// This is a documentation comment
interface example {
    /// Document specific items
    type-name: func();
}
```

### Constraints
- Use `///` for documentation
- Can be multi-line
- Placed before item being documented
- Markdown formatting supported

---

## Annotations

### Version Annotations
```wit
@since(version = 0.2.0)
interface poll { ... }

@since(version = 0.2.0)
resource pollable { ... }
```

**Purpose:** Track when features were introduced

### Unstable Features
```wit
@unstable(feature = foo)
interface experimental { ... }
```

**Purpose:** Mark features not yet stabilized

**Constraints:**
- Unstable features hidden by default
- Require `--features` flag to enable

---

## WIT Package Structure

### Single Package (Minimal)
```
package-dir/
└── types.wit
```

### Multi-Interface Package (WASI Pattern)
```
wasi:io/
├── error.wit       → interface error
├── poll.wit        → interface poll
├── streams.wit     → interface streams
└── world.wit       → world imports
```

**Constraints:**
- All `.wit` files in directory belong to same package
- Each file can define multiple interfaces
- One package declaration per package (typically in each file)

---

## Dependency Management (deps.toml)

### Purpose
Declare dependencies on other WIT packages

### Format (Inferred from usage)
```toml
[dependencies]
"{namespace}:{name}" = { path = "path/to/package" }
```

### Example Structure
```toml
[dependencies]
"wasi:io" = { path = "../wasi-io" }
"airssys:core-types" = { path = "./core/types.wit" }
```

**Note:** Exact format to be validated in Activity 1.2.2

---

## Naming Conventions Summary

| Element | Pattern | Example |
|---------|---------|---------|
| Package namespace | `[a-z][a-z0-9-]*` | `airssys`, `wasi` |
| Package name | `[a-z][a-z0-9-]*` | `core-types`, `filesystem` |
| Interface name | `[a-z][a-z0-9-]*` | `error`, `wall-clock` |
| Type name | `[a-z][a-z0-9-]*` | `descriptor-type`, `error-code` |
| Function name | `[a-z][a-z0-9-]*` | `open-at`, `to-debug-string` |
| Field/variant name | `[a-z][a-z0-9-]*` | `link-count`, `block-device` |

**Consistent Pattern:** Lowercase, hyphens for word separation, no underscores

---

## Reserved Keywords

**Identified from Testing and Spec:**
- `result` - Built-in result type
- `error` - Common but usable as interface name
- `option` - Built-in option type
- `list` - Built-in list type
- `tuple` - Built-in tuple type
- `use` - Import statement
- `import` - World import
- `export` - World export
- `interface` - Interface declaration
- `world` - World declaration
- `package` - Package declaration
- `type` - Type alias keyword (use `%type` in fields)
- `record` - Record declaration
- `enum` - Enum declaration
- `variant` - Variant declaration
- `flags` - Flags declaration
- `resource` - Resource declaration
- `func` - Function type

**Workaround:** Use descriptive names or `%` prefix in field names

---

## ADR-WASM-015 Compatibility Check

### Proposed Package Names

```wit
// Core packages
package airssys:core-types@1.0.0;          ✅ VALID
package airssys:core-component@1.0.0;      ✅ VALID
package airssys:core-capabilities@1.0.0;   ✅ VALID
package airssys:core-host@1.0.0;           ✅ VALID

// Extension packages
package airssys:ext-filesystem@1.0.0;      ✅ VALID
package airssys:ext-network@1.0.0;         ✅ VALID
package airssys:ext-process@1.0.0;         ✅ VALID
```

**Result:** All proposed names conform to WIT specification constraints

---

## Key Constraints for Implementation

### Package Level
1. ✅ One package per directory (all `.wit` files share package namespace)
2. ✅ Package name must use colon separator: `namespace:name`
3. ✅ Version required in semantic format: `@MAJOR.MINOR.PATCH`
4. ✅ Hyphens allowed in package names

### Type Level
1. ✅ Use hyphen-case for all identifiers (no underscores)
2. ✅ Avoid reserved keywords or use `%` prefix
3. ✅ Records, enums, variants, flags all follow same naming
4. ✅ Type aliases use `type` keyword

### Import Level
1. ✅ Use statements require full package version reference
2. ✅ Format: `use namespace:name/interface@version.{types}`
3. ✅ Dependencies must be resolvable via `deps.toml`

### Function Level
1. ✅ Parameters: `name: type` format
2. ✅ Return: `-> type` (omit for no return)
3. ✅ Result pattern: `result<ok, err>`

---

## Evidence Sources

### Primary Sources
1. **WASI Preview 2 WIT Files** (`.research/wasi-ref/wasip2/`)
   - `wasi:io` package (error.wit, poll.wit, streams.wit)
   - `wasi:filesystem` package (types.wit, preopens.wit)
   - `wasi:clocks` package (wall-clock.wit, monotonic-clock.wit)

2. **wasm-tools Test Cases** (`.research/wasm-tools-ref/`)
   - Minimal examples (smoke.wit, records.wit)
   - Test validation cases

3. **Practical Validation** (`tests/wit_validation/minimal_package/`)
   - Tested minimal valid package
   - Validated naming patterns
   - Confirmed error messages

---

## Next Steps

**Completed:**
- ✅ WIT specification constraints documented
- ✅ WASI patterns analyzed
- ✅ ADR-WASM-015 naming validated
- ✅ Evidence collected from multiple sources

**Next:**
- → Dependency management study (Activity 1.2.2)
- → Test package validation (Hour 5)
- → ADR feasibility confirmation (Hour 6)

---

**Document Version:** 1.0.0  
**Last Updated:** 2025-10-25  
**Research Status:** Complete - Specification constraints documented with evidence  
**Evidence Base:** WASI Preview 2, wasm-tools examples, practical testing
