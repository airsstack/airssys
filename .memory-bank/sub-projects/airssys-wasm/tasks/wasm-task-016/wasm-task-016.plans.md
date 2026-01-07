# WASM-TASK-016: Implementation Plans

## Plan References
- **ADR-WASM-025:** Clean-Slate Rebuild Architecture
- **ADR-WASM-026:** Implementation Roadmap (Phase 2 - lines 83-105)
- **ADR-WASM-023:** Module Boundary Enforcement (dependency rules)
- **KNOWLEDGE-WASM-037:** Rebuild Architecture Clean-Slate

## Target Structure Reference

Per ADR-WASM-026 lines 94-105, final structure after Phase 2:
```
airssys-wasm/src/
├── lib.rs              ← THIS TASK (update exports)
├── prelude.rs
├── core/               # LAYER 1
├── security/           # LAYER 2A
├── runtime/            # LAYER 2B
├── component/          # LAYER 3A (renamed from actor/)
├── messaging/          # LAYER 3B (new)
└── system/             # LAYER 4 (new)
```

## Implementation Actions

### Action 1: Update module declarations in lib.rs

**Objective:** Declare all 6 modules with updated dependency diagram

**File:** `src/lib.rs`

**Changes:**
1. Replace `pub mod actor;` with `pub mod component;`
2. Add `pub mod messaging;` after component
3. Add `pub mod system;` after messaging
4. Update documentation to reflect new structure

**Updated module section:**
```rust
// Foundation layer (no internal dependencies)
pub mod core;

// Security layer (imports from core/)
pub mod security;

// WASM execution layer (imports from core/, security/)
pub mod runtime;

// Component integration layer (imports from core/, security/, runtime/)
pub mod component;

// Messaging layer (imports from core/, security/, runtime/)
pub mod messaging;

// System integration layer (imports all lower layers)
pub mod system;
```

### Action 2: Update documentation diagram

**Objective:** Update the dependency diagram in lib.rs header

**Current diagram (lines 20-25):**
```
actor/ ──► runtime/ ──► security/ ──► core/
```

**New diagram:**
```
system/ ──► component/ ──► runtime/ ──► security/ ──► core/
   │            │             │              │
   │            └─────────────┼──────────────┤
   │                          │              │
   └───► messaging/ ──────────┴──────────────┘
```

### Action 3: Update module documentation

**Objective:** Update module list in lib.rs documentation

**Update lines 51-55:**
```rust
//! ## Module Documentation
//!
//! - [core](core) - Core types and abstractions
//! - [security](security) - Security capabilities and policies
//! - [runtime](runtime) - WASM execution engine
//! - [component](component) - Component lifecycle and management
//! - [messaging](messaging) - Inter-component messaging
//! - [system](system) - System integration and orchestration
```

## Verification Commands

Run after ALL actions complete:
```bash
# 1. Build check
cargo build -p airssys-wasm

# 2. Lint check
cargo clippy -p airssys-wasm --all-targets -- -D warnings

# 3. Documentation generation
cargo doc -p airssys-wasm --no-deps

# 4. Module boundary check (per ADR-WASM-026 lines 220-227)
grep -rn "use crate::component" src/runtime/
grep -rn "use crate::messaging" src/runtime/
grep -rn "use crate::system" src/runtime/
grep -rn "use crate::runtime" src/security/
grep -rn "use crate::" src/core/
# All should return empty
```

## Success Criteria
- All 6 modules properly declared
- Build passes with zero warnings
- Documentation reflects new architecture
- Phase 2 complete, ready for Phase 3
