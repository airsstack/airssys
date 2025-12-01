# ADR-WASM-007: Storage Backend Selection

**Status:** Accepted  
**Date:** 2025-10-19  
**Decision Makers:** Architecture Team  
**Related:** KNOWLEDGE-WASM-007 (Component Storage Architecture), KNOWLEDGE-WASM-008 (Storage Backend Comparison), ADR-WASM-005 (Capability-Based Security)

---

## Context

The airssys-wasm framework requires persistent key-value storage for components to maintain state, configuration, and data across executions. Components need durable storage similar to smart contracts in blockchain systems (Solana, NEAR Protocol, Ethereum), but adapted for general-purpose component applications.

### The Problem

**Components Need Persistent Storage:**
- Configuration persistence across restarts
- Application state between invocations
- Cache storage for performance
- User data and preferences
- Integration state (tokens, bookmarks, counters)

**Requirements:**
- **Durability**: Data survives component and system restarts
- **Isolation**: Component A cannot access Component B's storage
- **Performance**: Fast key-value operations (<1ms reads, <10ms writes)
- **Simplicity**: Intuitive API, language-agnostic
- **Quotas**: Prevent storage exhaustion by malicious components
- **Security**: Integrated with capability-based permission system

**Technical Constraints:**
- Must work with WASM Component Model
- Must support multiple languages (Rust, JavaScript, Go, Python)
- Must integrate with airssys-rt actor system (sequential message processing)
- Prefer pure Rust to avoid C++ compilation complexity

---

## Decision

### Core Decision: NEAR-Style Key-Value API with Sled Backend

**We will implement a NEAR-inspired simple key-value storage API, using Sled as the default embedded database backend (with RocksDB as optional fallback), prefix-based component isolation, application-level quota tracking, and NO transactions (relying on actor model sequential processing).**

### Key Design Principles

1. **Simplicity First**: NEAR-style intuitive KV API (get, set, delete, prefix scan)
2. **Pure Rust Default**: Sled backend avoids C++ compilation pain
3. **Blockchain-Proven Patterns**: Learn from production-tested smart contract storage
4. **Actor Model Alignment**: Sequential message processing eliminates need for transactions
5. **Flexible Architecture**: Trait abstraction allows backend swapping
6. **Security Integration**: Capability system controls storage access

---

## Detailed Decisions

### Decision 1: Storage API Design - NEAR-Style Key-Value

**Decision:** Adopt NEAR Protocol's simple key-value storage API.

**API Design:**

```rust
/// Simple key-value storage API (inspired by NEAR Protocol)
pub trait StorageBackend: Send + Sync {
    /// Write key-value pair
    fn set(&self, key: &[u8], value: &[u8]) -> Result<()>;
    
    /// Read value by key
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>>;
    
    /// Delete key-value pair
    fn delete(&self, key: &[u8]) -> Result<bool>;
    
    /// Check if key exists
    fn has(&self, key: &[u8]) -> Result<bool>;
    
    /// Iterate over keys with prefix
    fn scan_prefix(&self, prefix: &[u8]) -> Result<Box<dyn Iterator<Item = Result<(Vec<u8>, Vec<u8>)>>>>;
    
    /// Get all keys with prefix (for quota calculation)
    fn keys_with_prefix(&self, prefix: &[u8]) -> Result<Vec<Vec<u8>>>;
    
    /// Flush to disk (durability)
    fn flush(&self) -> Result<()>;
}
```

**Component-Facing API:**

```rust
/// High-level storage API for components
pub struct ComponentStorage {
    backend: Arc<dyn StorageBackend>,
    component_id: ComponentId,
    quota: Arc<ComponentQuota>,
}

impl ComponentStorage {
    /// Set value (quota-checked)
    pub fn set(&self, key: &[u8], value: &[u8]) -> Result<()> {
        // Check capability permission
        self.check_capability(StorageOp::Write)?;
        
        // Check quota
        self.quota.check_space(value.len() as u64)?;
        
        // Build prefixed key
        let full_key = self.build_key(key);
        
        // Write to backend
        self.backend.set(&full_key, value)?;
        
        // Update quota
        self.quota.add_bytes(value.len() as u64);
        
        Ok(())
    }
    
    /// Get value
    pub fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        self.check_capability(StorageOp::Read)?;
        let full_key = self.build_key(key);
        self.backend.get(&full_key)
    }
    
    /// Delete value
    pub fn delete(&self, key: &[u8]) -> Result<bool> {
        self.check_capability(StorageOp::Write)?;
        let full_key = self.build_key(key);
        
        // Get old value size for quota
        if let Some(old_value) = self.backend.get(&full_key)? {
            self.backend.delete(&full_key)?;
            self.quota.remove_bytes(old_value.len() as u64);
            Ok(true)
        } else {
            Ok(false)
        }
    }
    
    /// Check if key exists
    pub fn has(&self, key: &[u8]) -> Result<bool> {
        self.check_capability(StorageOp::Read)?;
        let full_key = self.build_key(key);
        self.backend.has(&full_key)
    }
    
    /// Iterate over keys (component-scoped)
    pub fn keys(&self) -> Result<Vec<Vec<u8>>> {
        self.check_capability(StorageOp::Read)?;
        let prefix = self.component_prefix();
        
        let keys = self.backend.keys_with_prefix(&prefix)?;
        
        // Strip prefix from keys
        Ok(keys.into_iter()
            .map(|k| k[prefix.len()..].to_vec())
            .collect())
    }
    
    /// Scan with prefix (component-scoped)
    pub fn scan_prefix(&self, prefix: &[u8]) -> Result<StorageIterator> {
        self.check_capability(StorageOp::Read)?;
        let full_prefix = self.build_key(prefix);
        let iter = self.backend.scan_prefix(&full_prefix)?;
        Ok(StorageIterator::new(iter, self.component_prefix().len()))
    }
    
    fn build_key(&self, key: &[u8]) -> Vec<u8> {
        let mut full_key = self.component_prefix();
        full_key.extend_from_slice(key);
        full_key
    }
    
    fn component_prefix(&self) -> Vec<u8> {
        format!("component:{}:", self.component_id).into_bytes()
    }
}
```

**Usage Example (Rust Component):**

```rust
// Inside component code
impl MyComponent {
    fn save_config(&mut self, config: &Config) -> Result<()> {
        let json = serde_json::to_vec(config)?;
        self.storage.set(b"config", &json)?;
        Ok(())
    }
    
    fn load_config(&self) -> Result<Option<Config>> {
        if let Some(data) = self.storage.get(b"config")? {
            let config = serde_json::from_slice(&data)?;
            Ok(Some(config))
        } else {
            Ok(None)
        }
    }
    
    fn save_user(&mut self, user_id: &str, user: &User) -> Result<()> {
        let key = format!("user:{}", user_id);
        let data = bincode::serialize(user)?;
        self.storage.set(key.as_bytes(), &data)?;
        Ok(())
    }
    
    fn list_users(&self) -> Result<Vec<String>> {
        let mut users = Vec::new();
        for result in self.storage.scan_prefix(b"user:")? {
            let (key, _value) = result?;
            // key is "user:123", extract ID
            let key_str = String::from_utf8_lossy(&key);
            if let Some(id) = key_str.strip_prefix("user:") {
                users.push(id.to_string());
            }
        }
        Ok(users)
    }
}
```

**Rationale:**
- **Simplicity**: NEAR's API is intuitive and proven in production
- **Language-Agnostic**: Works seamlessly across Rust, JavaScript, Go, Python
- **Prefix Operations**: Perfect for component isolation and iteration
- **No Manual Serialization**: Components handle their own serialization
- **Blockchain-Proven**: NEAR processes millions of transactions with this API

**Alternatives Rejected:**

**Solana-Style Account-Based:**
```rust
// Too low-level, requires manual serialization
let account = Account { data: borsh::to_vec(&state)?, owner: component_id };
```
❌ Rejected: Manual serialization burden, blockchain-specific

**Custom Hierarchical:**
```rust
// Too complex, language-specific
let collection = storage.namespace("component").collection("users");
```
❌ Rejected: Complex API, hard to implement in non-Rust languages

---

### Decision 2: Primary Storage Backend - Sled (Default) + RocksDB (Optional)

**Decision:** Use Sled as default backend with RocksDB available via feature flag.

**Feature Flag Configuration:**

```toml
# Cargo.toml
[features]
default = ["storage-sled"]

# Storage backends (mutually exclusive at build time)
storage-sled = ["sled"]
storage-rocksdb = ["rocksdb"]

[dependencies]
sled = { version = "0.34", optional = true }
rocksdb = { version = "0.24", optional = true, default-features = false }
```

**Backend Abstraction:**

```rust
// Conditional compilation
#[cfg(feature = "storage-sled")]
pub type DefaultBackend = SledBackend;

#[cfg(feature = "storage-rocksdb")]
pub type DefaultBackend = RocksDBBackend;

// Factory function
pub fn create_default_backend(path: impl AsRef<Path>) -> Result<DefaultBackend> {
    #[cfg(feature = "storage-sled")]
    {
        SledBackend::open(path)
    }
    
    #[cfg(feature = "storage-rocksdb")]
    {
        RocksDBBackend::open(path)
    }
}
```

**Sled Backend Implementation:**

```rust
pub struct SledBackend {
    db: sled::Db,
}

impl SledBackend {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let db = sled::open(path)?;
        Ok(Self { db })
    }
}

impl StorageBackend for SledBackend {
    fn set(&self, key: &[u8], value: &[u8]) -> Result<()> {
        self.db.insert(key, value)?;
        Ok(())
    }
    
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        Ok(self.db.get(key)?.map(|v| v.to_vec()))
    }
    
    fn delete(&self, key: &[u8]) -> Result<bool> {
        Ok(self.db.remove(key)?.is_some())
    }
    
    fn has(&self, key: &[u8]) -> Result<bool> {
        Ok(self.db.contains_key(key)?)
    }
    
    fn scan_prefix(&self, prefix: &[u8]) -> Result<Box<dyn Iterator<Item = Result<(Vec<u8>, Vec<u8>)>>>> {
        let iter = self.db.scan_prefix(prefix)
            .map(|result| {
                result.map(|(k, v)| (k.to_vec(), v.to_vec()))
                    .map_err(|e| StorageError::Backend(e.to_string()))
            });
        Ok(Box::new(iter))
    }
    
    fn keys_with_prefix(&self, prefix: &[u8]) -> Result<Vec<Vec<u8>>> {
        let keys: Vec<_> = self.db.scan_prefix(prefix)
            .map(|r| r.map(|(k, _)| k.to_vec()))
            .collect::<Result<_, _>>()?;
        Ok(keys)
    }
    
    fn flush(&self) -> Result<()> {
        self.db.flush()?;
        Ok(())
    }
}
```

**RocksDB Backend Implementation:**

```rust
pub struct RocksDBBackend {
    db: rocksdb::DB,
}

impl RocksDBBackend {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let mut opts = rocksdb::Options::default();
        opts.create_if_missing(true);
        opts.set_compression_type(rocksdb::DBCompressionType::Lz4);
        
        let db = rocksdb::DB::open(&opts, path)?;
        Ok(Self { db })
    }
}

impl StorageBackend for RocksDBBackend {
    fn set(&self, key: &[u8], value: &[u8]) -> Result<()> {
        self.db.put(key, value)?;
        Ok(())
    }
    
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        Ok(self.db.get(key)?)
    }
    
    fn delete(&self, key: &[u8]) -> Result<bool> {
        self.db.delete(key)?;
        Ok(true)  // RocksDB doesn't return whether key existed
    }
    
    fn has(&self, key: &[u8]) -> Result<bool> {
        Ok(self.db.get(key)?.is_some())
    }
    
    fn scan_prefix(&self, prefix: &[u8]) -> Result<Box<dyn Iterator<Item = Result<(Vec<u8>, Vec<u8>)>>>> {
        let iter = self.db.prefix_iterator(prefix)
            .map(|result| {
                result.map(|(k, v)| (k.to_vec(), v.to_vec()))
                    .map_err(|e| StorageError::Backend(e.to_string()))
            });
        Ok(Box::new(iter))
    }
    
    fn keys_with_prefix(&self, prefix: &[u8]) -> Result<Vec<Vec<u8>>> {
        let keys: Vec<_> = self.db.prefix_iterator(prefix)
            .map(|r| r.map(|(k, _)| k.to_vec()))
            .collect::<Result<_, _>>()?;
        Ok(keys)
    }
    
    fn flush(&self) -> Result<()> {
        self.db.flush()?;
        Ok(())
    }
}
```

**Backend Comparison:**

| Factor | Sled (Default) | RocksDB (Optional) |
|--------|----------------|---------------------|
| **Language** | ✅ Pure Rust | ❌ C++ (via FFI) |
| **Build Time** | ✅ Fast (1-2 min) | ❌ Slow (8-10 min) |
| **Compilation** | ✅ Zero deps | ❌ C++ toolchain |
| **Stability** | ⚠️ Beta | ✅ Production (10+ years) |
| **Performance** | ⭐⭐⭐⭐ Good | ⭐⭐⭐⭐⭐ Excellent |
| **Space Efficiency** | ⚠️ 2x amplification | ✅ 1.2x amplification |
| **Debugging** | ✅ Pure Rust traces | ❌ FFI complexity |
| **Features** | ⭐⭐⭐⭐ Modern | ⭐⭐⭐⭐⭐ Extensive |

**Rationale:**

**Sled as Default:**
- ✅ **Pure Rust**: Zero C++ compilation pain (fast builds, easy CI/CD)
- ✅ **Timeline Alignment**: airssys-wasm planned for Q3 2026+ (sled will mature)
- ✅ **Modern API**: Async support, watch API, lock-free architecture
- ✅ **Developer Experience**: Fast iteration, clean debugging
- ✅ **Good Enough Performance**: Meets airssys-wasm requirements
- ⚠️ **Beta Status Acceptable**: New project starting 2026, beta concerns mitigated by:
  - Trait abstraction (can switch if needed)
  - Regular backups (export/import tool)
  - sled will be more mature by 2026

**RocksDB as Optional:**
- ✅ **Proven Stability**: 10+ years production, battle-tested
- ✅ **Critical Production**: Available for stability-critical deployments
- ✅ **Performance**: Slightly better for high-throughput workloads
- ✅ **User Choice**: Organizations can choose based on needs
- ❌ **C++ Burden**: Only for users who accept compilation complexity

**Decision Matrix:**

| Scenario | Recommendation |
|----------|----------------|
| **Development (2024-2026)** | ✅ Sled (default) |
| **Initial Production (2026+)** | ✅ Sled (likely stable) |
| **Critical Production** | ⚠️ RocksDB (opt-in) |
| **High Performance** | ⚠️ RocksDB (opt-in) |
| **Fast Prototyping** | ✅ Sled (quickest) |

**Alternatives Rejected:**

**RocksDB as Default:**
❌ Rejected: C++ compilation pain outweighs stability benefits for new project starting in 2026

**redb (Pure Rust Alternative):**
```rust
// Newer pure Rust database (1.0+ released)
```
❌ Rejected: Younger project, less ecosystem adoption, sled more mature

**SQLite:**
❌ Rejected: Relational overhead not needed for simple KV, SQL adds complexity

---

### Decision 3: Component Isolation Strategy - Prefix-Based Namespacing

**Decision:** Use prefix-based namespacing for component storage isolation.

**Prefix Format:**

```rust
// Component storage prefix
let prefix = format!("component:{}:", component_id);

// Example: component "my-app-v1.0"
// Prefix: "component:my-app-v1.0:"
// Keys:   "component:my-app-v1.0:config"
//         "component:my-app-v1.0:user:123"
//         "component:my-app-v1.0:cache:key"
```

**Implementation:**

```rust
pub struct ComponentStorage {
    backend: Arc<dyn StorageBackend>,
    component_id: ComponentId,
    prefix_cache: OnceCell<Vec<u8>>,  // Cache prefix bytes
}

impl ComponentStorage {
    fn component_prefix(&self) -> &[u8] {
        self.prefix_cache.get_or_init(|| {
            format!("component:{}:", self.component_id).into_bytes()
        })
    }
    
    fn build_key(&self, key: &[u8]) -> Vec<u8> {
        let prefix = self.component_prefix();
        let mut full_key = Vec::with_capacity(prefix.len() + key.len());
        full_key.extend_from_slice(prefix);
        full_key.extend_from_slice(key);
        full_key
    }
    
    fn strip_prefix(&self, full_key: &[u8]) -> &[u8] {
        let prefix = self.component_prefix();
        &full_key[prefix.len()..]
    }
    
    // All operations automatically scoped to component
    pub fn set(&self, key: &[u8], value: &[u8]) -> Result<()> {
        let full_key = self.build_key(key);
        self.backend.set(&full_key, value)
    }
    
    pub fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        let full_key = self.build_key(key);
        self.backend.get(&full_key)
    }
    
    pub fn scan_prefix(&self, user_prefix: &[u8]) -> Result<StorageIterator> {
        let full_prefix = self.build_key(user_prefix);
        let iter = self.backend.scan_prefix(&full_prefix)?;
        
        // Strip component prefix from returned keys
        let prefix_len = self.component_prefix().len();
        Ok(StorageIterator::new(iter, prefix_len))
    }
}

pub struct StorageIterator {
    inner: Box<dyn Iterator<Item = Result<(Vec<u8>, Vec<u8>)>>>,
    prefix_len: usize,
}

impl Iterator for StorageIterator {
    type Item = Result<(Vec<u8>, Vec<u8>)>;
    
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|result| {
            result.map(|(k, v)| {
                // Strip component prefix from key
                let user_key = k[self.prefix_len..].to_vec();
                (user_key, v)
            })
        })
    }
}
```

**Isolation Verification:**

```rust
#[test]
fn test_component_isolation() {
    let backend = SledBackend::open("/tmp/test-storage")?;
    
    let storage_a = ComponentStorage::new(backend.clone(), "component-a");
    let storage_b = ComponentStorage::new(backend.clone(), "component-b");
    
    // Component A writes data
    storage_a.set(b"secret", b"password123")?;
    
    // Component B cannot read Component A's data
    assert_eq!(storage_b.get(b"secret")?, None);
    
    // Component B can write same key (different namespace)
    storage_b.set(b"secret", b"different")?;
    
    // Both components see their own data
    assert_eq!(storage_a.get(b"secret")?, Some(b"password123".to_vec()));
    assert_eq!(storage_b.get(b"secret")?, Some(b"different".to_vec()));
    
    // Component A cannot see Component B's keys
    let keys_a = storage_a.keys()?;
    assert_eq!(keys_a, vec![b"secret".to_vec()]);
    
    let keys_b = storage_b.keys()?;
    assert_eq!(keys_b, vec![b"secret".to_vec()]);
}
```

**Rationale:**
- ✅ **Simple Implementation**: Just prefix all keys
- ✅ **Works with All Backends**: Sled, RocksDB, any KV store
- ✅ **No Overhead**: Minimal memory/performance cost
- ✅ **Efficient Prefix Scans**: Both Sled and RocksDB optimize prefix iteration
- ✅ **Easy Deletion**: Drop all component data by deleting prefix range

**Alternatives Rejected:**

**Column Families (RocksDB) / Trees (Sled):**
```rust
// Each component gets separate column family
let cf = db.cf_handle(&component_id)?;
db.put_cf(cf, b"key", b"value")?;
```
❌ Rejected: 
- Not portable (backend-specific)
- Limit on number of CFs/trees
- More complex management

**Separate Database Per Component:**
```rust
// Open separate DB per component
let db = Backend::open(format!("data/{}", component_id))?;
```
❌ Rejected:
- Resource overhead (file handles, memory)
- Doesn't scale to many components
- Complex management

---

### Decision 4: Quota Management - Application-Level Tracking

**Decision:** Track storage quotas at application level with real-time enforcement.

**Quota Structure:**

```rust
pub struct ComponentQuota {
    component_id: ComponentId,
    max_bytes: u64,
    current_bytes: AtomicU64,
    last_calculated: Mutex<Instant>,
}

impl ComponentQuota {
    pub fn new(component_id: ComponentId, max_bytes: u64) -> Self {
        Self {
            component_id,
            max_bytes,
            current_bytes: AtomicU64::new(0),
            last_calculated: Mutex::new(Instant::now()),
        }
    }
    
    /// Check if additional space is available
    pub fn check_space(&self, additional_bytes: u64) -> Result<()> {
        let current = self.current_bytes.load(Ordering::Relaxed);
        
        if current + additional_bytes > self.max_bytes {
            return Err(StorageError::QuotaExceeded {
                component: self.component_id.clone(),
                current_bytes: current,
                max_bytes: self.max_bytes,
                requested_bytes: additional_bytes,
            });
        }
        
        Ok(())
    }
    
    /// Add bytes to quota (after successful write)
    pub fn add_bytes(&self, bytes: u64) {
        self.current_bytes.fetch_add(bytes, Ordering::Relaxed);
    }
    
    /// Remove bytes from quota (after delete)
    pub fn remove_bytes(&self, bytes: u64) {
        self.current_bytes.fetch_sub(bytes, Ordering::Relaxed);
    }
    
    /// Get current usage
    pub fn current_usage(&self) -> u64 {
        self.current_bytes.load(Ordering::Relaxed)
    }
    
    /// Calculate actual usage from storage (periodic reconciliation)
    pub async fn recalculate(&self, storage: &ComponentStorage) -> Result<u64> {
        let mut total = 0u64;
        
        for result in storage.scan_prefix(b"")? {
            let (_key, value) = result?;
            total += value.len() as u64;
        }
        
        // Update tracked value
        self.current_bytes.store(total, Ordering::Relaxed);
        *self.last_calculated.lock() = Instant::now();
        
        Ok(total)
    }
}
```

**Integration with Storage Operations:**

```rust
impl ComponentStorage {
    pub fn set(&self, key: &[u8], value: &[u8]) -> Result<()> {
        // 1. Check capability permission
        self.check_capability(StorageOp::Write)?;
        
        // 2. Check quota BEFORE write
        self.quota.check_space(value.len() as u64)?;
        
        // 3. Get old value size (for quota adjustment)
        let full_key = self.build_key(key);
        let old_size = self.backend.get(&full_key)?
            .map(|v| v.len() as u64)
            .unwrap_or(0);
        
        // 4. Write to backend
        self.backend.set(&full_key, value)?;
        
        // 5. Update quota (net change)
        let new_size = value.len() as u64;
        if new_size > old_size {
            self.quota.add_bytes(new_size - old_size);
        } else if old_size > new_size {
            self.quota.remove_bytes(old_size - new_size);
        }
        
        Ok(())
    }
    
    pub fn delete(&self, key: &[u8]) -> Result<bool> {
        self.check_capability(StorageOp::Write)?;
        
        let full_key = self.build_key(key);
        
        // Get old value size
        if let Some(old_value) = self.backend.get(&full_key)? {
            self.backend.delete(&full_key)?;
            self.quota.remove_bytes(old_value.len() as u64);
            Ok(true)
        } else {
            Ok(false)
        }
    }
}
```

**Quota Configuration:**

```toml
# Component.toml
[storage]
max_size = "100MB"  # Maximum storage quota

# Host configuration (override component requests)
[host.quotas]
default_max = "50MB"
trusted_max = "500MB"

# Per-component overrides
[host.quotas.components]
"critical-app" = "1GB"
"tiny-app" = "10MB"
```

**Quota Monitoring:**

```rust
/// Periodic quota reconciliation (background task)
async fn reconcile_quotas(storage_manager: Arc<StorageManager>) {
    loop {
        tokio::time::sleep(Duration::from_secs(300)).await;  // Every 5 minutes
        
        for (component_id, storage) in storage_manager.all_storages() {
            match storage.quota.recalculate(&storage).await {
                Ok(actual_usage) => {
                    let tracked_usage = storage.quota.current_usage();
                    let diff = (actual_usage as i64 - tracked_usage as i64).abs();
                    
                    if diff > 1024 * 1024 {  // >1MB difference
                        warn!(
                            component = %component_id,
                            actual = actual_usage,
                            tracked = tracked_usage,
                            diff = diff,
                            "Quota drift detected, reconciled"
                        );
                    }
                }
                Err(e) => {
                    error!(component = %component_id, error = %e, "Quota recalculation failed");
                }
            }
        }
    }
}
```

**Rationale:**
- ✅ **Real-Time Enforcement**: Prevents quota violations before they happen
- ✅ **Accurate Tracking**: Maintains precise byte counts
- ✅ **Performance**: Atomic operations, minimal overhead
- ✅ **Reconciliation**: Periodic recalculation corrects any drift
- ✅ **Backend-Agnostic**: Works with any storage backend

**Alternatives Rejected:**

**Periodic Scanning Only:**
```rust
// Calculate usage periodically, no real-time tracking
async fn check_quotas() {
    let usage = calculate_storage_usage().await;
    if usage > quota { warn!("Over quota"); }
}
```
❌ Rejected: 
- Not real-time (component can exceed temporarily)
- Expensive to calculate frequently

**OS-Level Quotas:**
```rust
// Use filesystem quotas (Linux quota system)
```
❌ Rejected:
- Platform-specific (Linux only)
- Doesn't work with embedded KV stores

---

### Decision 5: Transaction Support - NOT Required (Sequential Processing Model)

**Decision:** Do NOT implement transaction support. Rely on actor model sequential message processing for consistency.

**Rationale from Blockchain Architecture:**

**EVM Sequential Processing Model:**
- Each transaction processed sequentially (one at a time)
- Transactions queued in mempool (memory buffer)
- No concurrent access to contract state
- Consistency guaranteed by sequential execution

**airssys-rt Actor Model Equivalent:**
- Each component is an actor
- Messages processed sequentially per actor
- Component mailbox acts as mempool
- No concurrent access to component storage within single component instance

**Implementation:**

```rust
// Component processes messages sequentially (actor model)
impl Actor for MyComponent {
    async fn handle_message(&mut self, msg: Message) -> Result<()> {
        // Sequential processing - guaranteed by actor system
        match msg {
            Message::Transfer { from, to, amount } => {
                // NO TRANSACTION NEEDED - sequential execution ensures consistency
                
                // Read current balances
                let from_balance: u64 = self.storage.get(format!("balance:{}", from).as_bytes())?
                    .and_then(|v| bincode::deserialize(&v).ok())
                    .unwrap_or(0);
                
                let to_balance: u64 = self.storage.get(format!("balance:{}", to).as_bytes())?
                    .and_then(|v| bincode::deserialize(&v).ok())
                    .unwrap_or(0);
                
                // Validate
                if from_balance < amount {
                    return Err(InsufficientBalance);
                }
                
                // Update balances (sequential writes, no race conditions)
                let new_from = from_balance - amount;
                let new_to = to_balance + amount;
                
                self.storage.set(
                    format!("balance:{}", from).as_bytes(),
                    &bincode::serialize(&new_from)?
                )?;
                
                self.storage.set(
                    format!("balance:{}", to).as_bytes(),
                    &bincode::serialize(&new_to)?
                )?;
                
                // ✅ Consistent - sequential execution, no interruption
                Ok(())
            }
        }
    }
}
```

**Actor System Guarantees:**

```rust
/// Actor system guarantees sequential message processing
/// Messages queued in mailbox, processed one at a time
/// 
/// Timeline:
/// T1: Message A arrives → Mailbox [A]
/// T2: Process A (storage operations sequential)
/// T3: Message B arrives → Mailbox [B]
/// T4: A completes
/// T5: Process B (storage operations sequential)
/// 
/// NO concurrent access to component storage
/// NO race conditions
/// NO need for transactions
```

**Consistency Guarantees:**

| Scenario | Traditional DB | Actor Model (airssys-wasm) |
|----------|----------------|----------------------------|
| **Concurrent Writes** | ❌ Race conditions → Need transactions | ✅ Sequential execution → No races |
| **Multi-Key Updates** | ❌ Partial updates → Need transactions | ✅ Message processed atomically |
| **Read-Modify-Write** | ❌ Lost updates → Need transactions | ✅ Sequential → No lost updates |
| **Crash Recovery** | ⚠️ WAL + transactions | ⚠️ Message replay (supervisor) |

**Crash Handling:**

```rust
// Supervisor handles component crashes
// Messages can be replayed if needed
impl Supervisor {
    async fn handle_component_crash(&self, component_id: ComponentId, msg: Message) {
        // Component crashed while processing message
        
        // Option 1: Discard message (at-most-once delivery)
        warn!("Component crashed, discarding message");
        
        // Option 2: Retry message (at-least-once delivery)
        warn!("Component crashed, retrying message");
        self.restart_component_and_retry(component_id, msg).await;
        
        // Option 3: Dead letter queue
        self.send_to_dead_letter_queue(component_id, msg);
    }
}

// Components must implement idempotent operations for at-least-once
impl MyComponent {
    async fn handle_message(&mut self, msg: Message) -> Result<()> {
        match msg {
            Message::UpdateCounter { id, value } => {
                // Idempotent - can be retried safely
                self.storage.set(format!("counter:{}", id).as_bytes(), &value.to_le_bytes())?;
                Ok(())
            }
            
            Message::IncrementCounter { id } => {
                // NOT idempotent - need request deduplication if at-least-once
                let current: u64 = self.storage.get(format!("counter:{}", id).as_bytes())?
                    .and_then(|v| v.try_into().ok().map(u64::from_le_bytes))
                    .unwrap_or(0);
                
                self.storage.set(format!("counter:{}", id).as_bytes(), &(current + 1).to_le_bytes())?;
                Ok(())
            }
        }
    }
}
```

**When Transactions Would Be Needed:**

```rust
// Scenario: Multi-component coordination (future enhancement)
// 
// Component A needs to atomically update Component B's storage
// This is NOT POSSIBLE in actor model (by design - isolation)
// 
// Solution: Use message-passing coordination protocol
async fn transfer_across_components() {
    // Two-phase commit via messages
    
    // Phase 1: Prepare
    let prepare_a = component_a.send(Message::PrepareTransfer { amount: 100 }).await?;
    let prepare_b = component_b.send(Message::PrepareReceive { amount: 100 }).await?;
    
    // Phase 2: Commit or Abort
    if prepare_a && prepare_b {
        component_a.send(Message::CommitTransfer).await?;
        component_b.send(Message::CommitReceive).await?;
    } else {
        component_a.send(Message::AbortTransfer).await?;
        component_b.send(Message::AbortReceive).await?;
    }
}
```

**Rationale:**
- ✅ **Actor Model Sufficient**: Sequential processing eliminates need for transactions within component
- ✅ **Simpler Implementation**: No transaction overhead
- ✅ **Better Performance**: No locking, no transaction commit overhead
- ✅ **Blockchain-Proven**: EVM, Solana, NEAR all use sequential execution
- ✅ **YAGNI**: Don't add complexity until proven need
- ⚠️ **Future Enhancement**: Can add transactions later if multi-component coordination needed

**Alternatives Rejected:**

**Transactions Required:**
```rust
storage.transaction(|txn| {
    txn.set(b"key1", b"value1")?;
    txn.set(b"key2", b"value2")?;
    Ok(())
})?;
```
❌ Rejected:
- Unnecessary complexity (actor model provides sequential execution)
- Performance overhead (locking, commit)
- Not needed for single-component operations

**Transactions Optional:**
```rust
#[cfg(feature = "transactions")]
storage.transaction(...)?;
```
❌ Rejected:
- Inconsistent behavior across builds
- Developers confused when available vs not

---

### Decision 6: Storage Migration Strategy - Export/Import Tool

**Decision:** Provide export/import CLI tool for backend migration and backups.

**Export Format (JSON Lines):**

```jsonl
{"key":"Y29tcG9uZW50Om15LWFwcDpjb25maWc=","value":"eyJzZXR0aW5nIjoidmFsdWUifQ==","component":"my-app","timestamp":"2025-10-19T12:00:00Z"}
{"key":"Y29tcG9uZW50Om15LWFwcDp1c2VyOjEyMw==","value":"eyJuYW1lIjoiQWxpY2UifQ==","component":"my-app","timestamp":"2025-10-19T12:00:00Z"}
```

**CLI Commands:**

```bash
# Export all data (JSON Lines format)
$ airssys-wasm storage export > backup.jsonl
Exporting storage...
Component: my-app (125 keys, 5.2 MB)
Component: worker (48 keys, 1.1 MB)
Exported 173 keys, 6.3 MB total

# Export specific component
$ airssys-wasm storage export --component my-app > my-app-backup.jsonl

# Export with compression
$ airssys-wasm storage export | gzip > backup.jsonl.gz

# Import to same backend
$ airssys-wasm storage import < backup.jsonl
Importing storage...
Imported 173 keys, 6.3 MB total

# Import to different backend (migration)
$ cargo build --release --features storage-rocksdb
$ airssys-wasm storage import --backend rocksdb < backup.jsonl
Migrating to RocksDB backend...
Imported 173 keys, 6.3 MB total
Migration complete!

# List component storage usage
$ airssys-wasm storage list
Component          Keys    Size     Quota    Usage
─────────────────────────────────────────────────
my-app             125     5.2 MB   50 MB    10%
worker             48      1.1 MB   50 MB    2%
cache-service      892     45.8 MB  50 MB    92%  ⚠️
─────────────────────────────────────────────────
Total              1065    52.1 MB  150 MB   35%

# Delete component storage
$ airssys-wasm storage delete --component my-app
⚠️  This will permanently delete all storage for component 'my-app'
Continue? [y/N] y
Deleted 125 keys, 5.2 MB freed

# Verify storage integrity
$ airssys-wasm storage verify
Verifying storage integrity...
✅ All keys accessible
✅ No corruption detected
✅ Quota tracking accurate
Storage integrity OK
```

**Implementation:**

```rust
/// Export storage to JSON Lines format
pub async fn export_storage(
    storage_manager: &StorageManager,
    output: &mut dyn Write,
) -> Result<ExportStats> {
    let mut stats = ExportStats::default();
    
    for (component_id, storage) in storage_manager.all_storages() {
        info!("Exporting component: {}", component_id);
        
        for result in storage.scan_prefix(b"")? {
            let (key, value) = result?;
            
            let entry = ExportEntry {
                key: base64::encode(&key),
                value: base64::encode(&value),
                component: component_id.clone(),
                timestamp: Utc::now(),
            };
            
            serde_json::to_writer(&mut *output, &entry)?;
            writeln!(output)?;
            
            stats.keys += 1;
            stats.bytes += key.len() + value.len();
        }
        
        stats.components += 1;
    }
    
    Ok(stats)
}

/// Import storage from JSON Lines format
pub async fn import_storage(
    storage_manager: &StorageManager,
    input: &mut dyn BufRead,
) -> Result<ImportStats> {
    let mut stats = ImportStats::default();
    
    for line in input.lines() {
        let line = line?;
        let entry: ExportEntry = serde_json::from_str(&line)?;
        
        let key = base64::decode(&entry.key)?;
        let value = base64::decode(&entry.value)?;
        
        let storage = storage_manager.get_or_create(&entry.component)?;
        storage.set(&key, &value)?;
        
        stats.keys += 1;
        stats.bytes += key.len() + value.len();
    }
    
    Ok(stats)
}

#[derive(Serialize, Deserialize)]
struct ExportEntry {
    key: String,       // Base64-encoded
    value: String,     // Base64-encoded
    component: String,
    timestamp: DateTime<Utc>,
}
```

**Migration Workflow:**

```bash
# Step 1: Export from Sled
$ cargo build --release --features storage-sled
$ ./target/release/airssys-wasm storage export > backup.jsonl

# Step 2: Rebuild with RocksDB
$ cargo clean
$ cargo build --release --features storage-rocksdb

# Step 3: Import to RocksDB
$ ./target/release/airssys-wasm storage import < backup.jsonl

# Step 4: Verify
$ ./target/release/airssys-wasm storage verify
```

**Rationale:**
- ✅ **Clear Migration Path**: Easy switch between backends
- ✅ **Human-Readable**: JSON Lines can be inspected/modified
- ✅ **Backup/Restore**: Same tool for both use cases
- ✅ **Component Portability**: Can move components between hosts
- ✅ **Disaster Recovery**: Regular exports protect against corruption

---

## Consequences

### Positive Consequences

✅ **Simple, Intuitive API**
- NEAR-style KV API easy to learn
- Language-agnostic (Rust, JS, Go, Python)
- Minimal boilerplate for common operations
- Clear error messages

✅ **Pure Rust Benefits (Sled Default)**
- Fast compilation (1-2 minutes vs 8-10)
- Clean debugging (no FFI complexity)
- Easy CI/CD setup
- Cross-compilation simplicity

✅ **Production Escape Hatch (RocksDB Optional)**
- Battle-tested backend available if needed
- Feature flag provides user choice
- Migration path via export/import tool
- Critical deployments can use proven technology

✅ **Component Isolation**
- Prefix-based namespacing simple and effective
- No data leaks between components
- Efficient prefix scanning
- Works with any backend

✅ **Real-Time Quota Enforcement**
- Prevents storage exhaustion before it happens
- Precise tracking with periodic reconciliation
- Clear error messages when quota exceeded
- Per-component limits configurable

✅ **No Transaction Overhead**
- Actor model sequential processing ensures consistency
- Simpler implementation (no locking)
- Better performance (no commit overhead)
- Blockchain-proven pattern (EVM, Solana, NEAR)

✅ **Migration and Backup Tools**
- Export/import for backend migration
- JSON Lines format human-readable
- Regular backups protect data
- Component portability across hosts

### Negative Consequences

⚠️ **Sled Beta Status**
- **Issue**: Sled explicitly labeled beta software
- **Impact**: Less proven than RocksDB in production
- **Mitigation**:
  - Timeline alignment (airssys-wasm Q3 2026+, sled will mature)
  - Trait abstraction allows switching to RocksDB
  - Regular backups via export tool
  - RocksDB available via feature flag

⚠️ **No Transactions**
- **Issue**: Components cannot perform atomic multi-key operations
- **Impact**: Multi-component coordination requires message-passing protocols
- **Mitigation**:
  - Actor model sequential processing sufficient for most use cases
  - Can add transactions in future if proven need (YAGNI)
  - Idempotent operations pattern for at-least-once delivery

⚠️ **Prefix Overhead**
- **Issue**: All keys include component ID prefix (~20-30 bytes)
- **Impact**: Slightly larger key storage
- **Mitigation**:
  - Negligible overhead (<1% of total storage)
  - Simplicity benefits outweigh small size cost

⚠️ **Application-Level Quota Tracking**
- **Issue**: Need to track sizes accurately on every operation
- **Impact**: Small overhead per write/delete
- **Mitigation**:
  - Atomic operations very fast
  - Periodic reconciliation corrects drift
  - Essential for preventing storage exhaustion

⚠️ **Backend Feature Flag Complexity**
- **Issue**: Maintain two backend implementations
- **Impact**: 2x testing matrix, conditional compilation
- **Mitigation**:
  - Trait abstraction keeps backends isolated
  - Most users use default (sled)
  - RocksDB only for critical production needs

### Neutral Consequences

📝 **Storage Format Opaque to Components**
- Components see simple KV API
- Backend implementation hidden
- Can optimize backend without affecting components
- Trade-off: Less control, more abstraction

📝 **JSON Lines Export Format**
- Human-readable and editable
- Larger than binary format
- Acceptable trade-off for migration/backup use case

---

## Implementation Guidance

### Phase 1: Storage Backend Abstraction (Weeks 1-2)

**Week 1: Trait and Core Types**
```rust
// src/storage/mod.rs
pub mod backend;      // StorageBackend trait
pub mod sled_impl;    // Sled implementation
pub mod rocksdb_impl; // RocksDB implementation
pub mod quota;        // Quota tracking
pub mod error;        // Error types

// Core trait
pub trait StorageBackend: Send + Sync {
    fn set(&self, key: &[u8], value: &[u8]) -> Result<()>;
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>>;
    fn delete(&self, key: &[u8]) -> Result<bool>;
    fn has(&self, key: &[u8]) -> Result<bool>;
    fn scan_prefix(&self, prefix: &[u8]) -> Result<Box<dyn Iterator<Item = Result<(Vec<u8>, Vec<u8>)>>>>;
    fn keys_with_prefix(&self, prefix: &[u8]) -> Result<Vec<Vec<u8>>>;
    fn flush(&self) -> Result<()>;
}
```

**Week 2: Backend Implementations**
```rust
// Sled backend (default feature)
#[cfg(feature = "storage-sled")]
pub struct SledBackend { db: sled::Db }

// RocksDB backend (optional feature)
#[cfg(feature = "storage-rocksdb")]
pub struct RocksDBBackend { db: rocksdb::DB }
```

### Phase 2: Component Storage API (Week 3)

```rust
// src/storage/component.rs
pub struct ComponentStorage {
    backend: Arc<dyn StorageBackend>,
    component_id: ComponentId,
    quota: Arc<ComponentQuota>,
    capability_checker: Arc<CapabilityChecker>,
}

impl ComponentStorage {
    // High-level API with prefix scoping
    pub fn set(&self, key: &[u8], value: &[u8]) -> Result<()>;
    pub fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>>;
    pub fn delete(&self, key: &[u8]) -> Result<bool>;
    pub fn has(&self, key: &[u8]) -> Result<bool>;
    pub fn keys(&self) -> Result<Vec<Vec<u8>>>;
    pub fn scan_prefix(&self, prefix: &[u8]) -> Result<StorageIterator>;
}
```

### Phase 3: Quota Management (Week 4)

```rust
// Real-time quota tracking
pub struct ComponentQuota {
    component_id: ComponentId,
    max_bytes: u64,
    current_bytes: AtomicU64,
}

impl ComponentQuota {
    pub fn check_space(&self, additional_bytes: u64) -> Result<()>;
    pub fn add_bytes(&self, bytes: u64);
    pub fn remove_bytes(&self, bytes: u64);
    pub async fn recalculate(&self, storage: &ComponentStorage) -> Result<u64>;
}
```

### Phase 4: Export/Import Tools (Week 5)

```bash
# CLI commands
airssys-wasm storage export [--component <id>] > backup.jsonl
airssys-wasm storage import [--backend sled|rocksdb] < backup.jsonl
airssys-wasm storage list
airssys-wasm storage verify
airssys-wasm storage delete --component <id>
```

### Phase 5: Integration with Component Runtime (Week 6)

```rust
// Component gets storage handle
impl Component {
    fn storage(&self) -> &ComponentStorage {
        &self.storage
    }
}

// Host function integration
pub fn host_storage_set(caller: Caller, key: Vec<u8>, value: Vec<u8>) -> Result<()> {
    let storage = caller.data().storage();
    storage.set(&key, &value)
}
```

### Testing Strategy

**Unit Tests:**
- Storage backend implementations (sled, rocksdb)
- Prefix-based isolation
- Quota tracking accuracy
- Error handling

**Integration Tests:**
- Component storage operations end-to-end
- Quota enforcement
- Export/import functionality
- Backend migration

**Performance Tests:**
- Read/write latency (<1ms / <10ms targets)
- Prefix scan performance
- Quota tracking overhead
- Concurrent access (multiple components)

**Security Tests:**
- Component isolation (cannot access other component data)
- Quota enforcement (cannot exceed limits)
- Capability integration (permissions checked)

---

## Future Enhancements

### Phase 2: Advanced Features (Future)

**Compression Support:**
```toml
[storage]
compression = "zstd"  # Compress values >1KB
compression_level = 3
```

**TTL Support:**
```rust
storage.set_with_ttl(b"session:123", b"data", Duration::from_secs(3600))?;
// Automatically expires after 1 hour
```

**Storage Snapshots:**
```bash
# Create snapshot
$ airssys-wasm storage snapshot create my-snapshot

# Restore from snapshot
$ airssys-wasm storage snapshot restore my-snapshot
```

**Storage Metrics:**
```rust
pub struct StorageMetrics {
    pub reads_per_sec: f64,
    pub writes_per_sec: f64,
    pub read_latency_p50: Duration,
    pub read_latency_p99: Duration,
    pub cache_hit_rate: f64,
}
```

### Phase 3: Optional Transactions (If Needed)

```rust
// Optional transaction support (future)
#[cfg(feature = "storage-transactions")]
storage.transaction(|txn| {
    txn.set(b"key1", b"value1")?;
    txn.set(b"key2", b"value2")?;
    Ok(())
})?;
```

---

## References

### Related ADRs
- **ADR-WASM-002**: WASM Runtime Engine Selection (Wasmtime, async-first)
- **ADR-WASM-005**: Capability-Based Security Model (storage permissions)

### Related Knowledge
- **KNOWLEDGE-WASM-007**: Component Storage Architecture (blockchain storage models)
- **KNOWLEDGE-WASM-008**: Storage Backend Comparison (Sled vs RocksDB detailed analysis)

### Storage Backend References
- **Sled**: https://github.com/spacejam/sled
- **RocksDB**: https://rocksdb.org/
- **rust-rocksdb**: https://github.com/rust-rocksdb/rust-rocksdb

### Blockchain Storage References
- **NEAR Storage**: https://docs.near.org/concepts/storage/storage-staking
- **Solana Accounts**: https://docs.solana.com/developing/programming-model/accounts
- **EVM Storage**: https://ethereum.org/en/developers/docs/smart-contracts/anatomy/

---

## Decision Log

| Date | Decision | Participants |
|------|----------|--------------|
| 2025-10-19 | Storage API: NEAR-style KV | Architecture Team |
| 2025-10-19 | Backend: Sled default, RocksDB optional | Architecture Team |
| 2025-10-19 | Isolation: Prefix-based namespacing | Architecture Team |
| 2025-10-19 | Quota: Application-level tracking | Architecture Team |
| 2025-10-19 | Transactions: NOT required (actor model) | Architecture Team |
| 2025-10-19 | Migration: Export/import JSON Lines tool | Architecture Team |

---

**Status:** ✅ **Accepted**  
**Implementation Priority:** Critical (Phase 1 Foundation)  
**Next Review:** After Phase 1 implementation or if performance issues identified

---

## Appendix: Complete Storage Example

### Example Component Using Storage

```rust
use airssys_wasm::{Component, ComponentStorage, Message, Result};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct User {
    id: String,
    name: String,
    email: String,
    created_at: i64,
}

#[derive(Serialize, Deserialize)]
struct Config {
    max_users: u32,
    admin_email: String,
}

pub struct UserManagerComponent {
    storage: ComponentStorage,
}

impl UserManagerComponent {
    // Save configuration
    async fn save_config(&self, config: &Config) -> Result<()> {
        let data = serde_json::to_vec(config)?;
        self.storage.set(b"config", &data)?;
        Ok(())
    }
    
    // Load configuration
    async fn load_config(&self) -> Result<Option<Config>> {
        if let Some(data) = self.storage.get(b"config")? {
            let config = serde_json::from_slice(&data)?;
            Ok(Some(config))
        } else {
            Ok(None)
        }
    }
    
    // Save user
    async fn save_user(&self, user: &User) -> Result<()> {
        let key = format!("user:{}", user.id);
        let data = bincode::serialize(user)?;
        self.storage.set(key.as_bytes(), &data)?;
        Ok(())
    }
    
    // Get user
    async fn get_user(&self, user_id: &str) -> Result<Option<User>> {
        let key = format!("user:{}", user_id);
        if let Some(data) = self.storage.get(key.as_bytes())? {
            let user = bincode::deserialize(&data)?;
            Ok(Some(user))
        } else {
            Ok(None)
        }
    }
    
    // Delete user
    async fn delete_user(&self, user_id: &str) -> Result<bool> {
        let key = format!("user:{}", user_id);
        Ok(self.storage.delete(key.as_bytes())?)
    }
    
    // List all users
    async fn list_users(&self) -> Result<Vec<User>> {
        let mut users = Vec::new();
        
        for result in self.storage.scan_prefix(b"user:")? {
            let (_key, value) = result?;
            let user: User = bincode::deserialize(&value)?;
            users.push(user);
        }
        
        users.sort_by(|a, b| a.created_at.cmp(&b.created_at));
        Ok(users)
    }
    
    // Count users
    async fn count_users(&self) -> Result<usize> {
        let keys = self.storage.keys()?;
        let user_keys = keys.into_iter()
            .filter(|k| k.starts_with(b"user:"))
            .count();
        Ok(user_keys)
    }
}

impl Component for UserManagerComponent {
    async fn handle_message(&mut self, msg: Message) -> Result<()> {
        // Sequential processing - no transactions needed
        match msg {
            Message::CreateUser { id, name, email } => {
                // Check if exists
                if self.get_user(&id).await?.is_some() {
                    return Err(UserAlreadyExists(id));
                }
                
                // Create user
                let user = User {
                    id: id.clone(),
                    name,
                    email,
                    created_at: current_timestamp(),
                };
                
                self.save_user(&user).await?;
                Ok(())
            }
            
            Message::UpdateUser { id, name, email } => {
                // Get existing user
                let mut user = self.get_user(&id).await?
                    .ok_or(UserNotFound(id.clone()))?;
                
                // Update fields
                user.name = name;
                user.email = email;
                
                // Save (overwrites old data)
                self.save_user(&user).await?;
                Ok(())
            }
            
            Message::DeleteUser { id } => {
                if !self.delete_user(&id).await? {
                    return Err(UserNotFound(id));
                }
                Ok(())
            }
            
            Message::ListUsers => {
                let users = self.list_users().await?;
                // Send response...
                Ok(())
            }
        }
    }
}
```

### Example Host Configuration

```toml
# host-config.toml

[storage]
backend = "sled"  # or "rocksdb"
data_dir = "/var/lib/airssys-wasm/storage"

[storage.quotas]
default_max = "50MB"
trusted_max = "500MB"

# Per-component overrides
[storage.quotas.components]
"user-manager" = "100MB"
"cache-service" = "1GB"
"tiny-app" = "10MB"

[storage.backups]
enabled = true
interval = "1h"
retention = "7d"
path = "/var/backups/airssys-wasm"
```
