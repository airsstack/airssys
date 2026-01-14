# WASM-TASK-032: Implementation Plans

## Plan References
- **ADR-WASM-030:** Runtime Module Design (primary specification)
- **ADR-WASM-026:** Implementation Roadmap (Phase 5)
- **ADR-WASM-023:** Module Boundary Enforcement (CRITICAL - runtime/ cannot import from actor/)
- **ADR-WASM-025:** Clean-Slate Rebuild Architecture (provides context for runtime module)
- **KNOWLEDGE-WASM-037:** Rebuild Architecture (understanding the clean-slate approach)
- **KNOWLEDGE-WASM-039:** Runtime Module Responsibility (what runtime/ owns and what it doesn't)

**Note on ADR vs PROJECTS_STANDARD.md Conflicts:**
If any ADR patterns conflict with PROJECTS_STANDARD.md, the plan follows PROJECTS_STANDARD.md.
Any adjustments are noted with: "(Adjusted from ADR to comply with PROJECTS_STANDARD.md §[X.Y])"

**System Patterns:**
- ComponentLoader trait from core/ (runtime implements it)
- File-based loading with namespace structure
- WASM binary validation

**PROJECTS_STANDARD.md Compliance:**
- §2.1 (3-Layer Imports): Code will follow import organization
- §2.2 (No FQN): Types will be imported and used by simple name
- §3.2 (DateTime<Utc>): Time operations will use Utc (if applicable)
- §4.3 (Module Architecture): mod.rs files will only contain declarations
- §5.1 (Dependency Management): Dependencies from workspace will be correctly referenced
- §6.1 (YAGNI): Simple, direct solutions without over-engineering
- §6.2 (Avoid `dyn`): Static dispatch preferred over trait objects
- §6.4 (Implementation Quality Gates): Zero warnings, comprehensive tests

**Rust Guidelines Applied:**
- M-DESIGN-FOR-AI: Idiomatic APIs, thorough docs, testable
- M-MODULE-DOCS: Module documentation will be added
- M-ERRORS-CANONICAL-STRUCTS: Error types follow canonical structure (WasmError)
- M-STATIC-VERIFICATION: All lints enabled, clippy used
- M-FEATURES-ADDITIVE: Features will not break existing code

**Documentation Standards:**
- Diátaxis Type: Reference documentation for ComponentLoader implementations
- Quality: Professional tone, no hyperbole per documentation-quality-standards.md
- Evidence: Standards Compliance Checklist will be included

## Target Structure Reference

```
runtime/
├── mod.rs
├── engine.rs        # (WASM-TASK-031) ✅
├── loader.rs        # ← THIS TASK
├── store.rs         # (WASM-TASK-033)
├── host_fn.rs       # (WASM-TASK-034)
└── limiter.rs       # (WASM-TASK-035)
```

---

## Implementation Actions

### Action 1: Create `runtime/loader.rs`

**Objective:** Implement ComponentLoader trait implementations

**File:** `airssys-wasm/src/runtime/loader.rs`

**Specification (ADR-WASM-030 lines 283-375):**

```rust
//! Component loading implementations.

use crate::core::component::id::ComponentId;
use crate::core::runtime::errors::WasmError;
use crate::core::runtime::traits::ComponentLoader;

/// File-based component loader
pub struct FileComponentLoader {
    base_path: String,
}

impl FileComponentLoader {
    pub fn new(base_path: impl Into<String>) -> Self {
        Self {
            base_path: base_path.into(),
        }
    }

    fn component_path(&self, id: &ComponentId) -> String {
        format!(
            "{}/{}/{}/{}.wasm",
            self.base_path,
            id.namespace,
            id.name,
            id.instance
        )
    }
}

impl ComponentLoader for FileComponentLoader {
    fn load_bytes(&self, id: &ComponentId) -> Result<Vec<u8>, WasmError> {
        let path = self.component_path(id);

        std::fs::read(&path).map_err(|e| {
            WasmError::ComponentNotFound(format!("Failed to load {}: {}", path, e))
        })
    }

    fn validate(&self, bytes: &[u8]) -> Result<(), WasmError> {
        // Basic validation: check WASM magic number
        if bytes.len() < 4 {
            return Err(WasmError::InvalidComponent("File too small".to_string()));
        }

        // WASM magic number: 0x00 0x61 0x73 0x6D (\0asm)
        if &bytes[0..4] != b"\0asm" {
            return Err(WasmError::InvalidComponent(
                "Invalid WASM magic number".to_string(),
            ));
        }

        Ok(())
    }
}

/// In-memory component loader for testing
#[cfg(test)]
pub struct InMemoryComponentLoader {
    components: std::collections::HashMap<String, Vec<u8>>,
}

#[cfg(test)]
impl InMemoryComponentLoader {
    pub fn new() -> Self {
        Self {
            components: std::collections::HashMap::new(),
        }
    }

    pub fn add_component(&mut self, id: &ComponentId, bytes: Vec<u8>) {
        self.components.insert(id.to_string_id(), bytes);
    }
}

#[cfg(test)]
impl ComponentLoader for InMemoryComponentLoader {
    fn load_bytes(&self, id: &ComponentId) -> Result<Vec<u8>, WasmError> {
        self.components
            .get(&id.to_string_id())
            .cloned()
            .ok_or_else(|| WasmError::ComponentNotFound(id.to_string_id()))
    }

    fn validate(&self, bytes: &[u8]) -> Result<(), WasmError> {
        if bytes.len() < 4 || &bytes[0..4] != b"\0asm" {
            return Err(WasmError::InvalidComponent("Invalid WASM".to_string()));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_loader_path_construction() {
        let loader = FileComponentLoader::new("/wasm");
        let id = ComponentId::new("ns", "comp", "0");
        let path = loader.component_path(&id);
        assert_eq!(path, "/wasm/ns/comp/0.wasm");
    }

    #[test]
    fn test_validate_valid_wasm_magic() {
        let loader = FileComponentLoader::new("/wasm");
        let bytes = b"\0asm\x01\x00\x00\x00";
        assert!(loader.validate(bytes).is_ok());
    }

    #[test]
    fn test_validate_invalid_magic() {
        let loader = FileComponentLoader::new("/wasm");
        let bytes = b"notw\x01\x00\x00\x00";
        assert!(loader.validate(bytes).is_err());
    }

    #[test]
    fn test_validate_too_small() {
        let loader = FileComponentLoader::new("/wasm");
        let bytes = b"\0as";
        assert!(loader.validate(bytes).is_err());
    }

    #[test]
    fn test_in_memory_loader() {
        let mut loader = InMemoryComponentLoader::new();
        let id = ComponentId::new("test", "comp", "0");
        let bytes = b"\0asm\x01\x00\x00\x00".to_vec();
        
        loader.add_component(&id, bytes.clone());
        
        let loaded = loader.load_bytes(&id).unwrap();
        assert_eq!(loaded, bytes);
    }

    #[test]
    fn test_in_memory_loader_not_found() {
        let loader = InMemoryComponentLoader::new();
        let id = ComponentId::new("test", "comp", "0");

        let result = loader.load_bytes(&id);
        assert!(matches!(result, Err(WasmError::ComponentNotFound(_))));
    }
}
```

### Action 2: Update `runtime/mod.rs`

Add `pub mod loader;` to module declarations.

---

## Verification Commands

```bash
# 1. Build check
cargo build -p airssys-wasm

# 2. Lint check (zero warnings)
cargo clippy -p airssys-wasm --all-targets -- -D warnings

# 3. Unit tests
cargo test -p airssys-wasm --lib runtime::loader

# 4. Module architecture verification (ADR-WASM-023 - MANDATORY)
grep -rn "use crate::actor" airssys-wasm/src/runtime/
# Expected: [no output - clean]
```

## Testing Note

**Why integration tests are not required for this task:**

This task implements simple ComponentLoader trait implementations that:
- Read bytes from filesystem or in-memory storage
- Validate WASM magic number (4-byte check)
- Return appropriate error types

The functionality is:
1. Pure IO operations (no complex business logic)
2. Self-contained (no interaction with other modules)
3. Already fully covered by unit tests

Integration tests will be added in later tasks when:
- Components are actually loaded into the WASM engine
- End-to-end workflows involving multiple modules exist
- Real WASM components need to be tested

For now, the 6+ unit tests provide complete coverage of the ComponentLoader implementations.

---

## Success Criteria

- [ ] FileComponentLoader implemented
- [ ] InMemoryComponentLoader implemented (cfg(test))
- [ ] WASM magic number validation
- [ ] Build/Clippy pass with zero warnings
- [ ] All unit tests pass (6+ tests)
