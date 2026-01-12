# WASM-TASK-032: Implementation Plans

## Plan References
- **ADR-WASM-030:** Runtime Module Design (primary specification)
- **ADR-WASM-026:** Implementation Roadmap (Phase 5)

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

use std::path::Path;

use crate::core::component::id::ComponentId;
use crate::core::runtime::traits::ComponentLoader;

use super::engine::RuntimeError;

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
            id.namespace(),
            id.name(),
            id.instance()
        )
    }
}

impl ComponentLoader for FileComponentLoader {
    type Error = RuntimeError;

    fn load_bytes(&self, id: &ComponentId) -> Result<Vec<u8>, Self::Error> {
        let path = self.component_path(id);
        
        std::fs::read(&path).map_err(|e| {
            RuntimeError::ComponentNotFound(format!("Failed to load {}: {}", path, e))
        })
    }

    fn validate(&self, bytes: &[u8]) -> Result<(), Self::Error> {
        // Basic validation: check WASM magic number
        if bytes.len() < 8 {
            return Err(RuntimeError::InstantiationFailed("File too small".to_string()));
        }

        // WASM magic number: 0x00 0x61 0x73 0x6D (\0asm)
        if &bytes[0..4] != b"\0asm" {
            return Err(RuntimeError::InstantiationFailed(
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
    type Error = RuntimeError;

    fn load_bytes(&self, id: &ComponentId) -> Result<Vec<u8>, Self::Error> {
        self.components
            .get(&id.to_string_id())
            .cloned()
            .ok_or_else(|| RuntimeError::ComponentNotFound(id.to_string_id()))
    }

    fn validate(&self, bytes: &[u8]) -> Result<(), Self::Error> {
        if bytes.len() < 4 || &bytes[0..4] != b"\0asm" {
            return Err(RuntimeError::InstantiationFailed("Invalid WASM".to_string()));
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
        assert!(matches!(result, Err(RuntimeError::ComponentNotFound(_))));
    }
}
```

### Action 2: Update `runtime/mod.rs`

Add `pub mod loader;` to module declarations.

---

## Verification Commands

```bash
cargo build -p airssys-wasm
cargo clippy -p airssys-wasm --all-targets -- -D warnings
cargo test -p airssys-wasm --lib runtime::loader
```

---

## Success Criteria

- [ ] FileComponentLoader implemented
- [ ] InMemoryComponentLoader implemented (cfg(test))
- [ ] WASM magic number validation
- [ ] Build/Clippy pass with zero warnings
- [ ] All unit tests pass (6+ tests)
