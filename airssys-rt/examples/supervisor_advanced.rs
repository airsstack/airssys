//! Advanced Supervisor Patterns Examples
//!
//! This example demonstrates:
//! - All three restart strategies (OneForOne, OneForAll, RestForOne)
//! - Child trait implementation
//! - ChildSpec factory patterns
//! - Health monitoring integration
//! - Real-world supervision scenarios

use airssys_rt::monitoring::{InMemoryMonitor, MonitoringConfig};
use airssys_rt::supervisor::{
    Child, ChildHealth, ChildSpec, OneForAll, OneForOne, RestForOne, RestartPolicy, ShutdownPolicy,
    Supervisor, SupervisorNode,
};
use async_trait::async_trait;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

// =============================================================================
// Example 1: OneForOne Strategy - Independent Workers
// =============================================================================

#[derive(Debug)]
struct HttpWorker {
    id: String,
    should_fail: Arc<AtomicBool>,
}

#[derive(Debug)]
struct WorkerError {
    message: String,
}

impl std::fmt::Display for WorkerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "WorkerError: {}", self.message)
    }
}

impl std::error::Error for WorkerError {}

#[async_trait]
impl Child for HttpWorker {
    type Error = WorkerError;

    async fn start(&mut self) -> Result<(), Self::Error> {
        println!("[{}] Starting", self.id);

        // Simulate startup failure if flag is set
        if self.should_fail.load(Ordering::Relaxed) {
            self.should_fail.store(false, Ordering::Relaxed);
            return Err(WorkerError {
                message: format!("{} startup failed", self.id),
            });
        }

        Ok(())
    }

    async fn stop(&mut self, _timeout: Duration) -> Result<(), Self::Error> {
        println!("[{}] Stopping gracefully", self.id);
        Ok(())
    }

    async fn health_check(&self) -> ChildHealth {
        ChildHealth::Healthy
    }
}

async fn demonstrate_one_for_one() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== OneForOne Strategy: Independent Workers ===\n");
    println!("Each worker restarts independently when it fails.\n");

    let monitor = InMemoryMonitor::new(MonitoringConfig::default());
    let mut supervisor = SupervisorNode::<OneForOne, HttpWorker, _>::new(OneForOne, monitor);

    // Create ChildSpec for multiple independent workers
    for i in 0..3 {
        let should_fail = Arc::new(AtomicBool::new(false));

        let worker_id = format!("worker-{}", i);
        let spec = ChildSpec {
            id: worker_id.clone(),
            factory: {
                let id = worker_id.clone();
                let fail = Arc::clone(&should_fail);
                move || HttpWorker {
                    id: id.clone(),
                    should_fail: Arc::clone(&fail),
                }
            },
            restart_policy: RestartPolicy::Permanent, // Always restart
            shutdown_policy: ShutdownPolicy::Graceful(Duration::from_secs(2)),
            start_timeout: Duration::from_secs(10),
            shutdown_timeout: Duration::from_secs(10),
        };

        supervisor.start_child(spec).await?;
        println!("✅ {} started", worker_id);
    }

    sleep(Duration::from_secs(1)).await;

    println!(
        "\n✅ All workers running. Child count: {}",
        supervisor.child_count()
    );
    println!("   Workers continue processing independently.\n");

    sleep(Duration::from_secs(1)).await;

    Ok(())
}

// =============================================================================
// Example 2: OneForAll Strategy - Tightly Coupled Services
// =============================================================================

#[derive(Debug)]
struct TransactionService {
    service_name: String,
    transactions: Arc<tokio::sync::Mutex<Vec<String>>>,
    should_fail: Arc<AtomicBool>,
}

#[derive(Debug)]
struct ServiceError {
    message: String,
}

impl std::fmt::Display for ServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ServiceError: {}", self.message)
    }
}

impl std::error::Error for ServiceError {}

#[async_trait]
impl Child for TransactionService {
    type Error = ServiceError;

    async fn start(&mut self) -> Result<(), Self::Error> {
        println!("[{}] Starting with clean state", self.service_name);

        // Simulate startup failure if flag is set
        if self.should_fail.load(Ordering::Relaxed) {
            self.should_fail.store(false, Ordering::Relaxed);
            return Err(ServiceError {
                message: format!("{} startup failed", self.service_name),
            });
        }

        // Clear transactions on restart
        self.transactions.lock().await.clear();
        Ok(())
    }

    async fn stop(&mut self, _timeout: Duration) -> Result<(), Self::Error> {
        println!("[{}] Stopping gracefully", self.service_name);
        Ok(())
    }

    async fn health_check(&self) -> ChildHealth {
        ChildHealth::Healthy
    }
}

async fn demonstrate_one_for_all() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== OneForAll Strategy: Tightly Coupled Services ===\n");
    println!("All services restart together when one fails.\n");

    let monitor = InMemoryMonitor::new(MonitoringConfig::default());
    let mut supervisor =
        SupervisorNode::<OneForAll, TransactionService, _>::new(OneForAll, monitor);

    // Create tightly coupled transaction services
    let service_names = vec!["OrderService", "InventoryService", "PaymentService"];

    for service_name in service_names {
        let transactions = Arc::new(tokio::sync::Mutex::new(Vec::new()));
        let should_fail = Arc::new(AtomicBool::new(false));

        let spec = ChildSpec {
            id: service_name.to_string(),
            factory: {
                let name = service_name.to_string();
                let txs = Arc::clone(&transactions);
                let fail = Arc::clone(&should_fail);
                move || TransactionService {
                    service_name: name.clone(),
                    transactions: Arc::clone(&txs),
                    should_fail: Arc::clone(&fail),
                }
            },
            restart_policy: RestartPolicy::Permanent,
            shutdown_policy: ShutdownPolicy::Graceful(Duration::from_secs(2)),
            start_timeout: Duration::from_secs(10),
            shutdown_timeout: Duration::from_secs(10),
        };

        supervisor.start_child(spec).await?;
        println!("✅ {} started", service_name);
    }

    sleep(Duration::from_secs(1)).await;

    println!(
        "\n✅ All services running. Child count: {}",
        supervisor.child_count()
    );
    println!("   Services maintain consistent state together.\n");

    sleep(Duration::from_secs(1)).await;

    Ok(())
}

// =============================================================================
// Example 3: RestForOne Strategy - Pipeline Dependencies
// =============================================================================

#[derive(Debug)]
struct PipelineStage {
    stage_name: String,
    items_processed: Arc<AtomicU32>,
    should_fail: Arc<AtomicBool>,
}

#[derive(Debug)]
struct PipelineError {
    message: String,
}

impl std::fmt::Display for PipelineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PipelineError: {}", self.message)
    }
}

impl std::error::Error for PipelineError {}

#[async_trait]
impl Child for PipelineStage {
    type Error = PipelineError;

    async fn start(&mut self) -> Result<(), Self::Error> {
        println!("[{}] Starting", self.stage_name);

        // Simulate startup failure if flag is set
        if self.should_fail.load(Ordering::Relaxed) {
            self.should_fail.store(false, Ordering::Relaxed);
            return Err(PipelineError {
                message: format!("{} startup failed", self.stage_name),
            });
        }

        // Reset processing count on restart
        self.items_processed.store(0, Ordering::Relaxed);
        Ok(())
    }

    async fn stop(&mut self, _timeout: Duration) -> Result<(), Self::Error> {
        println!("[{}] Stopping gracefully", self.stage_name);
        Ok(())
    }

    async fn health_check(&self) -> ChildHealth {
        ChildHealth::Healthy
    }
}

async fn demonstrate_rest_for_one() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== RestForOne Strategy: Pipeline Dependencies ===\n");
    println!("When a stage fails, it and all following stages restart.\n");

    let monitor = InMemoryMonitor::new(MonitoringConfig::default());
    let mut supervisor = SupervisorNode::<RestForOne, PipelineStage, _>::new(RestForOne, monitor);

    // Create pipeline stages IN ORDER (important for RestForOne)
    let stage_names = vec!["Extractor", "Transformer", "Loader"];

    for stage_name in stage_names {
        let items_processed = Arc::new(AtomicU32::new(0));
        let should_fail = Arc::new(AtomicBool::new(false));

        let spec = ChildSpec {
            id: stage_name.to_string(),
            factory: {
                let name = stage_name.to_string();
                let items = Arc::clone(&items_processed);
                let fail = Arc::clone(&should_fail);
                move || PipelineStage {
                    stage_name: name.clone(),
                    items_processed: Arc::clone(&items),
                    should_fail: Arc::clone(&fail),
                }
            },
            restart_policy: RestartPolicy::Permanent,
            shutdown_policy: ShutdownPolicy::Graceful(Duration::from_secs(2)),
            start_timeout: Duration::from_secs(10),
            shutdown_timeout: Duration::from_secs(10),
        };

        supervisor.start_child(spec).await?;
        println!("✅ {} started", stage_name);
    }

    sleep(Duration::from_secs(1)).await;

    println!(
        "\n✅ Pipeline running. Child count: {}",
        supervisor.child_count()
    );
    println!("   Stages maintain sequential dependency order.\n");

    sleep(Duration::from_secs(1)).await;

    Ok(())
}

// =============================================================================
// Example 4: Health Monitoring Integration
// =============================================================================

#[derive(Debug)]
struct MonitoredWorker {
    id: String,
    health_status: Arc<tokio::sync::Mutex<ChildHealth>>,
}

#[async_trait]
impl Child for MonitoredWorker {
    type Error = WorkerError;

    async fn start(&mut self) -> Result<(), Self::Error> {
        println!("[{}] Starting with health monitoring", self.id);
        *self.health_status.lock().await = ChildHealth::Healthy;
        Ok(())
    }

    async fn stop(&mut self, _timeout: Duration) -> Result<(), Self::Error> {
        println!("[{}] Stopping", self.id);
        Ok(())
    }

    async fn health_check(&self) -> ChildHealth {
        self.health_status.lock().await.clone()
    }
}

async fn demonstrate_health_monitoring() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Health Monitoring Integration ===\n");
    println!("Supervisor can proactively check child health.\n");

    let monitor = InMemoryMonitor::new(MonitoringConfig::default());
    let mut supervisor = SupervisorNode::<OneForOne, MonitoredWorker, _>::new(OneForOne, monitor);

    // Create worker with health monitoring
    let health_status = Arc::new(tokio::sync::Mutex::new(ChildHealth::Healthy));

    let spec = ChildSpec {
        id: "monitored-worker".to_string(),
        factory: {
            let status = Arc::clone(&health_status);
            move || MonitoredWorker {
                id: "monitored-worker".to_string(),
                health_status: Arc::clone(&status),
            }
        },
        restart_policy: RestartPolicy::Permanent,
        shutdown_policy: ShutdownPolicy::Graceful(Duration::from_secs(2)),
        start_timeout: Duration::from_secs(10),
        shutdown_timeout: Duration::from_secs(10),
    };

    supervisor.start_child(spec).await?;
    println!("✅ Monitored worker started");

    sleep(Duration::from_secs(1)).await;

    // Check health (note: check_child_health requires ChildId which is internal)
    // For now, we demonstrate that the worker is running and healthy
    println!(
        "\n📊 Worker running. Child count: {}",
        supervisor.child_count()
    );
    println!("   Health monitoring is enabled via the Child trait's health_check method.\n");

    sleep(Duration::from_secs(1)).await;

    Ok(())
}

// =============================================================================
// Main: Run All Examples
// =============================================================================

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("╔════════════════════════════════════════════════════╗");
    println!("║  Advanced Supervisor Patterns Demonstration       ║");
    println!("╚════════════════════════════════════════════════════╝");

    // Example 1: OneForOne
    demonstrate_one_for_one().await?;
    sleep(Duration::from_secs(1)).await;

    // Example 2: OneForAll
    demonstrate_one_for_all().await?;
    sleep(Duration::from_secs(1)).await;

    // Example 3: RestForOne
    demonstrate_rest_for_one().await?;
    sleep(Duration::from_secs(1)).await;

    // Example 4: Health Monitoring
    demonstrate_health_monitoring().await?;

    println!("\n╔════════════════════════════════════════════════════╗");
    println!("║  All Examples Completed Successfully! ✅          ║");
    println!("╚════════════════════════════════════════════════════╝");

    Ok(())
}
