# KNOWLEDGE-WASM-007: Component Storage Architecture

**Status:** Complete  
**Created:** 2025-10-18  
**Last Updated:** 2025-10-18  
**Related ADRs:** None yet  
**Related Tasks:** None yet  
**Dependencies:** KNOWLEDGE-WASM-004, KNOWLEDGE-WASM-005

---

## Table of Contents

1. [Overview](#overview)
2. [Storage Requirements for Components](#storage-requirements-for-components)
3. [Blockchain Storage Models Comparison](#blockchain-storage-models-comparison)
4. [Storage Backend Technologies Comparison](#storage-backend-technologies-comparison)
5. [Recommended Approach for airssys-wasm](#recommended-approach-for-airssys-wasm)
6. [Storage API Design](#storage-api-design)
7. [Permission Model and Security](#permission-model-and-security)
8. [Quota Management and Limits](#quota-management-and-limits)
9. [Implementation Patterns by Language](#implementation-patterns-by-language)
10. [Performance Considerations](#performance-considerations)
11. [Future Extensions](#future-extensions)
12. [References](#references)

---

## Overview

### Purpose

This document defines the persistent storage architecture for airssys-wasm components, inspired by proven blockchain storage models (Solana, NEAR Protocol, Ethereum/EVM) but adapted for general-purpose component systems. Components need durable key-value storage for maintaining state across executions, similar to smart contracts in blockchain environments.

### Scope

**In Scope:**
- Persistent key-value storage API for components
- Component-scoped storage isolation
- Permission-based access control
- Storage quota and limits management
- Storage backend selection and architecture
- Language-agnostic storage patterns

**Out of Scope (Explicitly Excluded):**
- Economic models (rent, staking, gas costs)
- Blockchain-specific consensus mechanisms
- Distributed storage across nodes
- Storage payment or billing systems

### Design Philosophy

**Research-Driven Design:**
Learn from production-tested blockchain storage systems that have handled billions of dollars in value and millions of transactions. Adapt their proven patterns to general-purpose component systems.

**Key Principles:**
- **Simple and Intuitive**: NEAR-style key-value API for ease of use
- **Component Isolation**: Each component has its own storage namespace
- **Permission-Based**: Storage access controlled by capability system
- **Practical Limits**: Reasonable quotas to prevent abuse
- **Language Agnostic**: Works seamlessly across Rust, JavaScript, Go, etc.

---

## Storage Requirements for Components

### Why Components Need Persistent Storage

**Stateful Applications:**
- Configuration persistence across restarts
- User preferences and settings
- Application state between invocations
- Cache storage for performance optimization
- Session data and temporary working storage

**Data Processing:**
- Intermediate computation results
- Aggregated data and analytics
- Processing checkpoints for recovery
- Historical data retention

**Integration State:**
- API tokens and credentials (encrypted)
- Connection states and bookmarks
- Last synchronization timestamps
- Rate limit counters and quotas

### Storage Characteristics Required

**Durability:**
- Data survives component restarts
- Data survives host system restarts
- Data persists across updates (with migration support)

**Isolation:**
- Component A cannot access Component B's storage
- Storage namespace scoped to component identity
- No cross-component storage leaks

**Performance:**
- Fast key-value lookups (target: <1ms)
- Efficient bulk operations
- Minimal overhead for small values
- Acceptable performance for large values (up to 1MB per value)

**Simplicity:**
- Intuitive API (get, set, delete, has, keys)
- No complex query languages needed
- Clear error handling
- Language-agnostic design

---

## Blockchain Storage Models Comparison

### Solana Storage Model

**Architecture: Account-Based Storage**

**Key Characteristics:**
```rust
// Solana Account Structure
pub struct Account {
    lamports: u64,        // Account balance (native token)
    data: Vec<u8>,        // Arbitrary data storage (max 10 MiB)
    owner: Pubkey,        // Program that owns this account
    executable: bool,     // Whether account contains executable code
    rent_epoch: Epoch,    // Rent collection tracking
}
```

**Storage Approach:**
- **Account-Based**: Data stored in "accounts" with owner program
- **Maximum Size**: 10 MiB per account
- **Serialization**: Manual (typically Borsh or Bincode)
- **Ownership**: Only owning program can modify account data
- **Creation**: System Program creates accounts, assigns ownership
- **Separation**: Program accounts (executable) separate from data accounts

**Economic Model (Excluded from Our Design):**
- Rent deposit proportional to data size
- Fully refundable on account closure
- Rent-exempt threshold for permanent storage

**Backend Implementation:**
- **AccountsDB**: Custom database optimized for Solana's needs
- Designed for high-throughput parallel access
- Account-based indexing and retrieval

**Strengths:**
- ✅ Low-level control and flexibility
- ✅ Clear ownership model
- ✅ Large storage capacity (10 MiB per account)
- ✅ High performance for parallel access

**Limitations:**
- ❌ Manual serialization required
- ❌ Complex account management
- ❌ Developers must handle data structure design
- ❌ No built-in collections or abstractions

### NEAR Protocol Storage Model

**Architecture: Built-in Key-Value Store**

**Key Characteristics:**
```rust
// NEAR Storage API
pub mod env {
    pub fn storage_write(key: &[u8], value: &[u8]) -> bool;
    pub fn storage_read(key: &[u8]) -> Option<Vec<u8>>;
    pub fn storage_remove(key: &[u8]) -> bool;
    pub fn storage_has_key(key: &[u8]) -> bool;
}
```

**Storage Approach:**
- **Key-Value Store**: Direct KV operations built into runtime
- **SDK Collections**: High-level abstractions (Vector, LookupMap, UnorderedMap, Set variants, TreeMap)
- **Prefix-Based Indexing**: Collections use prefixes to organize data chunks
- **Automatic Serialization**: SDK handles serialization transparently
- **Native vs SDK Collections**:
  - Native (Array, Map, Set): Serialize as single value, use for <100 entries
  - SDK Collections: Lazy-load from storage, use for large datasets

**Storage Collections:**
```rust
// NEAR SDK Collections
use near_sdk::collections::{
    Vector,           // O(1) indexed access
    LookupMap,        // O(1) non-iterable map
    UnorderedMap,     // O(1) iterable map
    LookupSet,        // O(1) non-iterable set
    UnorderedSet,     // O(1) iterable set
    TreeMap,          // O(log n) ordered map
};

// Example usage
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Contract {
    data: UnorderedMap<String, String>,  // Prefix: b"d"
    users: LookupMap<AccountId, User>,    // Prefix: b"u"
}
```

**Economic Model (Excluded from Our Design):**
- Storage staking: ~1Ⓝ per 100 KB
- Refunded when data deleted
- Contract pays for its storage

**Backend Implementation:**
- **RocksDB** or similar embedded key-value database
- Efficient prefix-based range queries
- Optimized for key-value workloads

**Constraints:**
- **Transaction Limit**: 4 MB per transaction upload
- **Prefix Management**: Developers must ensure unique collection prefixes
- **Iteration Performance**: O(n) for ordered collections

**Strengths:**
- ✅ Simple intuitive API
- ✅ High-level collection abstractions
- ✅ Automatic serialization via SDK
- ✅ Efficient for large datasets
- ✅ Clear mental model (KV store)

**Limitations:**
- ❌ Prefix collision risks if not careful
- ❌ No complex queries (key-value only)
- ❌ SDK collections tied to Borsh serialization

### Ethereum/EVM Storage Model

**Architecture: Merkle Patricia Trie (State Tree)**

**Key Characteristics:**
```solidity
// EVM Storage Model
contract Example {
    uint256 x;                           // Slot 0
    mapping(uint => address) users;      // Slot 1 (base)
    uint256[] dynamicArray;              // Slot 2 (base)
}

// Storage access
// Simple variables: Direct slot access
// Mappings: keccak256(h(key) . slot)
// Arrays: keccak256(slot) + index
```

**Storage Approach:**
- **Slot-Based**: 32-byte storage slots (2^256 slots per contract)
- **Automatic Layout**: Compiler determines storage layout
- **Packing**: Multiple values <32 bytes packed into single slot
- **Complex Types**:
  - Mappings: Hash-based addressing (keccak256)
  - Dynamic Arrays: Hash-based with length tracking
  - Structs: Contiguous slot allocation
- **State Trie**: Global Merkle Patricia Trie for all contract storage
- **Verification**: Cryptographic proofs of storage state

**Storage Layout Rules:**
- First variable starts at slot 0
- Variables <32 bytes packed together when possible
- Structs and arrays start new slots
- Mappings and dynamic arrays computed via keccak256

**Economic Model (Excluded from Our Design):**
- Gas costs for storage operations
- SSTORE (write): High gas cost
- SLOAD (read): Lower gas cost
- Refunds for clearing storage (setting to zero)

**Backend Implementation:**
- **Patricia Merkle Trie**: Cryptographically verifiable state tree
- **LevelDB** (Geth) or **MDBX** (Erigon): Underlying key-value database
- **Content Addressing**: `key = keccak256(rlp(value))`
- **Multiple Tries**:
  - State Trie: All accounts (path: keccak256(address))
  - Storage Trie: Per-contract storage (path: keccak256(storage_slot))
  - Transaction Trie: Transactions per block
  - Receipts Trie: Transaction receipts per block

**Strengths:**
- ✅ Automatic storage layout management
- ✅ Efficient packing for small values
- ✅ Cryptographic verification (Merkle proofs)
- ✅ Well-defined semantics
- ✅ Compiler handles complexity

**Limitations:**
- ❌ Complex addressing for mappings/arrays
- ❌ Gas optimization required (storage packing awareness)
- ❌ Limited to 32-byte slots
- ❌ No high-level collections
- ❌ Trie overhead for simple key-value operations

### Comparison Summary Table

| Aspect | Solana | NEAR Protocol | Ethereum/EVM |
|--------|--------|---------------|--------------|
| **Storage Model** | Account-based | Key-Value Store | Slot-based State Trie |
| **Data Structure** | Vec<u8> (10 MiB max) | KV pairs (unlimited keys) | 32-byte slots (2^256 slots) |
| **Abstraction Level** | Low-level (manual) | High-level (SDK collections) | Medium (compiler-managed) |
| **Serialization** | Manual (Borsh/Bincode) | Automatic (SDK) | Automatic (ABI encoding) |
| **Collections** | Manual implementation | Built-in (Vector, Map, Set) | Manual (Solidity types) |
| **Backend** | AccountsDB (custom) | RocksDB (embedded KV) | LevelDB/MDBX (trie-based) |
| **Verification** | Account hashing | Standard KV | Merkle proofs (cryptographic) |
| **Max Size** | 10 MiB per account | No hard limit (4 MB tx limit) | Unlimited (gas-limited) |
| **Indexing** | Account addresses | Prefix-based ranges | Keccak256-based addressing |
| **Developer UX** | Complex (flexible) | Simple (intuitive) | Medium (compiler-assisted) |
| **Performance** | High (parallel accounts) | High (optimized KV) | Medium (trie overhead) |
| **Use Case** | Fine-grained control | General-purpose apps | Verifiable computation |

---

## Storage Backend Technologies Comparison

### Solana: AccountsDB

**Technology:** Custom database implementation optimized for Solana

**Architecture:**
- Account-based indexing (Pubkey → Account)
- Parallel account access and modification
- Optimized for high-throughput validators
- Memory-mapped files for performance
- Snapshot-based state management

**Characteristics:**
- ✅ Extremely high performance (parallel writes)
- ✅ Optimized for Solana's specific needs
- ❌ Tightly coupled to Solana architecture
- ❌ Not reusable for general-purpose systems

### NEAR Protocol: RocksDB

**Technology:** RocksDB - embedded key-value database (Facebook/Meta)

**Architecture:**
- LSM-tree (Log-Structured Merge-tree) based
- Optimized for write-heavy workloads
- Efficient range queries (prefix scans)
- Built-in compression support
- ACID transactions
- Embeddable (no separate server process)

**Characteristics:**
- ✅ Production-proven (used by many blockchain projects)
- ✅ Excellent performance for KV workloads
- ✅ Efficient prefix-based range queries
- ✅ Well-maintained and documented
- ✅ Language bindings for Rust, C++, Java, Python
- ✅ Embeddable (library, not server)

**Performance:**
- Read: ~1 μs for cached keys
- Write: ~10 μs (with batching)
- Bulk operations: Highly efficient with write batches
- Compression: Reduces storage footprint

### Ethereum/EVM: LevelDB / MDBX

**Technology:** LevelDB (Google) or MDBX (Erigon's optimized fork)

**LevelDB Architecture:**
- LSM-tree based (similar to RocksDB)
- Originally designed for Chrome browser storage
- Simpler than RocksDB, fewer configuration options
- Good performance for general KV workloads
- Less optimized for specific use cases

**MDBX Architecture (Erigon):**
- LMDB fork with optimizations
- Memory-mapped B+ tree
- ACID transactions with MVCC
- Zero-copy reads (memory-mapped)
- Optimized for read-heavy workloads

**Characteristics:**
- LevelDB:
  - ✅ Simple and reliable
  - ✅ Well-tested in production
  - ❌ Less performant than RocksDB for heavy workloads
  - ❌ Limited configuration options
- MDBX:
  - ✅ Extremely fast reads (memory-mapped)
  - ✅ ACID with MVCC
  - ❌ More complex than LevelDB
  - ❌ Higher memory usage

**Used for Ethereum:**
- LevelDB: Geth (original implementation)
- MDBX: Erigon (optimized client)
- Stores Merkle Patricia Trie nodes as key-value pairs

### Alternative Backends (Not Used by Blockchain Examples)

**SQLite:**
- Embedded SQL database
- ACID transactions
- Simple to integrate
- ✅ Excellent for structured queries
- ✅ Zero configuration
- ❌ Not optimized for pure KV workloads
- ❌ Overhead of SQL layer for simple operations

**sled:**
- Modern Rust-native embedded database
- Lock-free architecture
- ACID transactions
- ✅ Pure Rust (no C dependencies)
- ✅ Modern API design
- ❌ Less production-proven than RocksDB
- ❌ Smaller ecosystem and tooling

**Recommendation for airssys-wasm:** RocksDB
- Production-proven in NEAR and many other projects
- Excellent performance for KV workloads
- Efficient prefix-based operations (component namespacing)
- Strong Rust bindings (rust-rocksdb crate)
- Embeddable (no external dependencies)
- Well-documented and maintained

---

## Recommended Approach for airssys-wasm

### Design Decision: Storage Backend Abstraction Layer

**Core Principle: Backend-Agnostic Design**

airssys-wasm uses a **trait-based abstraction layer** that decouples the storage API from specific backend implementations. This allows engineers to choose the most appropriate storage backend based on their requirements (compilation simplicity, performance, stability, features).

**Storage API Model: NEAR-Style Key-Value Store**

After comparing the three blockchain storage models, we adopt **NEAR Protocol's API approach** for the component-facing interface:

**Why NEAR's API Model:**
1. **Simplicity**: Intuitive key-value API that's easy to understand and use
2. **Developer Experience**: High-level abstractions reduce cognitive load
3. **Language Agnostic**: Simple KV operations work across all languages
4. **Proven**: Production-tested in NEAR Protocol with excellent results
5. **Flexible**: Can build collections on top of KV primitives as needed
6. **No Blockchain Complexity**: No need for Merkle proofs, gas optimization, or account management

**Why Not Solana's API Model:**
- Too low-level (manual serialization increases complexity)
- Account management adds unnecessary abstraction
- Better suited for blockchain-specific optimizations

**Why Not EVM's API Model:**
- Slot-based addressing too complex for general-purpose use
- Merkle trie overhead unnecessary without verification requirements
- 32-byte slot limitation too restrictive

**Backend Implementation: Pluggable Architecture**

The storage backend is **completely abstracted** via the `StorageBackend` trait, enabling multiple implementations:

**Supported Backend Options:**
1. **Sled** (Recommended Default) - Pure Rust, zero C++ dependencies, fast compilation
2. **RocksDB** (Production Alternative) - Battle-tested, optional C++ dependency for stability
3. **Future Backends** - SQLite, redb, custom implementations

**Backend Selection Criteria:**
- Development phase: Sled (pure Rust, fast iteration)
- Production deployment: Choice of Sled or RocksDB based on stability requirements
- Custom requirements: Implement `StorageBackend` trait for specialized needs

See **KNOWLEDGE-WASM-008** for comprehensive backend comparison and selection guide.

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     Component Layer                          │
│  ┌───────────────────────────────────────────────────────┐  │
│  │ Component A                                           │  │
│  │ - storage_get(key) → value                           │  │
│  │ - storage_set(key, value) → result                   │  │
│  │ - storage_delete(key) → bool                         │  │
│  │ - storage_has(key) → bool                            │  │
│  │ - storage_keys(prefix) → iterator                    │  │
│  └───────────────────────────────────────────────────────┘  │
└────────────────────────┬────────────────────────────────────┘
                         │ WIT Interface (NEAR-style KV API)
                         ↓
┌─────────────────────────────────────────────────────────────┐
│                   Host Runtime Layer                         │
│  ┌───────────────────────────────────────────────────────┐  │
│  │ Storage Manager                                       │  │
│  │ - Permission validation                               │  │
│  │ - Component namespace isolation                       │  │
│  │ - Quota enforcement                                   │  │
│  │ - Key prefixing (component-id + key)                 │  │
│  └─────────────────────┬─────────────────────────────────┘  │
└────────────────────────┼────────────────────────────────────┘
                         │
                         ↓ StorageBackend Trait (Abstraction)
┌─────────────────────────────────────────────────────────────┐
│              Storage Backend Abstraction Layer               │
│  ┌───────────────────────────────────────────────────────┐  │
│  │ trait StorageBackend {                                │  │
│  │   fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>>;│
│  │   fn set(&self, key: &[u8], value: &[u8]) -> Result<()>;│
│  │   fn delete(&self, key: &[u8]) -> Result<bool>;      │  │
│  │   fn prefix_iterator(&self, prefix: &[u8]) -> ...;   │  │
│  │   fn flush(&self) -> Result<()>;                      │  │
│  │ }                                                      │  │
│  └───────────────────────────────────────────────────────┘  │
└────────────────────────┬────────────────────────────────────┘
                         │
         ┌───────────────┼───────────────┐
         │               │               │
         ↓               ↓               ↓
┌─────────────┐  ┌─────────────┐  ┌─────────────┐
│SledBackend  │  │RocksDbBackend│  │CustomBackend│
│(Default)    │  │(Optional)   │  │(Pluggable)  │
│             │  │             │  │             │
│Pure Rust    │  │C++ Bindings │  │User-defined │
│Fast Build   │  │Production   │  │Special needs│
└─────────────┘  └─────────────┘  └─────────────┘
```

**Key Design Principles:**

1. **Abstraction Over Implementation**: Components interact with uniform API regardless of backend
2. **Pluggable Backends**: Easy to switch or add new storage engines via trait implementation
3. **Namespace Isolation**: Component storage completely isolated at Storage Manager level
4. **Performance**: Direct trait dispatch (no dynamic dispatch overhead after initialization)

### Component Namespace Isolation

**Key Prefixing Strategy:**
```
Storage Key Format: <component-id>:<user-key>

Examples:
- Component A stores key "config" → "component-a:config"
- Component B stores key "config" → "component-b:config"
- Component A stores key "users:alice" → "component-a:users:alice"
```

**Implementation (Backend-Agnostic):**
```rust
// Storage Manager uses trait-based backend
pub struct StorageManager<B: StorageBackend> {
    backend: Arc<B>,
    permissions: Arc<PermissionManager>,
}

impl<B: StorageBackend> StorageManager<B> {
    // Host runtime automatically prefixes keys
    fn storage_set(&self, component_id: &str, key: &str, value: &[u8]) -> Result<()> {
        // Validate permissions
        self.permissions.check_storage_write(component_id)?;
        
        // Prefix key with component ID
        let prefixed_key = format!("{}:{}", component_id, key);
        
        // Use backend trait method
        self.backend.set(prefixed_key.as_bytes(), value)?;
        Ok(())
    }

    fn storage_get(&self, component_id: &str, key: &str) -> Result<Option<Vec<u8>>> {
        let prefixed_key = format!("{}:{}", component_id, key);
        self.backend.get(prefixed_key.as_bytes())
    }

    fn storage_keys(&self, component_id: &str, prefix: Option<&str>) -> Result<Vec<String>> {
        let search_prefix = match prefix {
            Some(p) => format!("{}:{}", component_id, p),
            None => format!("{}:", component_id),
        };
        
        // Backend-agnostic prefix iteration
        let keys = self.backend.prefix_iterator(search_prefix.as_bytes())
            .map(|(k, _)| String::from_utf8_lossy(&k).to_string())
            .map(|k| k.strip_prefix(&format!("{}:", component_id)).unwrap().to_string())
            .collect();
        
        Ok(keys)
    }
}
```

**Benefits:**
- ✅ Complete isolation between components
- ✅ Efficient prefix-based operations (supported by all major KV stores)
- ✅ Simple to implement and reason about
- ✅ No risk of key collisions
- ✅ Backend-agnostic (works with any KV store)

---

## Storage Backend Abstraction

### StorageBackend Trait Definition

The core abstraction that enables pluggable storage backends:

```rust
use std::pin::Pin;
use std::future::Future;

/// Storage backend trait for pluggable storage implementations
pub trait StorageBackend: Send + Sync {
    /// Get value for key
    /// Returns None if key doesn't exist
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, StorageError>;

    /// Set value for key (overwrites if exists)
    fn set(&self, key: &[u8], value: &[u8]) -> Result<(), StorageError>;

    /// Delete key
    /// Returns true if key existed, false otherwise
    fn delete(&self, key: &[u8]) -> Result<bool, StorageError>;

    /// Check if key exists
    fn has(&self, key: &[u8]) -> Result<bool, StorageError> {
        // Default implementation using get()
        Ok(self.get(key)?.is_some())
    }

    /// Get iterator over keys with given prefix
    /// Returns iterator yielding (key, value) pairs
    fn prefix_iterator<'a>(
        &'a self,
        prefix: &[u8],
    ) -> Box<dyn Iterator<Item = Result<(Vec<u8>, Vec<u8>), StorageError>> + 'a>;

    /// Flush/sync data to disk
    fn flush(&self) -> Result<(), StorageError>;

    /// Async flush (optional, for async runtimes)
    fn flush_async(&self) -> Pin<Box<dyn Future<Output = Result<(), StorageError>> + Send>> {
        // Default implementation wraps sync flush
        let result = self.flush();
        Box::pin(async move { result })
    }

    /// Get backend-specific statistics (optional)
    fn stats(&self) -> Option<BackendStats> {
        None
    }
}

/// Backend statistics (optional feature)
#[derive(Debug, Clone)]
pub struct BackendStats {
    pub total_keys: usize,
    pub total_size_bytes: u64,
    pub backend_type: String,
    pub additional_info: std::collections::HashMap<String, String>,
}

/// Storage operation errors
#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Backend error: {0}")]
    Backend(String),
    
    #[error("Key not found")]
    KeyNotFound,
    
    #[error("Serialization error: {0}")]
    Serialization(String),
}
```

### Backend Implementation Examples

**Sled Backend:**
```rust
use sled::Db;

pub struct SledBackend {
    db: Db,
}

impl SledBackend {
    pub fn new(path: &str) -> Result<Self, StorageError> {
        let db = sled::open(path)
            .map_err(|e| StorageError::Backend(e.to_string()))?;
        Ok(Self { db })
    }
}

impl StorageBackend for SledBackend {
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, StorageError> {
        self.db.get(key)
            .map(|opt| opt.map(|iv| iv.to_vec()))
            .map_err(|e| StorageError::Backend(e.to_string()))
    }

    fn set(&self, key: &[u8], value: &[u8]) -> Result<(), StorageError> {
        self.db.insert(key, value)
            .map(|_| ())
            .map_err(|e| StorageError::Backend(e.to_string()))
    }

    fn delete(&self, key: &[u8]) -> Result<bool, StorageError> {
        self.db.remove(key)
            .map(|opt| opt.is_some())
            .map_err(|e| StorageError::Backend(e.to_string()))
    }

    fn prefix_iterator<'a>(
        &'a self,
        prefix: &[u8],
    ) -> Box<dyn Iterator<Item = Result<(Vec<u8>, Vec<u8>), StorageError>> + 'a> {
        let iter = self.db.scan_prefix(prefix).map(|result| {
            result
                .map(|(k, v)| (k.to_vec(), v.to_vec()))
                .map_err(|e| StorageError::Backend(e.to_string()))
        });
        Box::new(iter)
    }

    fn flush(&self) -> Result<(), StorageError> {
        self.db.flush()
            .map_err(|e| StorageError::Backend(e.to_string()))
    }

    fn flush_async(&self) -> Pin<Box<dyn Future<Output = Result<(), StorageError>> + Send>> {
        let db = self.db.clone();
        Box::pin(async move {
            db.flush_async()
                .await
                .map_err(|e| StorageError::Backend(e.to_string()))
        })
    }

    fn stats(&self) -> Option<BackendStats> {
        Some(BackendStats {
            total_keys: self.db.len(),
            total_size_bytes: self.db.size_on_disk().unwrap_or(0),
            backend_type: "sled".to_string(),
            additional_info: std::collections::HashMap::new(),
        })
    }
}
```

**RocksDB Backend (Optional):**
```rust
use rocksdb::{DB, Options};
use std::sync::Arc;

pub struct RocksDbBackend {
    db: Arc<DB>,
}

impl RocksDbBackend {
    pub fn new(path: &str) -> Result<Self, StorageError> {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.set_prefix_extractor(rocksdb::SliceTransform::create_fixed_prefix(36));
        
        let db = DB::open(&opts, path)
            .map_err(|e| StorageError::Backend(e.to_string()))?;
        Ok(Self { db: Arc::new(db) })
    }
}

impl StorageBackend for RocksDbBackend {
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, StorageError> {
        self.db.get(key)
            .map_err(|e| StorageError::Backend(e.to_string()))
    }

    fn set(&self, key: &[u8], value: &[u8]) -> Result<(), StorageError> {
        self.db.put(key, value)
            .map_err(|e| StorageError::Backend(e.to_string()))
    }

    fn delete(&self, key: &[u8]) -> Result<bool, StorageError> {
        let existed = self.db.get(key)?.is_some();
        self.db.delete(key)
            .map_err(|e| StorageError::Backend(e.to_string()))?;
        Ok(existed)
    }

    fn prefix_iterator<'a>(
        &'a self,
        prefix: &[u8],
    ) -> Box<dyn Iterator<Item = Result<(Vec<u8>, Vec<u8>), StorageError>> + 'a> {
        let iter = self.db.prefix_iterator(prefix).map(|result| {
            result
                .map(|(k, v)| (k.to_vec(), v.to_vec()))
                .map_err(|e| StorageError::Backend(e.to_string()))
        });
        Box::new(iter)
    }

    fn flush(&self) -> Result<(), StorageError> {
        self.db.flush()
            .map_err(|e| StorageError::Backend(e.to_string()))
    }

    fn stats(&self) -> Option<BackendStats> {
        // RocksDB-specific statistics
        Some(BackendStats {
            total_keys: 0, // Expensive to calculate
            total_size_bytes: 0, // Can get from property_value()
            backend_type: "rocksdb".to_string(),
            additional_info: std::collections::HashMap::new(),
        })
    }
}
```

### Backend Selection and Configuration

**Runtime Configuration:**
```rust
// Storage manager with generic backend
pub struct StorageManager<B: StorageBackend> {
    backend: Arc<B>,
    config: StorageConfig,
}

#[derive(Clone)]
pub struct StorageConfig {
    pub data_dir: String,
    pub max_component_quota: u64,
    pub default_component_quota: u64,
}

impl<B: StorageBackend> StorageManager<B> {
    pub fn new(backend: B, config: StorageConfig) -> Self {
        Self {
            backend: Arc::new(backend),
            config,
        }
    }
}

// Usage: Choose backend at runtime
fn create_storage_manager(config: StorageConfig) -> StorageManager<impl StorageBackend> {
    #[cfg(feature = "storage-rocksdb")]
    {
        let backend = RocksDbBackend::new(&config.data_dir).unwrap();
        StorageManager::new(backend, config)
    }
    
    #[cfg(not(feature = "storage-rocksdb"))]
    {
        let backend = SledBackend::new(&config.data_dir).unwrap();
        StorageManager::new(backend, config)
    }
}
```

**Benefits of Trait-Based Abstraction:**
- ✅ **Zero cost**: Trait dispatch resolved at compile time (monomorphization)
- ✅ **Flexibility**: Easy to add new backends without changing host code
- ✅ **Testing**: Can mock backends for unit tests
- ✅ **Migration**: Can switch backends without breaking component API
- ✅ **Feature flags**: Optional backends via Cargo features

---

## Storage API Design

### WIT Interface Definition

```wit
// storage-api.wit
package airssys:wasm@0.1.0;

interface storage {
    /// Storage operation errors
    variant storage-error {
        /// Permission denied for this operation
        permission-denied,
        /// Storage quota exceeded
        quota-exceeded,
        /// Key not found
        key-not-found,
        /// Invalid key format (empty, too long, invalid characters)
        invalid-key,
        /// Value too large (exceeds max value size)
        value-too-large,
        /// Internal storage system error
        internal-error(string),
    }

    /// Storage operation result
    type storage-result<T> = result<T, storage-error>;

    /// Get value for key (returns None if key doesn't exist)
    storage-get: func(key: string) -> storage-result<option<list<u8>>>;

    /// Set value for key (overwrites if exists)
    storage-set: func(key: string, value: list<u8>) -> storage-result<unit>;

    /// Delete key (returns true if key existed, false otherwise)
    storage-delete: func(key: string) -> storage-result<bool>;

    /// Check if key exists
    storage-has: func(key: string) -> storage-result<bool>;

    /// Get all keys with optional prefix filter
    /// Returns iterator for efficient large result handling
    storage-keys: func(prefix: option<string>) -> storage-result<list<string>>;

    /// Get current storage size used by component (in bytes)
    storage-size: func() -> storage-result<u64>;

    /// Clear all storage for this component
    /// Requires explicit "storage:admin" permission
    storage-clear: func() -> storage-result<unit>;
}

// Component imports this interface
world component {
    import storage;
}
```

### API Examples by Language

**Rust:**
```rust
use airssys_wasm::storage;

// Get value
match storage::storage_get("user:alice")? {
    Some(data) => {
        let user: User = serde_json::from_slice(&data)?;
        println!("User: {:?}", user);
    }
    None => println!("User not found"),
}

// Set value
let user = User { name: "Alice", age: 30 };
let data = serde_json::to_vec(&user)?;
storage::storage_set("user:alice", &data)?;

// Delete value
if storage::storage_delete("user:alice")? {
    println!("User deleted");
}

// Check existence
if storage::storage_has("user:alice")? {
    println!("User exists");
}

// List keys with prefix
let user_keys = storage::storage_keys(Some("user:"))?;
for key in user_keys {
    println!("Found user key: {}", key);
}

// Get storage usage
let size = storage::storage_size()?;
println!("Storage used: {} bytes", size);
```

**JavaScript:**
```javascript
import { storage } from 'airssys:wasm/storage';

// Get value
const data = await storage.storageGet('user:alice');
if (data) {
    const user = JSON.parse(new TextDecoder().decode(data));
    console.log('User:', user);
}

// Set value
const user = { name: 'Alice', age: 30 };
const data = new TextEncoder().encode(JSON.stringify(user));
await storage.storageSet('user:alice', data);

// Delete value
const deleted = await storage.storageDelete('user:alice');
console.log('Deleted:', deleted);

// Check existence
const exists = await storage.storageHas('user:alice');
console.log('Exists:', exists);

// List keys with prefix
const userKeys = await storage.storageKeys('user:');
userKeys.forEach(key => console.log('Found user key:', key));

// Get storage usage
const size = await storage.storageSize();
console.log('Storage used:', size, 'bytes');
```

**Go:**
```go
import "airssys.dev/wasm/storage"

// Get value
data, err := storage.StorageGet("user:alice")
if err != nil {
    return err
}
if data != nil {
    var user User
    json.Unmarshal(data, &user)
    fmt.Printf("User: %+v\n", user)
}

// Set value
user := User{Name: "Alice", Age: 30}
data, _ := json.Marshal(user)
storage.StorageSet("user:alice", data)

// Delete value
deleted, _ := storage.StorageDelete("user:alice")
fmt.Printf("Deleted: %v\n", deleted)

// Check existence
exists, _ := storage.StorageHas("user:alice")
fmt.Printf("Exists: %v\n", exists)

// List keys with prefix
prefix := "user:"
userKeys, _ := storage.StorageKeys(&prefix)
for _, key := range userKeys {
    fmt.Printf("Found user key: %s\n", key)
}

// Get storage usage
size, _ := storage.StorageSize()
fmt.Printf("Storage used: %d bytes\n", size)
```

### Helper Functions and Patterns

**Typed Storage Helpers (Rust):**
```rust
// Helper trait for typed storage operations
pub trait TypedStorage {
    fn get<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>>;
    fn set<T: Serialize>(&self, key: &str, value: &T) -> Result<()>;
}

impl TypedStorage for () {
    fn get<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>> {
        match storage::storage_get(key)? {
            Some(data) => Ok(Some(serde_json::from_slice(&data)?)),
            None => Ok(None),
        }
    }

    fn set<T: Serialize>(&self, key: &str, value: &T) -> Result<()> {
        let data = serde_json::to_vec(value)?;
        storage::storage_set(key, &data)?;
        Ok(())
    }
}

// Usage
let user: Option<User> = ().get("user:alice")?;
().set("user:alice", &user)?;
```

**Collection Abstractions (Optional Future Extension):**
```rust
// Similar to NEAR SDK collections
pub struct StorageMap<K, V> {
    prefix: String,
    _phantom: PhantomData<(K, V)>,
}

impl<K: Serialize, V: DeserializeOwned> StorageMap<K, V> {
    pub fn new(prefix: String) -> Self {
        Self { prefix, _phantom: PhantomData }
    }

    pub fn get(&self, key: &K) -> Result<Option<V>> {
        let key_str = format!("{}:{}", self.prefix, serde_json::to_string(key)?);
        match storage::storage_get(&key_str)? {
            Some(data) => Ok(Some(serde_json::from_slice(&data)?)),
            None => Ok(None),
        }
    }

    pub fn set(&self, key: &K, value: &V) -> Result<()> {
        let key_str = format!("{}:{}", self.prefix, serde_json::to_string(key)?);
        let data = serde_json::to_vec(value)?;
        storage::storage_set(&key_str, &data)?;
        Ok(())
    }

    pub fn keys(&self) -> Result<Vec<K>> {
        let keys = storage::storage_keys(Some(&format!("{}:", self.prefix)))?;
        keys.into_iter()
            .map(|k| {
                let key_part = k.strip_prefix(&format!("{}:", self.prefix)).unwrap();
                serde_json::from_str(key_part)
            })
            .collect()
    }
}

// Usage
let users = StorageMap::<String, User>::new("users".to_string());
users.set(&"alice".to_string(), &User { name: "Alice", age: 30 })?;
let user = users.get(&"alice".to_string())?;
```

---

## Permission Model and Security

### Storage Permission System

**Permission Requirement:**
Components must declare storage permission in their manifest to use storage APIs.

**Manifest Declaration:**
```toml
# component-manifest.toml
[component]
id = "my-component"
name = "My Component"
version = "1.0.0"

[permissions]
# Request storage permission
storage = { access = "read-write", max-size = "10MB" }

# Alternative: Read-only storage
# storage = { access = "read-only" }

# Alternative: Admin access (allows storage-clear)
# storage = { access = "admin", max-size = "50MB" }
```

**Permission Levels:**

| Level | Operations Allowed | Use Case |
|-------|-------------------|----------|
| `none` | No storage access | Components that don't need persistence |
| `read-only` | `get`, `has`, `keys`, `size` | Read-only components, reporting tools |
| `read-write` | All except `clear` | Standard components needing persistence |
| `admin` | All operations including `clear` | System management components |

**Runtime Enforcement:**
```rust
// Host runtime validates permissions before operation
pub fn storage_set(
    component_id: &ComponentId,
    permissions: &Permissions,
    key: &str,
    value: &[u8],
) -> Result<(), StorageError> {
    // Check permission level
    match permissions.storage {
        StoragePermission::None | StoragePermission::ReadOnly => {
            return Err(StorageError::PermissionDenied);
        }
        StoragePermission::ReadWrite | StoragePermission::Admin => {
            // Allowed, continue
        }
    }

    // Check quota
    let current_size = get_component_storage_size(component_id)?;
    let new_size = current_size + value.len() as u64;
    if new_size > permissions.storage_quota {
        return Err(StorageError::QuotaExceeded);
    }

    // Perform operation
    let prefixed_key = format!("{}:{}", component_id, key);
    rocksdb.put(prefixed_key.as_bytes(), value)?;
    
    Ok(())
}
```

### Security Considerations

**Namespace Isolation:**
- Each component's storage is completely isolated via key prefixing
- No component can access another component's storage
- Host runtime enforces isolation at API boundary

**Key Validation:**
```rust
// Validate key format
fn validate_key(key: &str) -> Result<(), StorageError> {
    // Empty keys not allowed
    if key.is_empty() {
        return Err(StorageError::InvalidKey);
    }
    
    // Max key length (e.g., 256 bytes)
    if key.len() > 256 {
        return Err(StorageError::InvalidKey);
    }
    
    // Disallow control characters and null bytes
    if key.chars().any(|c| c.is_control() || c == '\0') {
        return Err(StorageError::InvalidKey);
    }
    
    Ok(())
}
```

**Value Size Limits:**
```rust
const MAX_VALUE_SIZE: usize = 1024 * 1024; // 1 MB per value

fn validate_value(value: &[u8]) -> Result<(), StorageError> {
    if value.len() > MAX_VALUE_SIZE {
        return Err(StorageError::ValueTooLarge);
    }
    Ok(())
}
```

**Encrypted Storage (Future Extension):**
```rust
// Optional: Encrypt sensitive data at application level
use aes_gcm::{Aes256Gcm, Key, Nonce};
use aes_gcm::aead::{Aead, NewAead};

pub fn encrypted_set(key: &str, plaintext: &[u8], encryption_key: &[u8]) -> Result<()> {
    let cipher = Aes256Gcm::new(Key::from_slice(encryption_key));
    let nonce = Nonce::from_slice(&[0u8; 12]); // Use proper nonce generation
    let ciphertext = cipher.encrypt(nonce, plaintext)?;
    
    storage::storage_set(key, &ciphertext)?;
    Ok(())
}

pub fn encrypted_get(key: &str, encryption_key: &[u8]) -> Result<Option<Vec<u8>>> {
    match storage::storage_get(key)? {
        Some(ciphertext) => {
            let cipher = Aes256Gcm::new(Key::from_slice(encryption_key));
            let nonce = Nonce::from_slice(&[0u8; 12]);
            let plaintext = cipher.decrypt(nonce, ciphertext.as_ref())?;
            Ok(Some(plaintext))
        }
        None => Ok(None),
    }
}
```

---

## Quota Management and Limits

### Storage Quotas

**Default Quotas by Permission Level:**

| Permission Level | Default Quota | Max Quota (Configurable) |
|-----------------|---------------|--------------------------|
| `none` | 0 bytes | N/A |
| `read-only` | N/A (read-only) | N/A |
| `read-write` | 10 MB | 100 MB |
| `admin` | 50 MB | 500 MB |

**Quota Configuration:**
```toml
# component-manifest.toml
[permissions]
storage = { access = "read-write", max-size = "25MB" }
```

**Quota Tracking:**
```rust
// Host runtime tracks storage usage per component
pub struct ComponentStorageInfo {
    pub component_id: ComponentId,
    pub current_size: u64,
    pub quota: u64,
    pub key_count: usize,
}

impl StorageManager {
    // Calculate component storage size
    pub fn get_component_size(&self, component_id: &ComponentId) -> Result<u64> {
        let prefix = format!("{}:", component_id);
        let mut total_size = 0u64;
        
        let iter = self.db.prefix_iterator(prefix.as_bytes());
        for (key, value) in iter {
            total_size += key.len() as u64 + value.len() as u64;
        }
        
        Ok(total_size)
    }
    
    // Check if operation would exceed quota
    pub fn check_quota(
        &self,
        component_id: &ComponentId,
        additional_bytes: u64,
    ) -> Result<bool> {
        let current = self.get_component_size(component_id)?;
        let quota = self.get_component_quota(component_id)?;
        
        Ok(current + additional_bytes <= quota)
    }
}
```

**Quota Enforcement:**
```rust
// Before write operations
pub fn storage_set(
    component_id: &ComponentId,
    key: &str,
    value: &[u8],
) -> Result<(), StorageError> {
    // Get existing value size (for updates)
    let prefixed_key = format!("{}:{}", component_id, key);
    let existing_size = match self.db.get(prefixed_key.as_bytes())? {
        Some(v) => v.len() as u64,
        None => 0,
    };
    
    // Calculate size delta
    let new_size = key.len() as u64 + value.len() as u64;
    let size_delta = new_size.saturating_sub(existing_size);
    
    // Check quota
    if !self.check_quota(component_id, size_delta)? {
        return Err(StorageError::QuotaExceeded);
    }
    
    // Perform write
    self.db.put(prefixed_key.as_bytes(), value)?;
    
    Ok(())
}
```

### Operational Limits

**Per-Operation Limits:**
```rust
pub const MAX_KEY_LENGTH: usize = 256;           // 256 bytes
pub const MAX_VALUE_SIZE: usize = 1024 * 1024;   // 1 MB
pub const MAX_KEYS_PER_QUERY: usize = 10_000;    // 10K keys
pub const MAX_BATCH_SIZE: usize = 100;           // 100 operations
```

**Performance Targets:**
- Single `get`: <1ms (cached), <10ms (disk)
- Single `set`: <10ms (with fsync), <1ms (without fsync)
- `keys()` iteration: <100ms for 10K keys
- Quota calculation: <100ms for components with <10K keys

---

## Implementation Patterns by Language

### Rust Implementation

**Using Storage Backend Abstraction:**
```rust
use std::sync::Arc;

/// Component-scoped storage wrapper (backend-agnostic)
pub struct ComponentStorage<B: StorageBackend> {
    backend: Arc<B>,
    component_id: String,
}

impl<B: StorageBackend> ComponentStorage<B> {
    pub fn new(backend: Arc<B>, component_id: String) -> Self {
        Self { backend, component_id }
    }

    fn prefixed_key(&self, key: &str) -> String {
        format!("{}:{}", self.component_id, key)
    }

    pub fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        let prefixed = self.prefixed_key(key);
        self.backend.get(prefixed.as_bytes())
            .map_err(|e| StorageError::InternalError(e.to_string()))
    }

    pub fn set(&self, key: &str, value: &[u8]) -> Result<()> {
        validate_key(key)?;
        validate_value(value)?;
        
        let prefixed = self.prefixed_key(key);
        self.backend.set(prefixed.as_bytes(), value)
            .map_err(|e| StorageError::InternalError(e.to_string()))
    }

    pub fn delete(&self, key: &str) -> Result<bool> {
        let prefixed = self.prefixed_key(key);
        self.backend.delete(prefixed.as_bytes())
            .map_err(|e| StorageError::InternalError(e.to_string()))
    }

    pub fn keys(&self, prefix: Option<&str>) -> Result<Vec<String>> {
        let search_prefix = match prefix {
            Some(p) => self.prefixed_key(p),
            None => format!("{}:", self.component_id),
        };
        
        let component_prefix = format!("{}:", self.component_id);
        let keys: Vec<String> = self.backend
            .prefix_iterator(search_prefix.as_bytes())
            .filter_map(|result| result.ok())
            .map(|(k, _)| String::from_utf8_lossy(&k).to_string())
            .filter(|k| k.starts_with(&search_prefix))
            .map(|k| k.strip_prefix(&component_prefix).unwrap().to_string())
            .collect();
        
        Ok(keys)
    }
}

// Example: Create storage with specific backend
fn example_usage() -> Result<()> {
    // Option 1: Use Sled backend
    let sled_backend = SledBackend::new("data/storage")?;
    let storage = ComponentStorage::new(Arc::new(sled_backend), "my-component".to_string());
    
    // Option 2: Use RocksDB backend (if feature enabled)
    #[cfg(feature = "storage-rocksdb")]
    {
        let rocks_backend = RocksDbBackend::new("data/storage")?;
        let storage = ComponentStorage::new(Arc::new(rocks_backend), "my-component".to_string());
    }
    
    // Use storage (backend-agnostic)
    storage.set("config", b"value")?;
    let value = storage.get("config")?;
    
    Ok(())
}
```

### JavaScript/TypeScript Implementation

**Component SDK wrapper:**
```typescript
// storage.ts
import { storage as wasmStorage } from 'airssys:wasm/storage';

export class Storage {
    async get<T>(key: string): Promise<T | null> {
        const data = await wasmStorage.storageGet(key);
        if (!data) return null;
        
        const json = new TextDecoder().decode(data);
        return JSON.parse(json) as T;
    }

    async set<T>(key: string, value: T): Promise<void> {
        const json = JSON.stringify(value);
        const data = new TextEncoder().encode(json);
        await wasmStorage.storageSet(key, data);
    }

    async delete(key: string): Promise<boolean> {
        return await wasmStorage.storageDelete(key);
    }

    async has(key: string): Promise<boolean> {
        return await wasmStorage.storageHas(key);
    }

    async keys(prefix?: string): Promise<string[]> {
        return await wasmStorage.storageKeys(prefix || null);
    }

    async size(): Promise<number> {
        return await wasmStorage.storageSize();
    }

    // Helper: Map-like interface
    async entries<T>(prefix?: string): Promise<Map<string, T>> {
        const keys = await this.keys(prefix);
        const entries = new Map<string, T>();
        
        for (const key of keys) {
            const value = await this.get<T>(key);
            if (value !== null) {
                entries.set(key, value);
            }
        }
        
        return entries;
    }
}

// Usage
const storage = new Storage();

interface User {
    name: string;
    age: number;
}

const user = await storage.get<User>('user:alice');
await storage.set('user:alice', { name: 'Alice', age: 30 });
```

### Go Implementation

**Component SDK wrapper:**
```go
package storage

import (
    "encoding/json"
    "airssys.dev/wasm/bindings/storage"
)

type Storage struct{}

func (s *Storage) Get(key string, result interface{}) error {
    data, err := storage.StorageGet(key)
    if err != nil {
        return err
    }
    if data == nil {
        return nil
    }
    return json.Unmarshal(data, result)
}

func (s *Storage) Set(key string, value interface{}) error {
    data, err := json.Marshal(value)
    if err != nil {
        return err
    }
    return storage.StorageSet(key, data)
}

func (s *Storage) Delete(key string) (bool, error) {
    return storage.StorageDelete(key)
}

func (s *Storage) Has(key string) (bool, error) {
    return storage.StorageHas(key)
}

func (s *Storage) Keys(prefix *string) ([]string, error) {
    return storage.StorageKeys(prefix)
}

func (s *Storage) Size() (uint64, error) {
    return storage.StorageSize()
}

// Usage
type User struct {
    Name string `json:"name"`
    Age  int    `json:"age"`
}

st := &Storage{}

var user User
err := st.Get("user:alice", &user)

user = User{Name: "Alice", Age: 30}
st.Set("user:alice", user)
```

---

## Performance Considerations

### Read Performance

**Cached Reads (RocksDB block cache):**
- Latency: <1 μs
- Throughput: >1M ops/sec

**Disk Reads (SSD):**
- Latency: 1-10 ms
- Throughput: 10K-100K ops/sec

**Optimization Strategies:**
```rust
// Use batch reads for multiple keys
pub fn batch_get(&self, keys: &[String]) -> Result<Vec<Option<Vec<u8>>>> {
    let mut results = Vec::with_capacity(keys.len());
    
    // RocksDB multi-get optimization
    let prefixed_keys: Vec<_> = keys.iter()
        .map(|k| self.prefixed_key(k))
        .collect();
    
    let values = self.db.multi_get(prefixed_keys)?;
    
    for value in values {
        results.push(value?);
    }
    
    Ok(results)
}
```

### Write Performance

**Without fsync (default):**
- Latency: ~1 ms
- Throughput: ~100K ops/sec

**With fsync (durability guarantee):**
- Latency: ~10 ms (depends on disk)
- Throughput: ~10K ops/sec

**Optimization Strategies:**
```rust
// Use batch writes for multiple operations
use rocksdb::WriteBatch;

pub fn batch_set(&self, operations: &[(String, Vec<u8>)]) -> Result<()> {
    let mut batch = WriteBatch::default();
    
    for (key, value) in operations {
        validate_key(key)?;
        validate_value(value)?;
        
        let prefixed = self.prefixed_key(key);
        batch.put(prefixed.as_bytes(), value);
    }
    
    self.db.write(batch)
        .map_err(|e| StorageError::InternalError(e.to_string()))
}
```

### Memory Usage

**RocksDB Memory Configuration:**
```rust
let mut opts = Options::default();

// Block cache (default: 8 MB, recommend: 64-512 MB)
opts.set_block_cache_size(64 * 1024 * 1024);

// Write buffer size (default: 4 MB)
opts.set_write_buffer_size(16 * 1024 * 1024);

// Max write buffers (default: 2)
opts.set_max_write_buffer_number(3);

// Compression (reduces disk usage)
opts.set_compression_type(rocksdb::DBCompressionType::Lz4);
```

### Storage Space Optimization

**Compression:**
- LZ4: Fast, moderate compression (~50% reduction)
- Zstd: Slower, better compression (~60-70% reduction)
- Snappy: Very fast, light compression (~40% reduction)

```rust
// Configure compression per level
opts.set_compression_per_level(&[
    rocksdb::DBCompressionType::None,   // Level 0
    rocksdb::DBCompressionType::None,   // Level 1
    rocksdb::DBCompressionType::Lz4,    // Level 2+
    rocksdb::DBCompressionType::Lz4,
    rocksdb::DBCompressionType::Zstd,   // Highest levels
]);
```

**Compaction:**
```rust
// Manual compaction for maintenance
self.db.compact_range(None::<&[u8]>, None::<&[u8]>);

// Automatic compaction tuning
opts.set_max_background_jobs(4);
opts.set_level_compaction_dynamic_level_bytes(true);
```

### Monitoring and Metrics

**Performance Metrics to Track:**
```rust
pub struct StorageMetrics {
    pub reads: u64,
    pub writes: u64,
    pub deletes: u64,
    pub bytes_read: u64,
    pub bytes_written: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub avg_read_latency_us: f64,
    pub avg_write_latency_us: f64,
}

impl StorageManager {
    pub fn get_metrics(&self) -> StorageMetrics {
        // Collect RocksDB statistics
        let stats = self.db.property_value("rocksdb.stats").unwrap();
        // Parse and return metrics
        // ...
    }
}
```

---

## Future Extensions

### Phase 1: Basic Storage (Current Design)
- Simple key-value operations (get, set, delete, has, keys)
- Component namespace isolation
- Basic quota enforcement
- RocksDB backend

### Phase 2: Advanced Features

**Transactions: REJECTED (See ADR-WASM-013)**

Transactions were explicitly rejected in favor of actor model sequential processing guarantees. The actor model provides equivalent consistency guarantees without the complexity of transaction management:

- **Actor Sequential Processing**: Each component processes messages sequentially (one at a time)
- **No Concurrent Access**: Component storage accessed only by owning component actor
- **Atomicity**: Message handler execution is atomic at storage operation level
- **Consistency**: Sequential execution eliminates race conditions and lost updates
- **Blockchain Precedent**: EVM, Solana, NEAR all use sequential execution without storage transactions

**Rationale**: Actor model message boundaries provide natural transaction semantics. Multi-key operations within a single message handler are atomic due to sequential processing. No need for explicit transaction API.

**Reference**: ADR-WASM-007 Decision 5, ADR-WASM-013

**~~Transactions (Rejected):~~**
```wit
// REJECTED: Not needed due to actor model sequential processing
// See ADR-WASM-013 for complete rationale
//
// interface storage-transactions {
//     transaction-begin: func() -> storage-result<transaction-id>;
//     transaction-commit: func(txn: transaction-id) -> storage-result<unit>;
//     transaction-rollback: func(txn: transaction-id) -> storage-result<unit>;
//     transaction-get: func(txn: transaction-id, key: string) -> storage-result<option<list<u8>>>;
//     transaction-set: func(txn: transaction-id, key: string, value: list<u8>) -> storage-result<unit>;
// }
```

**Range Queries:**
```wit
interface storage-ranges {
    /// Get range of keys (start inclusive, end exclusive)
    storage-range: func(
        start: string,
        end: string,
        limit: option<u32>
    ) -> storage-result<list<tuple<string, list<u8>>>>;
    
    /// Scan with cursor for pagination
    storage-scan: func(
        prefix: string,
        cursor: option<string>,
        limit: u32
    ) -> storage-result<tuple<list<tuple<string, list<u8>>>, option<string>>>;
}
```

**Collection Abstractions:**
```rust
// SDK-provided collection types
pub struct StorageVector<T> { ... }
pub struct StorageMap<K, V> { ... }
pub struct StorageSet<T> { ... }
pub struct StorageQueue<T> { ... }
```

### Phase 3: Advanced Capabilities
**Time-To-Live (TTL):**
```wit
/// Set value with expiration
storage-set-with-ttl: func(
    key: string,
    value: list<u8>,
    ttl-seconds: u64
) -> storage-result<unit>;

/// Get remaining TTL
storage-get-ttl: func(key: string) -> storage-result<option<u64>>;
```

**Storage Events:**
```wit
/// Subscribe to storage changes
storage-watch: func(prefix: string) -> storage-result<watch-id>;

/// Storage change event
record storage-event {
    key: string,
    operation: storage-operation,  // set, delete
    old-value: option<list<u8>>,
    new-value: option<list<u8>>,
}
```

**Backup and Migration:**
```wit
/// Export component storage
storage-export: func() -> storage-result<list<u8>>;

/// Import component storage
storage-import: func(data: list<u8>) -> storage-result<unit>;
```

### Phase 4: Distributed Storage (Future Research)
- Cross-host storage synchronization
- Eventual consistency models
- Conflict resolution strategies
- Distributed transactions

---

## References

### Blockchain Documentation
- **Solana Accounts**: https://docs.solana.com/developing/programming-model/accounts
- **NEAR Storage**: https://docs.near.org/build/smart-contracts/anatomy/storage
- **NEAR Collections**: https://docs.near.org/sdk/rust/contract-structure/collections
- **Ethereum Storage Layout**: https://docs.soliditylang.org/en/latest/internals/layout_in_storage.html
- **Ethereum Patricia Trie**: https://ethereum.org/developers/docs/data-structures-and-encoding/patricia-merkle-trie/

### Storage Backend Documentation
- **RocksDB**: https://rocksdb.org/
- **RocksDB Rust**: https://github.com/rust-rocksdb/rust-rocksdb
- **LevelDB**: https://github.com/google/leveldb
- **MDBX**: https://github.com/erthink/libmdbx

### Related AirsSys Documentation
- **KNOWLEDGE-WASM-004**: WIT Management Architecture (permission model)
- **KNOWLEDGE-WASM-005**: Messaging Architecture (component communication)
- **KNOWLEDGE-WASM-006**: Multiformat Strategy (data serialization)
- **KNOWLEDGE-WASM-008**: Storage Backend Comparison (comprehensive backend analysis) ⭐ **NEW**

### Design Decisions

**Why Trait-Based Abstraction:**
- Zero-cost abstraction via compile-time monomorphization
- Easy to add new backends without changing host runtime
- Enables testing with mock backends
- Allows users to choose based on requirements (compilation speed vs stability)
- Future-proof: Can switch backends without breaking component API

**Why Sled as Default Backend:**
- Pure Rust implementation (no C++ dependencies)
- Fast compilation and development iteration
- Zero FFI overhead
- Modern lock-free architecture
- Good performance for general use cases
- See KNOWLEDGE-WASM-008 for detailed comparison

**Why RocksDB as Optional Backend:**
- Production-proven in critical systems (NEAR, Facebook, Polkadot)
- Excellent stability and maturity
- Superior space efficiency
- Large ecosystem and tooling
- Available via feature flag for production deployments
- See KNOWLEDGE-WASM-008 for detailed comparison

**Why NEAR-style API over Solana-style:**
- Simpler developer experience (no account management)
- More intuitive for general-purpose components
- Language-agnostic (works well across all supported languages)
- Proven in production without blockchain-specific complexity

**Why Not EVM Slot-Based Model:**
- Unnecessary complexity for general-purpose storage
- Merkle trie overhead without verification benefits
- 32-byte slot limitation too restrictive
- Hash-based addressing adds cognitive overhead

---

## Document History

| Version | Date | Changes | Author |
|---------|------|---------|--------|
| 1.0 | 2025-10-18 | Initial complete storage architecture | AI Agent |

---

**Next Steps:**
1. ✅ Create KNOWLEDGE-WASM-008 for comprehensive storage backend comparison
2. Create ADR for storage abstraction layer design
3. Create ADR for default backend selection (Sled vs RocksDB)
4. Design detailed implementation plan for storage manager
5. Create integration tasks for airssys-rt actor system
6. Design storage migration patterns for component updates
7. Implement storage quota monitoring and alerting system
8. Create mock backend for testing and development

