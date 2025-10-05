//! Lock-free actor registry with pre-computed routing keys.
//!
//! Provides O(1) actor address resolution using DashMap for concurrent access
//! without locks. Supports actor pools with configurable routing strategies.

// Layer 1: Standard library imports
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::sync::Arc;

// Layer 2: Third-party crate imports
use dashmap::DashMap;
use rand::Rng;

// Layer 3: Internal module imports
use super::error::BrokerError;
use crate::mailbox::MailboxSender;
use crate::message::Message;
use crate::util::ActorAddress;

/// Pool routing strategy for load balancing across actor pools.
///
/// Defines how messages are distributed among actors in a pool for
/// workload balancing and fault tolerance.
///
/// # Strategies
///
/// - **RoundRobin**: Sequential distribution for even load distribution
/// - **Random**: Uniform random selection for simple load balancing
///
/// # Future Strategies (Deferred - YAGNI §6.1)
///
/// - **LeastLoaded**: Requires metrics integration (RT-TASK-008)
/// - **Custom**: Application-specific routing logic
///
/// # Example
///
/// ```rust
/// use airssys_rt::broker::PoolStrategy;
///
/// let strategy = PoolStrategy::RoundRobin;
/// assert_eq!(strategy, PoolStrategy::RoundRobin);
///
/// let random_strategy = PoolStrategy::Random;
/// assert_eq!(random_strategy, PoolStrategy::Random);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PoolStrategy {
    /// Round-robin selection (sequential, predictable).
    ///
    /// Distributes messages sequentially across pool members.
    /// Best for even load distribution with predictable ordering.
    RoundRobin,

    /// Random selection (uniform distribution, non-deterministic).
    ///
    /// Selects pool members randomly with uniform distribution.
    /// Best for simple load balancing without state tracking.
    Random,
    // Future: LeastLoaded - requires metrics (deferred to RT-TASK-008)
}

/// Lock-free actor registry with pre-computed routing keys.
///
/// Provides O(1) actor address resolution using DashMap for concurrent
/// access without locks. Designed for high-throughput message routing
/// with support for 10,000+ concurrent actors.
///
/// # Performance Characteristics
///
/// - **Concurrent Access**: Lock-free reads/writes with DashMap
/// - **Lookup Complexity**: O(1) average case for address resolution
/// - **Cache-Friendly**: Pre-computed routing keys eliminate hash recomputation
/// - **Scalability**: Designed and tested for 10,000+ registered actors
/// - **Memory Efficiency**: Minimal overhead per registered actor
///
/// # Architecture
///
/// The registry maintains three concurrent data structures:
/// - **Routing Table**: ActorAddress → MailboxSender mapping
/// - **Routing Keys**: Pre-computed hash → ActorAddress cache
/// - **Actor Pools**: Pool name → [ActorAddress] collections
///
/// # Example (System-Level Usage)
///
/// ```ignore
/// use airssys_rt::broker::{ActorRegistry, PoolStrategy};
///
/// let registry = ActorRegistry::new();
///
/// // Register actor
/// registry.register(address, mailbox_sender)?;
///
/// // Resolve actor
/// let sender = registry.resolve(&address)?;
///
/// // Pool routing
/// let pool_member = registry.get_pool_member("workers", PoolStrategy::RoundRobin)?;
/// ```
pub struct ActorRegistry<M: Message, S: MailboxSender<M>> {
    /// Primary routing table: address → sender
    routing_table: Arc<DashMap<ActorAddress, S>>,

    /// Pre-computed routing keys for fast lookup: hash → address
    routing_keys: Arc<DashMap<u64, ActorAddress>>,

    /// Actor pools: pool_name → [addresses]
    pools: Arc<DashMap<String, Vec<ActorAddress>>>,

    /// Round-robin counters for pool strategies: pool_name → counter
    pool_counters: Arc<DashMap<String, usize>>,

    /// Phantom data for message type (zero-sized marker)
    _phantom: PhantomData<M>,
}

impl<M: Message, S: MailboxSender<M>> ActorRegistry<M, S> {
    /// Create a new empty actor registry.
    ///
    /// Initializes all internal data structures with zero capacity.
    /// Memory is allocated lazily as actors are registered.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let registry = ActorRegistry::<MyMessage, MyMailboxSender>::new();
    /// ```
    pub fn new() -> Self {
        Self {
            routing_table: Arc::new(DashMap::new()),
            routing_keys: Arc::new(DashMap::new()),
            pools: Arc::new(DashMap::new()),
            pool_counters: Arc::new(DashMap::new()),
            _phantom: PhantomData,
        }
    }

    /// Register an actor with the given address and mailbox sender.
    ///
    /// Makes the actor addressable for message routing. If the address
    /// is already registered, returns an error without modifying state.
    ///
    /// # Arguments
    ///
    /// * `address` - Unique actor address (Id, Named, Pool variants)
    /// * `sender` - Mailbox sender for message delivery
    ///
    /// # Errors
    ///
    /// Returns `BrokerError::DuplicateRegistration` if the address is
    /// already registered in the routing table.
    ///
    /// # Example
    ///
    /// ```ignore
    /// registry.register(
    ///     ActorAddress::Named { id, name: "worker-1".to_string() },
    ///     mailbox_sender,
    /// )?;
    /// ```
    pub fn register(&self, address: ActorAddress, sender: S) -> Result<(), BrokerError> {
        // Check for duplicate registration
        if self.routing_table.contains_key(&address) {
            return Err(BrokerError::DuplicateRegistration(address));
        }

        // Compute routing key once
        let routing_key = Self::compute_routing_key(&address);

        // Insert into routing table
        self.routing_table.insert(address.clone(), sender);

        // Cache routing key for fast lookup
        self.routing_keys.insert(routing_key, address.clone());

        // Register in pool if applicable
        if let ActorAddress::Named { ref name, .. } = address {
            // Check if this is a pool member (e.g., "pool-name:member-1")
            if let Some(pool_name) = name.split(':').next() {
                if name.contains(':') {
                    // This is a pool member
                    self.pools
                        .entry(pool_name.to_string())
                        .or_default()
                        .push(address);
                }
            }
        }

        Ok(())
    }

    /// Unregister an actor by address.
    ///
    /// Removes the actor from all registry data structures including
    /// the routing table, routing key cache, and any actor pools.
    ///
    /// # Arguments
    ///
    /// * `address` - Actor address to unregister
    ///
    /// # Errors
    ///
    /// Returns `BrokerError::ActorNotFound` if the address is not
    /// currently registered.
    ///
    /// # Example
    ///
    /// ```ignore
    /// registry.unregister(&address)?;
    /// ```
    pub fn unregister(&self, address: &ActorAddress) -> Result<(), BrokerError> {
        // Remove from routing table
        if self.routing_table.remove(address).is_none() {
            return Err(BrokerError::ActorNotFound(address.clone()));
        }

        // Remove routing key
        let routing_key = Self::compute_routing_key(address);
        self.routing_keys.remove(&routing_key);

        // Remove from pool if applicable
        if let ActorAddress::Named { ref name, .. } = address {
            if let Some(pool_name) = name.split(':').next() {
                if name.contains(':') {
                    if let Some(mut pool) = self.pools.get_mut(pool_name) {
                        pool.retain(|addr| addr != address);
                    }
                }
            }
        }

        Ok(())
    }

    /// Resolve an actor address to its mailbox sender.
    ///
    /// Performs O(1) lookup in the routing table to find the mailbox
    /// sender for the given address.
    ///
    /// # Arguments
    ///
    /// * `address` - Actor address to resolve
    ///
    /// # Returns
    ///
    /// The mailbox sender if found, cloned for use by the caller.
    ///
    /// # Errors
    ///
    /// Returns `BrokerError::ActorNotFound` if the address is not
    /// registered in the routing table.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let sender = registry.resolve(&address)?;
    /// sender.send(envelope).await?;
    /// ```
    pub fn resolve(&self, address: &ActorAddress) -> Result<S, BrokerError> {
        self.routing_table
            .get(address)
            .map(|entry| entry.value().clone())
            .ok_or_else(|| BrokerError::ActorNotFound(address.clone()))
    }

    /// Resolve by pre-computed routing key (fast path).
    ///
    /// Uses cached routing key to avoid hash recomputation. This is
    /// the fastest resolution path when routing key is known.
    ///
    /// # Arguments
    ///
    /// * `key` - Pre-computed routing key (hash of ActorAddress)
    ///
    /// # Returns
    ///
    /// The mailbox sender if found, or None if the key is not cached.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let key = compute_routing_key(&address);
    /// if let Some(sender) = registry.resolve_by_routing_key(key) {
    ///     sender.send(envelope).await?;
    /// }
    /// ```
    pub fn resolve_by_routing_key(&self, key: u64) -> Option<S> {
        self.routing_keys.get(&key).and_then(|entry| {
            let address = entry.value();
            self.routing_table.get(address).map(|s| s.value().clone())
        })
    }

    /// Get an actor address from a pool using the specified strategy.
    ///
    /// Selects one actor from the named pool according to the routing
    /// strategy (RoundRobin or Random).
    ///
    /// # Arguments
    ///
    /// * `pool_name` - Name of the actor pool
    /// * `strategy` - Routing strategy to use for selection
    ///
    /// # Returns
    ///
    /// An actor address from the pool, or None if the pool doesn't
    /// exist or is empty.
    ///
    /// # Example
    ///
    /// ```ignore
    /// if let Some(worker) = registry.get_pool_member("workers", PoolStrategy::RoundRobin) {
    ///     let sender = registry.resolve(&worker)?;
    ///     sender.send(envelope).await?;
    /// }
    /// ```
    pub fn get_pool_member(&self, pool_name: &str, strategy: PoolStrategy) -> Option<ActorAddress> {
        let pool = self.pools.get(pool_name)?;
        if pool.is_empty() {
            return None;
        }

        match strategy {
            PoolStrategy::RoundRobin => {
                let mut counter = self.pool_counters.entry(pool_name.to_string()).or_insert(0);
                let index = *counter % pool.len();
                *counter = counter.wrapping_add(1);
                Some(pool[index].clone())
            }
            PoolStrategy::Random => {
                let index = rand::thread_rng().gen_range(0..pool.len());
                Some(pool[index].clone())
            }
        }
    }

    /// Compute routing key from address (for pre-computation and caching).
    ///
    /// Uses DefaultHasher to compute a stable hash of the address.
    /// This hash is cached to avoid recomputation during message routing.
    ///
    /// # Arguments
    ///
    /// * `address` - Actor address to hash
    ///
    /// # Returns
    ///
    /// 64-bit hash value suitable for use as routing key
    fn compute_routing_key(address: &ActorAddress) -> u64 {
        let mut hasher = DefaultHasher::new();
        address.hash(&mut hasher);
        hasher.finish()
    }

    /// Get the number of registered actors.
    ///
    /// Returns the total count of actors currently registered in the
    /// routing table.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let count = registry.actor_count();
    /// println!("Registry has {} actors", count);
    /// ```
    pub fn actor_count(&self) -> usize {
        self.routing_table.len()
    }

    /// Get the number of actor pools.
    ///
    /// Returns the total count of named pools currently managed.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let pool_count = registry.pool_count();
    /// println!("Registry has {} pools", pool_count);
    /// ```
    pub fn pool_count(&self) -> usize {
        self.pools.len()
    }

    /// Get the size of a specific pool.
    ///
    /// Returns the number of actors in the named pool, or None if
    /// the pool doesn't exist.
    ///
    /// # Arguments
    ///
    /// * `pool_name` - Name of the pool to query
    ///
    /// # Example
    ///
    /// ```ignore
    /// if let Some(size) = registry.pool_size("workers") {
    ///     println!("Worker pool has {} actors", size);
    /// }
    /// ```
    pub fn pool_size(&self, pool_name: &str) -> Option<usize> {
        self.pools.get(pool_name).map(|pool| pool.len())
    }
}

impl<M: Message, S: MailboxSender<M>> Default for ActorRegistry<M, S> {
    fn default() -> Self {
        Self::new()
    }
}

impl<M: Message, S: MailboxSender<M>> Clone for ActorRegistry<M, S> {
    /// Cheap clone via Arc (M-SERVICES-CLONE pattern).
    ///
    /// All clones share the same underlying data structures.
    fn clone(&self) -> Self {
        Self {
            routing_table: Arc::clone(&self.routing_table),
            routing_keys: Arc::clone(&self.routing_keys),
            pools: Arc::clone(&self.pools),
            pool_counters: Arc::clone(&self.pool_counters),
            _phantom: PhantomData,
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use crate::mailbox::metrics::AtomicMetrics;
    use crate::mailbox::UnboundedMailbox;
    use crate::message::{Message, MessagePriority};
    use crate::util::ActorId;

    #[derive(Debug, Clone)]
    #[allow(dead_code)]
    struct TestMessage {
        data: String,
    }

    impl Message for TestMessage {
        const MESSAGE_TYPE: &'static str = "test_message";

        fn priority(&self) -> MessagePriority {
            MessagePriority::Normal
        }
    }

    // Type alias for clarity in tests
    type TestMailbox = UnboundedMailbox<TestMessage, AtomicMetrics>;
    type TestSender = crate::mailbox::UnboundedMailboxSender<TestMessage, AtomicMetrics>;

    #[test]
    fn test_new_registry() {
        let registry = ActorRegistry::<TestMessage, TestSender>::new();
        assert_eq!(registry.actor_count(), 0);
        assert_eq!(registry.pool_count(), 0);
    }

    #[test]
    fn test_register_and_resolve() {
        let registry = ActorRegistry::<TestMessage, TestSender>::new();
        let (_receiver, sender) = TestMailbox::new();
        let address = ActorAddress::anonymous();

        // Register actor
        registry.register(address.clone(), sender).unwrap();
        assert_eq!(registry.actor_count(), 1);

        // Resolve actor
        let resolved = registry.resolve(&address);
        assert!(resolved.is_ok());
    }

    #[test]
    fn test_duplicate_registration() {
        let registry = ActorRegistry::<TestMessage, TestSender>::new();
        let (_receiver1, sender1) = TestMailbox::new();
        let (_receiver2, sender2) = TestMailbox::new();
        let address = ActorAddress::anonymous();

        // First registration succeeds
        assert!(registry.register(address.clone(), sender1).is_ok());

        // Second registration fails
        let result = registry.register(address, sender2);
        assert!(matches!(result, Err(BrokerError::DuplicateRegistration(_))));
    }

    #[test]
    fn test_unregister() {
        let registry = ActorRegistry::<TestMessage, TestSender>::new();
        let (_receiver, sender) = TestMailbox::new();
        let address = ActorAddress::anonymous();

        // Register and verify
        registry.register(address.clone(), sender).unwrap();
        assert_eq!(registry.actor_count(), 1);

        // Unregister and verify
        registry.unregister(&address).unwrap();
        assert_eq!(registry.actor_count(), 0);

        // Resolve should fail
        assert!(matches!(
            registry.resolve(&address),
            Err(BrokerError::ActorNotFound(_))
        ));
    }

    #[test]
    fn test_unregister_not_found() {
        let registry = ActorRegistry::<TestMessage, TestSender>::new();
        let address = ActorAddress::anonymous();

        let result = registry.unregister(&address);
        assert!(matches!(result, Err(BrokerError::ActorNotFound(_))));
    }

    #[test]
    fn test_resolve_not_found() {
        let registry = ActorRegistry::<TestMessage, TestSender>::new();
        let address = ActorAddress::anonymous();

        let result = registry.resolve(&address);
        assert!(matches!(result, Err(BrokerError::ActorNotFound(_))));
    }

    #[test]
    fn test_routing_key_resolution() {
        let registry = ActorRegistry::<TestMessage, TestSender>::new();
        let (_receiver, sender) = TestMailbox::new();
        let address = ActorAddress::anonymous();

        // Register actor
        registry.register(address.clone(), sender).unwrap();

        // Compute routing key
        let key = ActorRegistry::<TestMessage, TestSender>::compute_routing_key(&address);

        // Resolve by routing key
        let resolved = registry.resolve_by_routing_key(key);
        assert!(resolved.is_some());
    }

    #[test]
    fn test_pool_registration() {
        let registry = ActorRegistry::<TestMessage, TestSender>::new();

        // Register pool members
        for i in 0..3 {
            let (_receiver, sender) = TestMailbox::new();
            let address = ActorAddress::Named {
                id: ActorId::new(),
                name: format!("workers:worker-{i}"),
            };
            registry.register(address, sender).unwrap();
        }

        assert_eq!(registry.actor_count(), 3);
        assert_eq!(registry.pool_count(), 1);
        assert_eq!(registry.pool_size("workers"), Some(3));
    }

    #[test]
    fn test_pool_round_robin_strategy() {
        let registry = ActorRegistry::<TestMessage, TestSender>::new();

        // Register 3 pool members
        for i in 0..3 {
            let (_receiver, sender) = TestMailbox::new();
            let address = ActorAddress::Named {
                id: ActorId::new(),
                name: format!("workers:worker-{i}"),
            };
            registry.register(address, sender).unwrap();
        }

        // Get members in round-robin fashion
        let member1 = registry.get_pool_member("workers", PoolStrategy::RoundRobin);
        let member2 = registry.get_pool_member("workers", PoolStrategy::RoundRobin);
        let member3 = registry.get_pool_member("workers", PoolStrategy::RoundRobin);
        let member4 = registry.get_pool_member("workers", PoolStrategy::RoundRobin);

        assert!(member1.is_some());
        assert!(member2.is_some());
        assert!(member3.is_some());
        assert!(member4.is_some());

        // Fourth should wrap around to first
        assert_eq!(member1, member4);
    }

    #[test]
    fn test_pool_random_strategy() {
        let registry = ActorRegistry::<TestMessage, TestSender>::new();

        // Register 5 pool members for better randomness testing
        for i in 0..5 {
            let (_receiver, sender) = TestMailbox::new();
            let address = ActorAddress::Named {
                id: ActorId::new(),
                name: format!("workers:worker-{i}"),
            };
            registry.register(address, sender).unwrap();
        }

        // Get 10 random members
        let mut members = Vec::new();
        for _ in 0..10 {
            if let Some(member) = registry.get_pool_member("workers", PoolStrategy::Random) {
                members.push(member);
            }
        }

        // Should have selected 10 members
        assert_eq!(members.len(), 10);

        // All members should be valid (registered)
        for member in &members {
            assert!(registry.resolve(member).is_ok());
        }
    }

    #[test]
    fn test_pool_empty() {
        let registry = ActorRegistry::<TestMessage, TestSender>::new();

        let member = registry.get_pool_member("nonexistent", PoolStrategy::RoundRobin);
        assert!(member.is_none());
    }

    #[test]
    fn test_concurrent_registration() {
        use std::thread;

        let registry = ActorRegistry::<TestMessage, TestSender>::new();
        let registry_clone = registry.clone();

        // Spawn thread to register actors concurrently
        let handle = thread::spawn(move || {
            for i in 0..100 {
                let (_receiver, sender) = TestMailbox::new();
                let address = ActorAddress::Named {
                    id: ActorId::new(),
                    name: format!("actor-{i}"),
                };
                let _ = registry_clone.register(address, sender);
            }
        });

        // Register actors in main thread
        for i in 100..200 {
            let (_receiver, sender) = TestMailbox::new();
            let address = ActorAddress::Named {
                id: ActorId::new(),
                name: format!("actor-{i}"),
            };
            let _ = registry.register(address, sender);
        }

        handle.join().expect("Thread panicked");

        // Should have all actors registered
        assert_eq!(registry.actor_count(), 200);
    }

    #[test]
    fn test_registry_clone() {
        let registry = ActorRegistry::<TestMessage, TestSender>::new();
        let (_receiver, sender) = TestMailbox::new();
        let address = ActorAddress::anonymous();

        registry.register(address.clone(), sender).unwrap();

        // Clone shares same data
        let registry_clone = registry.clone();
        assert_eq!(registry_clone.actor_count(), 1);
        assert!(registry_clone.resolve(&address).is_ok());
    }

    #[test]
    fn test_pool_strategy_equality() {
        assert_eq!(PoolStrategy::RoundRobin, PoolStrategy::RoundRobin);
        assert_eq!(PoolStrategy::Random, PoolStrategy::Random);
        assert_ne!(PoolStrategy::RoundRobin, PoolStrategy::Random);
    }
}
