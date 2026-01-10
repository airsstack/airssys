# KNOWLEDGE-WASM-041: Storage Management Architecture (Clean-Slate Rebuild)

**Status:** Active  
**Created:** 2026-01-10  
**Last Updated:** 2026-01-10  
**Related ADRs:** ADR-WASM-028 (Core Module Structure), ADR-WASM-025 (Clean-Slate Rebuild)  
**Related Tasks:** WASM-TASK-021 (core/storage/ submodule)  
**Supersedes:** Partially supersedes KNOWLEDGE-WASM-007 for clean-slate rebuild context

---

## Table of Contents

1. [Overview](#overview)
2. [Storage Call Flow Architecture](#storage-call-flow-architecture)
3. [Namespace Isolation Model](#namespace-isolation-model)
4. [Type System Design](#type-system-design)
5. [Storage Layer Composition](#storage-layer-composition)
6. [Caching Strategy](#caching-strategy)
7. [Implementation Location](#implementation-location)
8. [References](#references)

---

## Overview

### Purpose

This document defines the storage management architecture for the **clean-slate rebuild** of airssys-wasm. It builds upon the foundational concepts in KNOWLEDGE-WASM-007 but specifies the concrete design for Phase 3 (Core Module) implementation.

### Scope

**In Scope:**
- Storage call flow from component to backend
- Namespace isolation (Solana-inspired)
- Type system (`StorageValue`, `ComponentStorage` trait)
- Layered storage composition (namespace, cache, backend)
- Module location for each type

**Out of Scope:**
- Backend implementations (Sled, RocksDB) - see KNOWLEDGE-WASM-007/008
- Economic models - not applicable
- Distributed storage - future extension

### Key Design Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Namespace isolation | Host-enforced implicit | Solana-inspired, simple API, security |
| Value type | Dedicated `StorageValue` ADT | Domain boundary clarity |
| Caching | Layered decorator pattern | Composable, flexible |
| Trait location | `core/storage/` (Layer 1) | Abstraction in foundation |

---

## Storage Call Flow Architecture

### Complete Flow Diagram

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                          WASM Component (Guest)                              │
│                                                                              │
│  // Component code calls WIT-generated bindings                              │
│  let value = storage::get("user:123")?;                                      │
│                                                                              │
│  NOTE: Component does NOT implement ComponentStorage                         │
│        Component calls WIT interface only                                    │
│                                                                              │
└──────────────────────────────────┬──────────────────────────────────────────┘
                                   │
                                   │ FFI Boundary (wit_bindgen::generate!)
                                   │ Generates Rust bindings from storage.wit
                                   ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                         Wasmtime Host Functions                              │
│                                                                              │
│  // Host receives call with component context (Caller<State>)                │
│  fn host_storage_get(                                                        │
│      caller: Caller<ComponentState>,  // Knows which component called        │
│      key: String,                                                            │
│  ) -> Result<Option<Vec<u8>>, StorageError> {                                │
│      let storage = caller.data().storage(); // Gets composed storage chain  │
│      storage.get(&key)  // Calls ComponentStorage trait                      │
│  }                                                                           │
│                                                                              │
│  NOTE: Host function bridges WIT to Rust, uses storage from component state  │
│                                                                              │
└──────────────────────────────────┬──────────────────────────────────────────┘
                                   │
                                   │ Uses ComponentStorage trait
                                   ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                    Storage Chain (Decorator Pattern)                         │
│                                                                              │
│  All layers implement ComponentStorage trait                                 │
│                                                                              │
│  ┌───────────────────────────────────────────────────────────────────────┐  │
│  │ Layer 1: NamespacedStorage<CachedStorage<Backend>>                    │  │
│  │   - Implements ComponentStorage                                       │  │
│  │   - Prefixes all keys with component namespace                        │  │
│  │   - Delegates to inner layer                                          │  │
│  └───────────────────────────────────────────────────────────────────────┘  │
│                               │                                              │
│  ┌───────────────────────────────────────────────────────────────────────┐  │
│  │ Layer 2: CachedStorage<Backend>                                       │  │
│  │   - Implements ComponentStorage                                       │  │
│  │   - In-memory cache (LRU, TTL)                                        │  │
│  │   - Cache hit → return cached value                                   │  │
│  │   - Cache miss → delegate to backend                                  │  │
│  └───────────────────────────────────────────────────────────────────────┘  │
│                               │                                              │
│  ┌───────────────────────────────────────────────────────────────────────┐  │
│  │ Layer 3: Backend (FileStorage, MemoryStorage, etc.)                   │  │
│  │   - Implements ComponentStorage                                       │  │
│  │   - Actual persistence                                                │  │
│  └───────────────────────────────────────────────────────────────────────┘  │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Key Points

1. **Component (Guest)**: Calls WIT interface, does NOT implement `ComponentStorage`
2. **Host Functions**: Bridge WIT calls to Rust, get storage from component state
3. **Storage Chain**: All layers implement `ComponentStorage`, compose via delegation

---

## Namespace Isolation Model

### Design Decision (2026-01-10)

**Model:** Host-enforced implicit namespace isolation (Solana-inspired)

### How It Works

```
┌─────────────────────────────────────────────────────────────────┐
│ Component A calls: storage.get("user:123")                      │
│         ↓                                                       │
│ NamespacedStorage prefixes: get("app/service-a/001/user:123")   │
│         ↓                                                       │
│ ✅ Returns Component A's data only                              │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│ Component B calls: storage.get("user:123")                      │
│         ↓                                                       │
│ NamespacedStorage prefixes: get("app/service-b/001/user:123")   │
│         ↓                                                       │
│ ✅ Returns Component B's data (different namespace)             │
└─────────────────────────────────────────────────────────────────┘
```

### Namespace Format

```
{namespace}/{name}/{instance}/{user_key}
└───────────────────────────┘ └─────────┘
      ComponentId part        User's key
```

**Example:**
- ComponentId: `{ namespace: "app", name: "service-a", instance: "001" }`
- User key: `"user:123"`
- Namespaced key: `"app/service-a/001/user:123"`

### Security Guarantees

| Guarantee | Implementation |
|-----------|---------------|
| **Isolation** | Keys automatically prefixed by NamespacedStorage |
| **No cross-access** | Component cannot specify another namespace |
| **Transparent** | Component API is simple (just `key: &str`) |
| **Enforced** | Host layer, not component, controls prefixing |

### Future Extensions

- Cross-component access via explicit **StorageCapability** grants
- For now, strict isolation is the only mode

---

## Type System Design

### Two Type Systems

The storage system has two separate but related type systems:

| Layer | Type | Location | Purpose |
|-------|------|----------|---------|
| **WIT Bindings** | `storage-value` → `Vec<u8>` | Generated by `wit_bindgen::generate!` | FFI/Host-Guest boundary |
| **Core API** | `StorageValue` struct | `core/storage/value.rs` | Internal domain abstraction |

### Why Separate Types?

**Domain Boundary Clarity:**
- `MessagePayload` is for messaging domain
- `StorageValue` is for storage domain
- Engineers immediately know purpose from type name

### Type Conversions

```rust
// core/storage/value.rs
impl From<Vec<u8>> for StorageValue {
    fn from(data: Vec<u8>) -> Self {
        Self::new(data)
    }
}

impl From<StorageValue> for Vec<u8> {
    fn from(value: StorageValue) -> Self {
        value.into_bytes()
    }
}
```

### Core Types Summary

```rust
// core/storage/value.rs
pub struct StorageValue(Vec<u8>);

// core/storage/errors.rs (aligned with WIT errors.wit)
pub enum StorageError {
    NotFound(String),
    AlreadyExists(String),
    QuotaExceeded,
    InvalidKey(String),
    IoError(String),
}

// core/storage/traits.rs
pub trait ComponentStorage: Send + Sync {
    fn get(&self, key: &str) -> Result<Option<StorageValue>, StorageError>;
    fn set(&self, key: &str, value: StorageValue) -> Result<(), StorageError>;
    fn delete(&self, key: &str) -> Result<(), StorageError>;
    fn exists(&self, key: &str) -> Result<bool, StorageError>;
    fn list_keys(&self, prefix: Option<&str>) -> Result<Vec<String>, StorageError>;
}
```

---

## Storage Layer Composition

### NamespacedStorage Wrapper

```rust
// core/storage/namespaced.rs
use crate::core::component::id::ComponentId;
use super::traits::ComponentStorage;
use super::value::StorageValue;
use super::errors::StorageError;

/// Wraps a storage backend with automatic namespace prefixing.
///
/// All keys are prefixed with the component's namespace,
/// ensuring isolation between components.
pub struct NamespacedStorage<S: ComponentStorage> {
    namespace: ComponentId,
    inner: S,
}

impl<S: ComponentStorage> NamespacedStorage<S> {
    pub fn new(namespace: ComponentId, storage: S) -> Self {
        Self { namespace, inner: storage }
    }

    fn prefix_key(&self, key: &str) -> String {
        format!("{}/{}", self.namespace.to_string_id(), key)
    }

    fn strip_prefix<'a>(&self, key: &'a str) -> &'a str {
        let prefix = format!("{}/", self.namespace.to_string_id());
        key.strip_prefix(&prefix).unwrap_or(key)
    }
}

impl<S: ComponentStorage> ComponentStorage for NamespacedStorage<S> {
    fn get(&self, key: &str) -> Result<Option<StorageValue>, StorageError> {
        self.inner.get(&self.prefix_key(key))
    }

    fn set(&self, key: &str, value: StorageValue) -> Result<(), StorageError> {
        self.inner.set(&self.prefix_key(key), value)
    }

    fn delete(&self, key: &str) -> Result<(), StorageError> {
        self.inner.delete(&self.prefix_key(key))
    }

    fn exists(&self, key: &str) -> Result<bool, StorageError> {
        self.inner.exists(&self.prefix_key(key))
    }

    fn list_keys(&self, prefix: Option<&str>) -> Result<Vec<String>, StorageError> {
        let ns_prefix = match prefix {
            Some(p) => self.prefix_key(p),
            None => format!("{}/", self.namespace.to_string_id()),
        };
        let keys = self.inner.list_keys(Some(&ns_prefix))?;
        Ok(keys.into_iter().map(|k| self.strip_prefix(&k).to_string()).collect())
    }
}
```

### Host Composition Example

```rust
// During host initialization for each component
fn create_component_storage(
    component_id: ComponentId,
    backend: Arc<dyn ComponentStorage>,
    cache_config: CacheConfig,
) -> Box<dyn ComponentStorage> {
    // Compose layers (innermost first)
    let cached = CachedStorage::new(backend, cache_config);
    let namespaced = NamespacedStorage::new(component_id, cached);
    Box::new(namespaced)
}
```

---

## Caching Strategy

### CachedStorage Wrapper

```rust
// core/storage/cached.rs (or system/storage/cached.rs)
use std::time::{Duration, Instant};
use std::collections::HashMap;
use std::sync::RwLock;

pub struct CacheConfig {
    pub max_entries: usize,
    pub ttl: Option<Duration>,
    pub max_size_bytes: usize,
}

struct CacheEntry {
    value: StorageValue,
    created_at: Instant,
}

pub struct CachedStorage<S: ComponentStorage> {
    inner: S,
    cache: RwLock<HashMap<String, CacheEntry>>,
    config: CacheConfig,
}

impl<S: ComponentStorage> CachedStorage<S> {
    pub fn new(storage: S, config: CacheConfig) -> Self {
        Self {
            inner: storage,
            cache: RwLock::new(HashMap::new()),
            config,
        }
    }

    fn is_expired(&self, entry: &CacheEntry) -> bool {
        if let Some(ttl) = self.config.ttl {
            entry.created_at.elapsed() > ttl
        } else {
            false
        }
    }
}

impl<S: ComponentStorage> ComponentStorage for CachedStorage<S> {
    fn get(&self, key: &str) -> Result<Option<StorageValue>, StorageError> {
        // Check cache first
        if let Some(entry) = self.cache.read().unwrap().get(key) {
            if !self.is_expired(entry) {
                return Ok(Some(entry.value.clone()));
            }
        }
        
        // Cache miss - fetch from backend
        let value = self.inner.get(key)?;
        
        // Update cache
        if let Some(ref v) = value {
            let mut cache = self.cache.write().unwrap();
            cache.insert(key.to_string(), CacheEntry {
                value: v.clone(),
                created_at: Instant::now(),
            });
            // TODO: Eviction if over capacity
        }
        
        Ok(value)
    }

    fn set(&self, key: &str, value: StorageValue) -> Result<(), StorageError> {
        // Write-through: update backend first
        self.inner.set(key, value.clone())?;
        
        // Update cache
        let mut cache = self.cache.write().unwrap();
        cache.insert(key.to_string(), CacheEntry {
            value,
            created_at: Instant::now(),
        });
        
        Ok(())
    }

    fn delete(&self, key: &str) -> Result<(), StorageError> {
        // Remove from cache
        self.cache.write().unwrap().remove(key);
        // Delete from backend
        self.inner.delete(key)
    }

    fn exists(&self, key: &str) -> Result<bool, StorageError> {
        // Check cache first
        if self.cache.read().unwrap().contains_key(key) {
            return Ok(true);
        }
        self.inner.exists(key)
    }

    fn list_keys(&self, prefix: Option<&str>) -> Result<Vec<String>, StorageError> {
        // Cache doesn't help here, delegate to backend
        self.inner.list_keys(prefix)
    }
}
```

### Cache Characteristics

| Feature | Implementation |
|---------|---------------|
| **Eviction Policy** | LRU (can extend to TTL-based) |
| **Write Strategy** | Write-through (sync) |
| **Scope** | Per-component (under NamespacedStorage) |
| **Invalidation** | On delete, TTL expiry |
| **Memory Limits** | Configurable max entries and size |

---

## Implementation Location

### Module Locations Summary

| Type | Module | Layer | Created In |
|------|--------|-------|------------|
| `StorageValue` | `core/storage/value.rs` | Layer 1 | WASM-TASK-021 |
| `StorageError` | `core/storage/errors.rs` | Layer 1 | WASM-TASK-021 |
| `ComponentStorage` trait | `core/storage/traits.rs` | Layer 1 | WASM-TASK-021 |
| `NamespacedStorage` | `core/storage/namespaced.rs` | Layer 1 | WASM-TASK-021 (extended) |
| `CachedStorage` | `system/storage/cached.rs` | Layer 4 | Future task |
| Backend impls | `system/storage/backend/` | Layer 4 | Future task |

### Directory Structure (After WASM-TASK-021)

```
core/storage/
├── mod.rs           # Module declarations
├── value.rs         # StorageValue ADT
├── errors.rs        # StorageError enum
├── traits.rs        # ComponentStorage trait
└── namespaced.rs    # NamespacedStorage wrapper [NEW - Option B]
```

---

## References

- **KNOWLEDGE-WASM-007**: Component Storage Architecture (foundational concepts)
- **KNOWLEDGE-WASM-008**: Storage Backend Comparison
- **ADR-WASM-028**: Core Module Structure (Layer 1 compliance)
- **ADR-WASM-025**: Clean-Slate Rebuild Architecture
- **Solana Account Model**: Inspiration for namespace isolation
- **NEAR Protocol Storage**: Inspiration for simple KV API
