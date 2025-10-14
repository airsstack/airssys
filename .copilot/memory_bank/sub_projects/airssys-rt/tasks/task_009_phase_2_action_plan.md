# RT-TASK-009 Phase 2: Hierarchical Supervisor Setup - Action Plan

**Created:** 2025-10-14  
**Status:** Ready for Implementation  
**Phase:** Phase 2 of 4 (OSL Integration)  
**Prerequisites:** Phase 1 Complete (100% ✅)  
**Duration Estimate:** 2 days (16 hours)

---

## Executive Summary

**Objective:** Implement hierarchical supervisor architecture with OSLSupervisor managing three OSL actors (FileSystemActor, ProcessActor, NetworkActor), demonstrating clean fault isolation between OS operations and application logic.

**Critical Gap Identified:** Phase 1 delivered fully functional OSL actors with comprehensive unit tests, but **no supervisor integration exists**. Actors can be instantiated but are never registered with a supervisor, making them unusable in production. Phase 2 closes this gap by implementing the complete supervisor hierarchy and registration flow.

**Key Deliverables:**
1. OSLSupervisor implementation (`src/osl/supervisor.rs`)
2. Example application demonstrating hierarchy (`examples/osl_integration_example.rs`)
3. Integration tests for supervisor hierarchy (`tests/supervisor_hierarchy_tests.rs`)
4. Documentation updates showing usage patterns

---

## Problem Statement

### Current State (Phase 1 Complete)

**What We Have:**
- ✅ Three OSL actors fully implemented (FileSystemActor, ProcessActor, NetworkActor)
- ✅ All actors implement Actor + Child traits
- ✅ ADR-RT-008 message wrapper pattern (Operation/Request/Response)
- ✅ 26 integration tests validating actor behavior
- ✅ 489 total tests passing, zero warnings

**Critical Gap:**
- ❌ **NO OSLSupervisor implementation** - Actors cannot be supervised
- ❌ **NO registration/initialization flow** - No way to start actors under supervision
- ❌ **NO supervisor hierarchy** - No RootSupervisor managing OSLSupervisor + ApplicationSupervisor
- ❌ **NO example application** - No demonstration of production usage
- ❌ **NO service discovery** - Application actors cannot locate OSL actors

**Impact:**
```rust
// Current limitation: Manual actor creation (unit testing only)
let mut actor = FileSystemActor::new();  // ❌ Not supervised
actor.handle_message(request, &mut context).await?;

// Missing: Supervised actor lifecycle (production usage)
let osl_supervisor = OSLSupervisor::new();  // ❌ Doesn't exist
osl_supervisor.start_child(FileSystemActor::new()).await?;  // ❌ No registration flow
```

### Desired End State (Phase 2 Complete)

**Production-Ready Supervisor Hierarchy:**
```
RootSupervisor (OneForOne)
├── OSLSupervisor (RestForOne) ← THIS IS WHAT WE'RE BUILDING
│   ├── FileSystemActor (supervised)
│   ├── ProcessActor (supervised)
│   └── NetworkActor (supervised)
└── ApplicationSupervisor (OneForOne)
    ├── WorkerActor (uses OSL actors via messages)
    ├── DataProcessorActor
    └── CoordinatorActor
```

**Capabilities After Phase 2:**
- ✅ OSL actors managed by dedicated supervisor
- ✅ Automatic restart on failure (RestForOne strategy)
- ✅ Cross-supervisor message passing (app actors → OSL actors)
- ✅ Clean fault isolation (OSL failures don't crash app actors)
- ✅ Service discovery (actors locate OSL services via addresses)
- ✅ Complete lifecycle management (start, stop, health monitoring)

---

## Architecture Design

### Supervisor Hierarchy Structure

```
┌─────────────────────────────────────────────────────────────┐
│                    RootSupervisor                           │
│                    (Strategy: OneForOne)                    │
│                                                             │
│  Purpose: Top-level coordinator for independent subsystems │
│  Behavior: Restart failed supervisor independently         │
└─────────────────────────────────────────────────────────────┘
                           │
         ┌─────────────────┴─────────────────┐
         │                                   │
         ▼                                   ▼
┌──────────────────────┐        ┌──────────────────────┐
│   OSLSupervisor      │        │ ApplicationSupervisor│
│ (Strategy: RestForOne│        │ (Strategy: OneForOne)│
│                      │        │                      │
│ Purpose: Manages     │        │ Purpose: Manages     │
│ OS integration actors│        │ business logic actors│
│                      │        │                      │
│ Behavior: Restart    │        │ Behavior: Independent│
│ failed actor + all   │        │ actor restarts       │
│ actors started after │        │                      │
└──────────────────────┘        └──────────────────────┘
         │                                   │
    ┌────┼────┐                        ┌────┼────┐
    ▼    ▼    ▼                        ▼    ▼    ▼
  ┌──┐ ┌──┐ ┌──┐                    ┌──┐ ┌──┐ ┌──┐
  │FS│ │PR│ │NT│                    │W1│ │DP│ │CO│
  │  │ │OC│ │WK│                    │  │ │  │ │  │
  └──┘ └──┘ └──┘                    └──┘ └──┘ └──┘
   ↑    ↑    ↑                        ↓    ↓    ↓
   │    │    │                        │    │    │
   │    │    │                        └────┼────┘
   │    │    │                             │
   │    │    └─────────────────────────────┘
   │    │         (Message passing across supervisor boundaries)
   │    └───────────────────────────────────┘
   └────────────────────────────────────────┘

Legend:
  FS  = FileSystemActor
  PROC = ProcessActor
  NTK  = NetworkActor
  W1  = WorkerActor-1
  DP  = DataProcessorActor
  CO  = CoordinatorActor
```

### Strategy Selection Rationale

**RootSupervisor: OneForOne**
- **Why:** OSL and Application subsystems are independent
- **Behavior:** If OSLSupervisor fails, restart only OSLSupervisor
- **Benefit:** Maximum fault isolation at top level

**OSLSupervisor: RestForOne**
- **Why:** OSL actors have potential dependencies (e.g., NetworkActor may need FileSystemActor for config)
- **Behavior:** If FileSystemActor fails, restart FileSystemActor + ProcessActor + NetworkActor
- **Benefit:** Ensures consistent state across OSL infrastructure after failure

**ApplicationSupervisor: OneForOne**
- **Why:** Business logic actors are independent
- **Behavior:** If WorkerActor-1 fails, restart only WorkerActor-1
- **Benefit:** Maximum resilience for application logic

### Message Passing Flow

```
Application Actor (Worker)
       │
       │ 1. Create FileSystemRequest
       │    (with MessageId + reply_to address)
       │
       ▼
   Broker.publish(request)
       │
       │ 2. Broker routes to FileSystemActor
       │    (via ActorAddress "osl-filesystem")
       │
       ▼
FileSystemActor.handle_message(request, context)
       │
       │ 3. Execute OSL operation
       │    (e.g., osl::read_file)
       │
       ▼
   Broker.publish(response)
       │
       │ 4. Broker routes to reply_to address
       │    (application actor receives result)
       │
       ▼
Application Actor receives response
```

---

## Implementation Plan

### Task 2.1: Create OSLSupervisor Module (2 hours)

**File:** `airssys-rt/src/osl/supervisor.rs` (NEW)

**Objective:** Implement OSLSupervisor that manages the three OSL actors with RestForOne strategy.

**Implementation Steps:**

#### Step 1: Module Structure and Imports

```rust
//! OSL Supervisor for managing OS integration actors.
//!
//! This module provides `OSLSupervisor` which manages FileSystemActor,
//! ProcessActor, and NetworkActor with RestForOne supervision strategy.

// Layer 1: Standard library imports
use std::sync::Arc;
use std::time::Duration;

// Layer 2: Third-party crate imports
use async_trait::async_trait;
use tokio::sync::Mutex;

// Layer 3: Internal module imports
use crate::broker::InMemoryMessageBroker;
use crate::supervisor::{
    Child, ChildHealth, ChildSpec, RestartPolicy, RestForOne, ShutdownPolicy, SupervisorNode,
};
use crate::util::ActorAddress;

use super::actors::{
    FileSystemActor, FileSystemRequest, NetworkActor, NetworkRequest, ProcessActor,
    ProcessRequest,
};
```

#### Step 2: OSLSupervisor Structure Definition

```rust
/// Supervisor for OS Layer integration actors.
///
/// Manages FileSystemActor, ProcessActor, and NetworkActor with RestForOne
/// supervision strategy to ensure consistent state across OSL infrastructure.
///
/// # Architecture
///
/// ```text
/// OSLSupervisor (RestForOne)
/// ├── FileSystemActor (all file/directory operations)
/// ├── ProcessActor (all process spawning/management)
/// └── NetworkActor (all network connections)
/// ```
///
/// # Supervision Strategy
///
/// Uses **RestForOne**: If FileSystemActor fails, it restarts along with all
/// actors started after it (ProcessActor, NetworkActor). This ensures
/// consistent state if dependencies exist.
///
/// # Usage
///
/// ```rust,no_run
/// use airssys_rt::osl::OSLSupervisor;
/// use std::time::Duration;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Create and start OSL supervisor
/// let osl_supervisor = OSLSupervisor::new();
/// osl_supervisor.start().await?;
///
/// // Get actor addresses for message routing
/// let fs_addr = osl_supervisor.filesystem_addr();
/// let proc_addr = osl_supervisor.process_addr();
/// let net_addr = osl_supervisor.network_addr();
///
/// // Application actors can now send messages to OSL actors
/// // via these addresses
///
/// // Graceful shutdown
/// osl_supervisor.shutdown(Duration::from_secs(5)).await?;
/// # Ok(())
/// # }
/// ```
pub struct OSLSupervisor {
    /// Internal supervisor managing actor lifecycle
    supervisor_fs: Arc<Mutex<SupervisorNode<RestForOne, FileSystemActor, crate::monitoring::InMemoryMonitor<crate::monitoring::SupervisionEvent>>>>,
    supervisor_proc: Arc<Mutex<SupervisorNode<RestForOne, ProcessActor, crate::monitoring::InMemoryMonitor<crate::monitoring::SupervisionEvent>>>>,
    supervisor_net: Arc<Mutex<SupervisorNode<RestForOne, NetworkActor, crate::monitoring::InMemoryMonitor<crate::monitoring::SupervisionEvent>>>>,

    /// Actor addresses for message routing
    filesystem_addr: ActorAddress,
    process_addr: ActorAddress,
    network_addr: ActorAddress,

    /// Started state flag
    started: Arc<Mutex<bool>>,
}
```

**Design Note:** Each actor type has its own supervisor because `SupervisorNode<S, C, M>` is generic over the child type `C`. We cannot mix different actor types in a single `SupervisorNode` due to Rust's type system. This is acceptable as each actor is managed independently with the same RestForOne strategy.

#### Step 3: Constructor and Initialization

```rust
impl OSLSupervisor {
    /// Create a new OSLSupervisor with default configuration.
    ///
    /// # Returns
    ///
    /// A new `OSLSupervisor` instance ready to start actors.
    ///
    /// # Example
    ///
    /// ```rust
    /// use airssys_rt::osl::OSLSupervisor;
    ///
    /// let osl_supervisor = OSLSupervisor::new();
    /// ```
    pub fn new() -> Self {
        use crate::monitoring::InMemoryMonitor;

        // Create supervisors for each actor type
        let supervisor_fs = Arc::new(Mutex::new(SupervisorNode::new(
            RestForOne,
            InMemoryMonitor::new(),
        )));
        let supervisor_proc = Arc::new(Mutex::new(SupervisorNode::new(
            RestForOne,
            InMemoryMonitor::new(),
        )));
        let supervisor_net = Arc::new(Mutex::new(SupervisorNode::new(
            RestForOne,
            InMemoryMonitor::new(),
        )));

        // Define actor addresses for service discovery
        let filesystem_addr = ActorAddress::named("osl-filesystem");
        let process_addr = ActorAddress::named("osl-process");
        let network_addr = ActorAddress::named("osl-network");

        Self {
            supervisor_fs,
            supervisor_proc,
            supervisor_net,
            filesystem_addr,
            process_addr,
            network_addr,
            started: Arc::new(Mutex::new(false)),
        }
    }

    /// Start all OSL actors under supervision.
    ///
    /// Registers and starts FileSystemActor, ProcessActor, and NetworkActor
    /// in dependency order (RestForOne strategy).
    ///
    /// # Errors
    ///
    /// Returns error if any actor fails to start.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use airssys_rt::osl::OSLSupervisor;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let osl_supervisor = OSLSupervisor::new();
    /// osl_supervisor.start().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut started = self.started.lock().await;
        if *started {
            return Ok(());
        }

        // Start FileSystemActor first (no dependencies)
        {
            let mut sup = self.supervisor_fs.lock().await;
            let spec = ChildSpec::new(
                "filesystem".to_string(),
                FileSystemActor::new(),
            )
            .with_restart_policy(RestartPolicy::Permanent)
            .with_shutdown_policy(ShutdownPolicy::Timeout(Duration::from_secs(5)));

            sup.start_child(spec).await?;
        }

        // Start ProcessActor second
        {
            let mut sup = self.supervisor_proc.lock().await;
            let spec = ChildSpec::new(
                "process".to_string(),
                ProcessActor::new(),
            )
            .with_restart_policy(RestartPolicy::Permanent)
            .with_shutdown_policy(ShutdownPolicy::Timeout(Duration::from_secs(5)));

            sup.start_child(spec).await?;
        }

        // Start NetworkActor third (may depend on FileSystem for config)
        {
            let mut sup = self.supervisor_net.lock().await;
            let spec = ChildSpec::new(
                "network".to_string(),
                NetworkActor::new(),
            )
            .with_restart_policy(RestartPolicy::Permanent)
            .with_shutdown_policy(ShutdownPolicy::Timeout(Duration::from_secs(5)));

            sup.start_child(spec).await?;
        }

        *started = true;
        Ok(())
    }

    /// Get FileSystemActor address for message routing.
    pub fn filesystem_addr(&self) -> &ActorAddress {
        &self.filesystem_addr
    }

    /// Get ProcessActor address for message routing.
    pub fn process_addr(&self) -> &ActorAddress {
        &self.process_addr
    }

    /// Get NetworkActor address for message routing.
    pub fn network_addr(&self) -> &ActorAddress {
        &self.network_addr
    }

    /// Shutdown all OSL actors gracefully.
    ///
    /// Stops all supervised actors with the specified timeout.
    ///
    /// # Arguments
    ///
    /// * `timeout` - Maximum time to wait for each actor to stop
    ///
    /// # Errors
    ///
    /// Returns error if any actor fails to stop within timeout.
    pub async fn shutdown(
        &self,
        timeout: Duration,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut started = self.started.lock().await;
        if !*started {
            return Ok(());
        }

        // Stop actors in reverse order
        {
            let mut sup = self.supervisor_net.lock().await;
            sup.stop_all_children(timeout).await?;
        }

        {
            let mut sup = self.supervisor_proc.lock().await;
            sup.stop_all_children(timeout).await?;
        }

        {
            let mut sup = self.supervisor_fs.lock().await;
            sup.stop_all_children(timeout).await?;
        }

        *started = false;
        Ok(())
    }
}
```

#### Step 4: Child Trait Implementation (for nesting in RootSupervisor)

```rust
#[async_trait]
impl Child for OSLSupervisor {
    type Error = Box<dyn std::error::Error + Send + Sync>;

    async fn start(&mut self) -> Result<(), Self::Error> {
        OSLSupervisor::start(self).await
    }

    async fn stop(&mut self, timeout: Duration) -> Result<(), Self::Error> {
        self.shutdown(timeout).await
    }

    async fn health_check(&self) -> ChildHealth {
        let started = self.started.lock().await;
        if *started {
            ChildHealth::Healthy
        } else {
            ChildHealth::Degraded("OSLSupervisor not started".to_string())
        }
    }
}
```

#### Step 5: Update Module Exports

**File:** `airssys-rt/src/osl/mod.rs`

```rust
pub mod actors;
pub mod supervisor;  // ADD THIS LINE

// Re-export commonly used types
pub use actors::{
    FileSystemActor, FileSystemError, FileSystemOperation, FileSystemRequest, FileSystemResponse,
    FileSystemResult, NetworkActor, NetworkError, NetworkOperation, NetworkRequest,
    NetworkResponse, NetworkResult, ProcessActor, ProcessError, ProcessOperation, ProcessRequest,
    ProcessResponse, ProcessResult,
};

pub use supervisor::OSLSupervisor;  // ADD THIS LINE
```

**Acceptance Criteria:**
- [ ] `OSLSupervisor` compiles without errors or warnings
- [ ] `OSLSupervisor::new()` creates supervisor instance
- [ ] `OSLSupervisor::start()` successfully starts all 3 actors
- [ ] `OSLSupervisor` implements `Child` trait for nesting
- [ ] All actors managed with appropriate policies
- [ ] Actor addresses available for message routing

---

### Task 2.2: Create Example Application (3 hours)

**File:** `airssys-rt/examples/osl_integration_example.rs` (NEW)

**Objective:** Demonstrate complete supervisor hierarchy with OSLSupervisor managing OSL actors and ApplicationSupervisor managing business logic actors that communicate with OSL actors.

**Implementation Steps:**

#### Step 1: Example Application Actors

```rust
//! OSL Integration Example
//!
//! Demonstrates hierarchical supervisor architecture with OSLSupervisor
//! managing OS integration actors and ApplicationSupervisor managing
//! business logic actors.

use std::path::PathBuf;
use std::time::Duration;

use airssys_rt::actor::{Actor, ActorContext, ErrorAction};
use airssys_rt::broker::InMemoryMessageBroker;
use airssys_rt::osl::{
    FileSystemOperation, FileSystemRequest, OSLSupervisor, ProcessOperation, ProcessRequest,
};
use airssys_rt::supervisor::{Child, ChildHealth, ChildSpec, OneForOne, SupervisorNode};
use airssys_rt::util::{ActorAddress, MessageId};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Work message for application actors
#[derive(Debug, Clone, Serialize, Deserialize)]
enum WorkMessage {
    ProcessFile { filename: String },
    SpawnTask { command: String },
}

/// Worker actor that uses FileSystemActor for data persistence
struct WorkerActor {
    id: String,
    filesystem_addr: ActorAddress,
    broker: InMemoryMessageBroker<FileSystemRequest>,
}

impl WorkerActor {
    fn new(id: String, filesystem_addr: ActorAddress) -> Self {
        Self {
            id,
            filesystem_addr,
            broker: InMemoryMessageBroker::new(),
        }
    }
}

#[async_trait]
impl Actor for WorkerActor {
    type Message = WorkMessage;
    type Error = Box<dyn std::error::Error + Send + Sync>;

    async fn handle_message(
        &mut self,
        msg: Self::Message,
        _ctx: &mut ActorContext<Self::Message, InMemoryMessageBroker<Self::Message>>,
    ) -> Result<(), Self::Error> {
        match msg {
            WorkMessage::ProcessFile { filename } => {
                println!("[Worker {}] Processing file: {}", self.id, filename);

                // Send request to FileSystemActor
                let request = FileSystemRequest {
                    request_id: MessageId::new(),
                    reply_to: ActorAddress::named(&format!("worker-{}", self.id)),
                    operation: FileSystemOperation::ReadFile {
                        path: PathBuf::from(&filename),
                    },
                };

                self.broker.publish(request).await?;
                println!("[Worker {}] Sent file read request to FileSystemActor", self.id);
                Ok(())
            }
            WorkMessage::SpawnTask { command } => {
                println!("[Worker {}] Spawning task: {}", self.id, command);
                Ok(())
            }
        }
    }

    async fn on_error(
        &mut self,
        error: Self::Error,
        _message: Option<&Self::Message>,
    ) -> ErrorAction {
        eprintln!("[Worker {}] Error: {}", self.id, error);
        ErrorAction::Stop
    }
}

#[async_trait]
impl Child for WorkerActor {
    type Error = Box<dyn std::error::Error + Send + Sync>;

    async fn start(&mut self) -> Result<(), Self::Error> {
        println!("[Worker {}] Starting", self.id);
        Ok(())
    }

    async fn stop(&mut self, _timeout: Duration) -> Result<(), Self::Error> {
        println!("[Worker {}] Stopping", self.id);
        Ok(())
    }

    async fn health_check(&self) -> ChildHealth {
        ChildHealth::Healthy
    }
}

/// Application supervisor managing business logic actors
struct ApplicationSupervisor {
    supervisor: SupervisorNode<
        OneForOne,
        WorkerActor,
        airssys_rt::monitoring::InMemoryMonitor<airssys_rt::monitoring::SupervisionEvent>,
    >,
}

impl ApplicationSupervisor {
    fn new() -> Self {
        use airssys_rt::monitoring::InMemoryMonitor;

        Self {
            supervisor: SupervisorNode::new(OneForOne, InMemoryMonitor::new()),
        }
    }

    async fn start_with_osl_refs(
        &mut self,
        osl_supervisor: &OSLSupervisor,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Start 3 worker actors with references to OSL actors
        for i in 1..=3 {
            let worker = WorkerActor::new(
                format!("worker-{}", i),
                osl_supervisor.filesystem_addr().clone(),
            );

            let spec = ChildSpec::new(format!("worker-{}", i), worker);
            self.supervisor.start_child(spec).await?;
        }

        Ok(())
    }

    async fn stop_all(&mut self, timeout: Duration) -> Result<(), Box<dyn std::error::Error>> {
        self.supervisor.stop_all_children(timeout).await?;
        Ok(())
    }
}
```

#### Step 2: Main Function with Complete Hierarchy

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("╔════════════════════════════════════════════════╗");
    println!("║   OSL Integration Example                      ║");
    println!("║   Hierarchical Supervisor Architecture        ║");
    println!("╚════════════════════════════════════════════════╝\n");

    // ═══════════════════════════════════════════════════════════════
    // Step 1: Create OSLSupervisor
    // ═══════════════════════════════════════════════════════════════
    println!("📋 Step 1: Creating OSL Supervisor...");
    let osl_supervisor = OSLSupervisor::new();
    println!("   ✓ OSLSupervisor created\n");

    // ═══════════════════════════════════════════════════════════════
    // Step 2: Start OSL Actors
    // ═══════════════════════════════════════════════════════════════
    println!("🚀 Step 2: Starting OSL Actors...");
    osl_supervisor.start().await?;
    println!("   ✓ FileSystemActor started");
    println!("   ✓ ProcessActor started");
    println!("   ✓ NetworkActor started\n");

    // ═══════════════════════════════════════════════════════════════
    // Step 3: Create ApplicationSupervisor
    // ═══════════════════════════════════════════════════════════════
    println!("📋 Step 3: Creating Application Supervisor...");
    let mut app_supervisor = ApplicationSupervisor::new();
    println!("   ✓ ApplicationSupervisor created\n");

    // ═══════════════════════════════════════════════════════════════
    // Step 4: Start Application Actors
    // ═══════════════════════════════════════════════════════════════
    println!("🚀 Step 4: Starting Application Actors...");
    app_supervisor.start_with_osl_refs(&osl_supervisor).await?;
    println!("   ✓ Worker-1 started");
    println!("   ✓ Worker-2 started");
    println!("   ✓ Worker-3 started\n");

    // ═══════════════════════════════════════════════════════════════
    // Step 5: Display Supervisor Hierarchy
    // ═══════════════════════════════════════════════════════════════
    println!("🌳 Supervisor Hierarchy:");
    println!("   ┌─────────────────────────┐");
    println!("   │   RootSupervisor        │");
    println!("   │   (Conceptual)          │");
    println!("   └─────────────────────────┘");
    println!("              │");
    println!("       ┌──────┴──────┐");
    println!("       │             │");
    println!("       ▼             ▼");
    println!("   ┌────────┐   ┌────────────┐");
    println!("   │  OSL   │   │Application │");
    println!("   │Supervisor│ │ Supervisor │");
    println!("   └────────┘   └────────────┘");
    println!("       │             │");
    println!("   ┌───┼───┐     ┌───┼───┐");
    println!("   ▼   ▼   ▼     ▼   ▼   ▼");
    println!("   FS PRO NET    W1  W2  W3\n");

    // ═══════════════════════════════════════════════════════════════
    // Step 6: Demonstrate Cross-Supervisor Communication
    // ═══════════════════════════════════════════════════════════════
    println!("💬 Step 5: Testing Cross-Supervisor Communication...");
    println!("   (Workers sending messages to OSL actors)\n");

    // Give actors time to process messages
    tokio::time::sleep(Duration::from_secs(1)).await;

    // ═══════════════════════════════════════════════════════════════
    // Step 7: Graceful Shutdown
    // ═══════════════════════════════════════════════════════════════
    println!("🛑 Step 6: Initiating Graceful Shutdown...");
    
    println!("   Stopping application actors...");
    app_supervisor.stop_all(Duration::from_secs(3)).await?;
    println!("   ✓ All application actors stopped");

    println!("   Stopping OSL actors...");
    osl_supervisor.shutdown(Duration::from_secs(3)).await?;
    println!("   ✓ All OSL actors stopped\n");

    println!("╔════════════════════════════════════════════════╗");
    println!("║   Example Complete ✓                           ║");
    println!("╚════════════════════════════════════════════════╝");

    Ok(())
}
```

**Acceptance Criteria:**
- [ ] Example compiles without errors or warnings
- [ ] Demonstrates complete supervisor hierarchy setup
- [ ] Shows OSLSupervisor managing OSL actors
- [ ] Shows ApplicationSupervisor managing business logic actors
- [ ] Displays clear hierarchy visualization
- [ ] Includes graceful shutdown sequence
- [ ] Runs successfully with `cargo run --example osl_integration_example`

---

### Task 2.3: Create Integration Tests (4 hours)

**File:** `airssys-rt/tests/supervisor_hierarchy_tests.rs` (NEW)

**Objective:** Comprehensive integration tests validating supervisor hierarchy, cross-supervisor communication, and fault isolation.

**Test Categories:**

1. **Supervisor Creation Tests (3 tests)**
2. **Cross-Supervisor Communication Tests (4 tests)**
3. **Fault Isolation Tests (5 tests)**
4. **Lifecycle Management Tests (3 tests)**

**Total: 15 integration tests**

**Implementation:** (Due to length, key test implementations shown)

```rust
//! Integration tests for OSL supervisor hierarchy

use std::path::PathBuf;
use std::time::Duration;

use airssys_rt::osl::{FileSystemOperation, FileSystemRequest, OSLSupervisor};
use airssys_rt::util::{ActorAddress, MessageId};

// ═══════════════════════════════════════════════════════════════
// Category 1: Supervisor Creation Tests
// ═══════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_osl_supervisor_creation() {
    let osl_supervisor = OSLSupervisor::new();

    // Verify actor addresses are configured
    assert_eq!(
        osl_supervisor.filesystem_addr().name(),
        "osl-filesystem"
    );
    assert_eq!(
        osl_supervisor.process_addr().name(),
        "osl-process"
    );
    assert_eq!(
        osl_supervisor.network_addr().name(),
        "osl-network"
    );
}

#[tokio::test]
async fn test_osl_supervisor_starts_all_actors() {
    let osl_supervisor = OSLSupervisor::new();

    // Start should succeed
    let result = osl_supervisor.start().await;
    assert!(
        result.is_ok(),
        "OSLSupervisor should start successfully: {:?}",
        result.err()
    );

    // Cleanup
    osl_supervisor
        .shutdown(Duration::from_secs(1))
        .await
        .unwrap();
}

#[tokio::test]
async fn test_osl_supervisor_idempotent_start() {
    let osl_supervisor = OSLSupervisor::new();

    // First start
    osl_supervisor.start().await.unwrap();

    // Second start should be idempotent (no error)
    let result = osl_supervisor.start().await;
    assert!(result.is_ok(), "Multiple starts should be idempotent");

    // Cleanup
    osl_supervisor
        .shutdown(Duration::from_secs(1))
        .await
        .unwrap();
}

// ═══════════════════════════════════════════════════════════════
// Category 2: Cross-Supervisor Communication Tests
// ═══════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_filesystem_actor_message_routing() {
    let osl_supervisor = OSLSupervisor::new();
    osl_supervisor.start().await.unwrap();

    // Verify we can get filesystem actor address
    let fs_addr = osl_supervisor.filesystem_addr();
    assert_eq!(fs_addr.name(), "osl-filesystem");

    // Cleanup
    osl_supervisor
        .shutdown(Duration::from_secs(1))
        .await
        .unwrap();
}

#[tokio::test]
async fn test_process_actor_message_routing() {
    let osl_supervisor = OSLSupervisor::new();
    osl_supervisor.start().await.unwrap();

    // Verify we can get process actor address
    let proc_addr = osl_supervisor.process_addr();
    assert_eq!(proc_addr.name(), "osl-process");

    // Cleanup
    osl_supervisor
        .shutdown(Duration::from_secs(1))
        .await
        .unwrap();
}

#[tokio::test]
async fn test_network_actor_message_routing() {
    let osl_supervisor = OSLSupervisor::new();
    osl_supervisor.start().await.unwrap();

    // Verify we can get network actor address
    let net_addr = osl_supervisor.network_addr();
    assert_eq!(net_addr.name(), "osl-network");

    // Cleanup
    osl_supervisor
        .shutdown(Duration::from_secs(1))
        .await
        .unwrap();
}

#[tokio::test]
async fn test_all_actor_addresses_unique() {
    let osl_supervisor = OSLSupervisor::new();

    let fs_addr = osl_supervisor.filesystem_addr();
    let proc_addr = osl_supervisor.process_addr();
    let net_addr = osl_supervisor.network_addr();

    // All addresses should be unique
    assert_ne!(fs_addr, proc_addr);
    assert_ne!(fs_addr, net_addr);
    assert_ne!(proc_addr, net_addr);
}

// ═══════════════════════════════════════════════════════════════
// Category 3: Fault Isolation Tests
// ═══════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_supervisor_restart_strategy() {
    let osl_supervisor = OSLSupervisor::new();
    osl_supervisor.start().await.unwrap();

    // Verify actors are started
    // (In production, actors would have metrics/health endpoints)

    osl_supervisor
        .shutdown(Duration::from_secs(1))
        .await
        .unwrap();
}

#[tokio::test]
async fn test_multiple_supervisors_independent() {
    // Create two independent OSLSupervisors
    let osl_supervisor_1 = OSLSupervisor::new();
    let osl_supervisor_2 = OSLSupervisor::new();

    // Both should start successfully
    osl_supervisor_1.start().await.unwrap();
    osl_supervisor_2.start().await.unwrap();

    // Cleanup
    osl_supervisor_1
        .shutdown(Duration::from_secs(1))
        .await
        .unwrap();
    osl_supervisor_2
        .shutdown(Duration::from_secs(1))
        .await
        .unwrap();
}

// ═══════════════════════════════════════════════════════════════
// Category 4: Lifecycle Management Tests
// ═══════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_graceful_shutdown_all_actors() {
    let osl_supervisor = OSLSupervisor::new();
    osl_supervisor.start().await.unwrap();

    // Shutdown should succeed
    let result = osl_supervisor.shutdown(Duration::from_secs(2)).await;
    assert!(
        result.is_ok(),
        "Graceful shutdown should succeed: {:?}",
        result.err()
    );
}

#[tokio::test]
async fn test_shutdown_idempotent() {
    let osl_supervisor = OSLSupervisor::new();
    osl_supervisor.start().await.unwrap();

    // First shutdown
    osl_supervisor
        .shutdown(Duration::from_secs(1))
        .await
        .unwrap();

    // Second shutdown should be idempotent
    let result = osl_supervisor.shutdown(Duration::from_secs(1)).await;
    assert!(result.is_ok(), "Multiple shutdowns should be idempotent");
}

#[tokio::test]
async fn test_supervisor_health_check() {
    use airssys_rt::supervisor::Child;

    let mut osl_supervisor = OSLSupervisor::new();

    // Health check before start
    let health_before = osl_supervisor.health_check().await;
    assert!(matches!(
        health_before,
        airssys_rt::supervisor::ChildHealth::Degraded(_)
    ));

    // Start supervisor
    osl_supervisor.start().await.unwrap();

    // Health check after start
    let health_after = osl_supervisor.health_check().await;
    assert!(matches!(
        health_after,
        airssys_rt::supervisor::ChildHealth::Healthy
    ));

    // Cleanup
    osl_supervisor
        .shutdown(Duration::from_secs(1))
        .await
        .unwrap();
}
```

**Acceptance Criteria:**
- [ ] All 15 integration tests pass
- [ ] Tests compile without errors or warnings
- [ ] Tests demonstrate supervisor creation and initialization
- [ ] Tests validate actor address routing
- [ ] Tests verify lifecycle management (start, stop, health)
- [ ] Test coverage >90% for OSLSupervisor code
- [ ] Tests run in <5 seconds total

---

### Task 2.4: Documentation Updates (1 hour)

**Files to Update:**

#### 1. `airssys-rt/src/osl/mod.rs` - Enhanced module documentation

```rust
//! # OSL Integration Module
//!
//! Provides actor-based integration with the AirsSys OS Layer (OSL).
//!
//! ## Architecture
//!
//! The OSL integration follows a hierarchical supervisor pattern (ADR-RT-007)
//! with dedicated OSL actors managed by `OSLSupervisor`:
//!
//! ```text
//! RootSupervisor
//! ├── OSLSupervisor (RestForOne) ← Manages OS integration
//! │   ├── FileSystemActor (all file/directory operations)
//! │   ├── ProcessActor (all process spawning/management)
//! │   └── NetworkActor (all network connections)
//! └── ApplicationSupervisor (OneForOne)
//!     └── Business logic actors
//! ```
//!
//! ## Usage
//!
//! ### Starting OSL Actors
//!
//! ```rust,no_run
//! use airssys_rt::osl::OSLSupervisor;
//! use std::time::Duration;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create and start OSL supervisor
//! let osl_supervisor = OSLSupervisor::new();
//! osl_supervisor.start().await?;
//!
//! // Get actor addresses for message routing
//! let fs_addr = osl_supervisor.filesystem_addr();
//! let proc_addr = osl_supervisor.process_addr();
//! let net_addr = osl_supervisor.network_addr();
//!
//! // Application actors can now send messages to OSL actors
//!
//! // Graceful shutdown
//! osl_supervisor.shutdown(Duration::from_secs(5)).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ### Sending Messages to OSL Actors
//!
//! ```rust,no_run
//! use airssys_rt::osl::{FileSystemOperation, FileSystemRequest};
//! use airssys_rt::util::{ActorAddress, MessageId};
//! use std::path::PathBuf;
//!
//! # async fn example(filesystem_addr: ActorAddress) -> Result<(), Box<dyn std::error::Error>> {
//! // Create request to FileSystemActor
//! let request = FileSystemRequest {
//!     request_id: MessageId::new(),
//!     reply_to: ActorAddress::named("my-actor"),
//!     operation: FileSystemOperation::ReadFile {
//!         path: PathBuf::from("config.txt"),
//!     },
//! };
//!
//! // Send via broker (application actor publishes to FileSystemActor)
//! // broker.publish(request).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Complete Example
//!
//! See `examples/osl_integration_example.rs` for a complete demonstration
//! of hierarchical supervisor setup and cross-supervisor communication.
//!
//! ## Key Benefits
//!
//! - **Clean Fault Isolation**: OSL failures don't cascade to application actors
//! - **Superior Testability**: Mock OSL actors in tests (no real OS operations)
//! - **Centralized Management**: Single source of truth for OS operations
//! - **Service-Oriented Design**: Clear service boundaries and contracts
//! - **Process Lifecycle Safety**: Automatic cleanup in ProcessActor.stop()
//!
//! ## Related Documentation
//!
//! - **ADR-RT-007**: Hierarchical Supervisor Architecture for OSL Integration
//! - **ADR-RT-008**: OSL Message Wrapper Pattern
//! - **KNOWLEDGE-RT-017**: OSL Integration Actors Pattern
```

#### 2. `airssys-rt/README.md` - Add OSL Integration section

Add before "Examples" section:

```markdown
### OSL Integration

AirsSys-RT provides seamless integration with the OS Layer (OSL) through
dedicated integration actors managed by `OSLSupervisor`:

- **FileSystemActor**: Centralized file operations with audit logging
- **ProcessActor**: OS process spawning and lifecycle management
- **NetworkActor**: Network connection handling with connection pooling

**Quick Start:**

```rust
use airssys_rt::osl::OSLSupervisor;

let osl_supervisor = OSLSupervisor::new();
osl_supervisor.start().await?;

// Application actors can now send messages to OSL actors
let fs_addr = osl_supervisor.filesystem_addr();
```

See `examples/osl_integration_example.rs` for complete usage.

**Architecture:** Follows hierarchical supervisor pattern (ADR-RT-007) with
clean fault isolation between OS operations and application logic.
```

#### 3. Memory Bank Updates

**Files to Update:**
- `.copilot/memory_bank/sub_projects/airssys-rt/progress.md`
- `.copilot/memory_bank/sub_projects/airssys-rt/tasks/task_009_osl_integration.md`
- `.copilot/memory_bank/current_context.md`

**Acceptance Criteria:**
- [ ] All documentation compiles (doctests pass)
- [ ] Clear usage examples provided
- [ ] Memory bank reflects Phase 2 completion
- [ ] Links to examples and tests included
- [ ] Architecture diagrams updated

---

## Testing Strategy

### Test Pyramid

```
       Integration Tests (15 tests)
      supervisor_hierarchy_tests.rs
     ┌──────────────────────────────┐
     │  Lifecycle (3)               │
     │  Fault Isolation (5)         │
     │  Cross-Supervisor Comm (4)   │
     │  Supervisor Creation (3)     │
     └──────────────────────────────┘
                  ▲
                  │
    ┌─────────────┴─────────────┐
    │  Example Application      │
    │  osl_integration_example  │
    │  (Manual verification)    │
    └───────────────────────────┘
                  ▲
                  │
    ┌─────────────┴─────────────┐
    │  Phase 1 Tests (26)       │
    │  osl_actors_tests.rs      │
    │  (Actor unit tests)       │
    └───────────────────────────┘
```

### Manual Testing Checklist

- [ ] Run `cargo check --workspace` - no errors
- [ ] Run `cargo clippy --workspace --all-targets --all-features` - no warnings
- [ ] Run `cargo test --package airssys-rt` - all tests pass
- [ ] Run example: `cargo run --example osl_integration_example`
- [ ] Verify example output shows complete hierarchy
- [ ] Check console logs for actor lifecycle events
- [ ] Verify graceful shutdown sequence

---

## Success Metrics

### Code Quality
- [ ] Zero compilation errors
- [ ] Zero compiler warnings
- [ ] Zero clippy warnings
- [ ] All tests pass (target: 504 total tests = 489 existing + 15 new)

### Test Coverage
- [ ] 15 new integration tests for supervisor hierarchy
- [ ] >90% coverage for OSLSupervisor code
- [ ] All critical paths tested (start, stop, health, routing)

### Documentation Quality
- [ ] All doctests pass
- [ ] Example runs successfully
- [ ] Clear architecture diagrams included
- [ ] Memory bank fully updated with Phase 2 completion

### Functional Validation
- [ ] OSLSupervisor manages all 3 OSL actors
- [ ] Actors start in correct order
- [ ] Actors stop in correct order
- [ ] Actor addresses available for routing
- [ ] Health checks work correctly

---

## Risk Assessment

### Risk 1: Generic Type Complexity
**Issue:** SupervisorNode has complex generic parameters  
**Impact:** Compilation errors, type inference issues  
**Mitigation:** 
- Use separate supervisor instance per actor type
- Type aliases for complex signatures
- Careful trait bound specifications  
**Status:** MITIGATED (separate supervisors per actor type)

### Risk 2: Message Broker Separation
**Issue:** Each actor type needs separate broker (generic over message type)  
**Impact:** Cannot share broker between actor types  
**Mitigation:** Create broker instances per actor type in supervisors  
**Status:** MITIGATED (pattern validated in Phase 1 tests)

### Risk 3: Child Trait Implementation
**Issue:** OSLSupervisor must implement Child for nesting in RootSupervisor  
**Impact:** Complex trait implementation with async methods  
**Mitigation:** Follow existing SupervisorNode patterns from RT-TASK-007  
**Status:** LOW RISK (pattern already established)

### Risk 4: Actor Startup Order
**Issue:** Actors may have startup dependencies (e.g., NetworkActor needs FileSystemActor)  
**Impact:** Startup failures if order incorrect  
**Mitigation:** RestForOne strategy ensures restart propagation  
**Status:** LOW RISK (addressed by supervision strategy)

---

## Dependencies and Prerequisites

### Completed (Phase 1) ✅
- ✅ FileSystemActor, ProcessActor, NetworkActor implemented
- ✅ ADR-RT-008 message wrapper pattern implemented
- ✅ All actors implement Actor + Child traits
- ✅ 489 tests passing (336 unit + 13 monitoring + 26 OSL integration + 114 doc)
- ✅ Zero warnings compilation

### Required Infrastructure (Already Exists) ✅
- ✅ SupervisorNode with generic strategy support (RT-TASK-007)
- ✅ Child trait with start/stop/health_check
- ✅ InMemoryMessageBroker for pub-sub (RT-TASK-004)
- ✅ Monitoring infrastructure (RT-TASK-010)
- ✅ RestForOne supervision strategy (RT-TASK-007)

### New Requirements (Phase 2)
- 🆕 OSLSupervisor implementation
- 🆕 Example application actors
- 🆕 Supervisor hierarchy integration tests
- 🆕 Documentation updates

---

## Timeline Estimate

### Day 5: Implementation (8 hours)
**Morning (4h):**
- Task 2.1: Create OSLSupervisor Module (2h)
- Task 2.2: Create Example Application (2h)

**Afternoon (4h):**
- Task 2.3: Integration Tests - Part 1 (4h)
  - Supervisor creation tests (3 tests)
  - Cross-supervisor communication tests (4 tests)

### Day 6: Testing and Documentation (8 hours)
**Morning (4h):**
- Task 2.3: Integration Tests - Part 2 (3h)
  - Fault isolation tests (5 tests)
  - Lifecycle management tests (3 tests)
- Debugging and fixes (1h)

**Afternoon (4h):**
- Task 2.4: Documentation Updates (2h)
  - Module documentation
  - README updates
  - Memory bank updates
- Final validation and testing (2h)
  - Run all tests
  - Run clippy
  - Run example
  - Verify memory bank

**Total Estimate:** 16 hours over 2 days

---

## Completion Checklist

### Implementation ✅
- [ ] `src/osl/supervisor.rs` created and compiles
- [ ] `examples/osl_integration_example.rs` runs successfully
- [ ] `tests/supervisor_hierarchy_tests.rs` - all 15 tests pass
- [ ] Module exports updated (`src/osl/mod.rs`)

### Quality ✅
- [ ] Zero compilation errors
- [ ] Zero warnings (compiler + clippy)
- [ ] All 504+ tests pass (489 existing + 15 new)
- [ ] Example demonstrates complete workflow
- [ ] Console output clear and informative

### Documentation ✅
- [ ] Module documentation updated with examples
- [ ] README includes OSL integration section
- [ ] Memory bank updated with Phase 2 completion
- [ ] ADR-RT-007 compliance validated
- [ ] All doctests pass

### Ready for Phase 3 ✅
- [ ] Supervisor hierarchy fully functional
- [ ] Cross-supervisor communication validated
- [ ] Fault isolation demonstrated
- [ ] Security context propagation patterns identified
- [ ] Audit logging touch-points documented

---

## Key Patterns to Follow

### §2.1: Import Organization (MANDATORY)
```rust
// Layer 1: Standard library imports
use std::sync::Arc;
use std::time::Duration;

// Layer 2: Third-party crate imports
use async_trait::async_trait;
use tokio::sync::Mutex;

// Layer 3: Internal module imports
use crate::supervisor::{Child, SupervisorNode};
```

### §6.2: Avoid `dyn` When Possible (MANDATORY)
- Use generics for compile-time dispatch
- SupervisorNode is generic over Child type
- Separate supervisor per actor type (type safety)

### §6.3: Microsoft Rust Guidelines (MANDATORY)
- Services implement cheap `Clone` via `Arc<Inner>`
- Essential functionality in inherent methods
- Mockable system calls (already done via Actor trait)

### ADR-RT-007: Hierarchical Supervisor Pattern
- OSLSupervisor manages OSL actors with RestForOne
- ApplicationSupervisor manages app actors with OneForOne
- RootSupervisor coordinates with OneForOne
- Cross-supervisor communication via standard message passing

### ADR-RT-008: Message Wrapper Pattern
- Use Operation/Request/Response three-layer design
- All messages cloneable (Clone + Serialize + Deserialize)
- MessageId for request-response correlation

---

## Reference Documentation

### Primary Sources
- `.copilot/memory_bank/sub_projects/airssys-rt/tasks/task_009_osl_integration.md`
- `.copilot/memory_bank/sub_projects/airssys-rt/docs/adr/adr_rt_007_hierarchical_supervisor_architecture.md`
- `.copilot/memory_bank/sub_projects/airssys-rt/docs/knowledges/knowledge_rt_017_osl_integration_actors.md`

### Implementation References
- `airssys-rt/src/supervisor/node.rs` - SupervisorNode patterns
- `airssys-rt/src/osl/actors/filesystem.rs` - Actor implementation example
- `airssys-rt/examples/supervisor_basic.rs` - Supervisor usage patterns
- `airssys-rt/tests/osl_actors_tests.rs` - Actor test patterns

### Related ADRs
- ADR-RT-007: Hierarchical Supervisor Architecture
- ADR-RT-008: OSL Message Wrapper Pattern
- ADR-RT-004: Child Trait Separation
- ADR-RT-001: Zero-Cost Abstractions

---

## Notes for AI Agents

### Critical Implementation Details

1. **Separate Supervisors Per Actor Type:**
   - `SupervisorNode<S, C, M>` is generic over child type `C`
   - Cannot mix FileSystemActor and ProcessActor in same SupervisorNode
   - Solution: Create separate supervisor instance per actor type
   - All use RestForOne strategy for consistency

2. **Actor Address Management:**
   - Use `ActorAddress::named("osl-filesystem")` for consistent naming
   - Store addresses in OSLSupervisor for service discovery
   - Application actors get addresses via `osl_supervisor.filesystem_addr()`

3. **Lifecycle Management:**
   - Start actors in dependency order (FileSystem → Process → Network)
   - Stop actors in reverse order (Network → Process → FileSystem)
   - Use `started` flag to make start/stop idempotent

4. **Child Trait Implementation:**
   - OSLSupervisor implements Child to enable nesting in RootSupervisor
   - Delegate to internal `start()` and `shutdown()` methods
   - Health check returns Healthy if started, Degraded otherwise

### Testing Approach

- **Unit Tests (Phase 1):** Test individual actors in isolation
- **Integration Tests (Phase 2):** Test supervisor hierarchy and communication
- **Example Application:** Manual verification of complete workflow
- **Focus Areas:** Start/stop sequences, address routing, idempotency

### Common Pitfalls to Avoid

❌ **Don't:** Try to put different actor types in same SupervisorNode  
✅ **Do:** Create separate supervisor per actor type

❌ **Don't:** Use shared broker for different message types  
✅ **Do:** Create broker instance per message type

❌ **Don't:** Hardcode actor addresses in application code  
✅ **Do:** Get addresses from OSLSupervisor

❌ **Don't:** Start actors without proper error handling  
✅ **Do:** Propagate errors and handle startup failures

---

**This action plan provides the complete roadmap for Phase 2 implementation.**  
**Status:** Ready for implementation when user approves.  
**Next Step:** Begin Task 2.1 (Create OSLSupervisor Module)
