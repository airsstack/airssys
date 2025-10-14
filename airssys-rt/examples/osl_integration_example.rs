//! OSL Integration Example
//!
//! Demonstrates the OSL (Operating System Layer) supervisor hierarchy with broker-based
//! communication patterns. This example shows:
//!
//! 1. Creating a shared message broker
//! 2. Initializing OSLSupervisor with broker injection
//! 3. Request-response patterns for FileSystem, Process, and Network operations
//! 4. Graceful supervisor shutdown
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────┐
//! │     InMemoryMessageBroker          │
//! │         (OSLMessage)                │
//! └─────────────────────────────────────┘
//!              ↑           ↑
//!              │           │
//!    ┌─────────┴───────────┴─────────┐
//!    │     OSLSupervisor<M, B>       │
//!    │    (RestForOne Strategy)      │
//!    └───────────────────────────────┘
//!              ↓           ↓
//!    ┌─────────┴───────────┴─────────┐
//!    │  FileSystem │ Process │ Network│
//!    │   Actor     │  Actor  │  Actor │
//!    └─────────────┴─────────┴────────┘
//! ```

use std::collections::HashMap;
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Duration;

use airssys_rt::broker::{InMemoryMessageBroker, MessageBroker};
use airssys_rt::message::MessageEnvelope;
use airssys_rt::osl::actors::messages::{
    FileSystemOperation, FileSystemRequest, FileSystemResponse, NetworkOperation, NetworkRequest,
    NetworkResponse, ProcessOperation, ProcessRequest, ProcessResponse,
};
use airssys_rt::osl::supervisor::OSLMessage;
use airssys_rt::osl::OSLSupervisor;
use airssys_rt::util::{ActorAddress, MessageId};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("╔════════════════════════════════════════╗");
    println!("║   OSL Integration Example              ║");
    println!("║   Broker-Based Communication Demo      ║");
    println!("╚════════════════════════════════════════╝\n");

    // Step 1: Create shared message broker
    println!("📦 Step 1: Creating shared InMemoryMessageBroker<OSLMessage>");
    let broker = InMemoryMessageBroker::<OSLMessage>::new();
    println!("   ✓ Broker created\n");

    // Step 2: Create OSLSupervisor with broker injection
    println!("🎯 Step 2: Creating OSLSupervisor with broker");
    let supervisor = OSLSupervisor::new(broker.clone());
    println!("   ✓ OSLSupervisor created\n");

    // Step 3: Start OSLSupervisor (spawns all child actors)
    println!("🚀 Step 3: Starting OSLSupervisor");
    supervisor.start().await?;
    println!("   ✓ Supervisor started (FileSystem, Process, Network actors spawned)\n");

    // Allow actors to initialize
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Step 4: Demonstrate operations
    println!("🔄 Step 4: Demonstrating OSL Operations\n");

    demonstrate_filesystem_operation(&broker).await?;
    demonstrate_process_operation(&broker).await?;
    demonstrate_network_operation(&broker).await?;

    // Step 5: Note about shutdown
    println!("\n� Note: OSLSupervisor shutdown API is pending implementation (Task 2.3)");
    println!("   Actors will be cleaned up when the program exits.\n");

    println!("✨ Example complete!\n");
    Ok(())
}

/// Demonstrate FileSystem operation with request-response pattern
async fn demonstrate_filesystem_operation(
    broker: &InMemoryMessageBroker<OSLMessage>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("📁 FileSystem Operation Demo");

    // Create client address for response routing
    let client_addr = ActorAddress::named("client");

    // Subscribe to all messages on the broker
    let mut response_rx = broker.subscribe().await?;

    // Create FileSystem request
    let request = FileSystemRequest {
        operation: FileSystemOperation::ReadFile {
            path: "/etc/hosts".into(),
        },
        reply_to: client_addr.clone(),
        request_id: MessageId::new(),
    };

    // Wrap in OSLMessage and envelope
    let envelope = MessageEnvelope::new(OSLMessage::FileSystemReq(request))
        .with_sender(client_addr.clone())
        .with_reply_to(client_addr.clone());

    println!("   → Sending ReadFile request for /etc/hosts");
    broker.publish(envelope).await?;

    // Wait for response (filter for our reply_to address)
    println!("   ⏳ Waiting for response...");
    let response = tokio::time::timeout(Duration::from_secs(2), async {
        while let Some(msg) = response_rx.recv().await {
            if msg.reply_to.as_ref() == Some(&client_addr) {
                return Ok(msg);
            }
        }
        Err("No response received")
    })
    .await??;

    // Process response
    match response.payload {
        OSLMessage::FileSystemResp(FileSystemResponse { result, .. }) => {
            println!("   ✓ Received FileSystem response: {:?}\n", result);
        }
        _ => println!("   ✗ Unexpected response type\n"),
    }

    Ok(())
}

/// Demonstrate Process operation with request-response pattern
async fn demonstrate_process_operation(
    broker: &InMemoryMessageBroker<OSLMessage>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("⚙️  Process Operation Demo");

    let client_addr = ActorAddress::named("client");
    let mut response_rx = broker.subscribe().await?;

    // Create Process request with correct Spawn structure
    let request = ProcessRequest {
        operation: ProcessOperation::Spawn {
            program: PathBuf::from("echo"),
            args: vec!["Hello from OSL!".to_string()],
            env: HashMap::new(),
            working_dir: None,
        },
        reply_to: client_addr.clone(),
        request_id: MessageId::new(),
    };

    let envelope = MessageEnvelope::new(OSLMessage::ProcessReq(request))
        .with_sender(client_addr.clone())
        .with_reply_to(client_addr.clone());

    println!("   → Sending Spawn request for 'echo Hello from OSL!'");
    broker.publish(envelope).await?;

    println!("   ⏳ Waiting for response...");
    let response = tokio::time::timeout(Duration::from_secs(2), async {
        while let Some(msg) = response_rx.recv().await {
            if msg.reply_to.as_ref() == Some(&client_addr) {
                return Ok(msg);
            }
        }
        Err("No response received")
    })
    .await??;

    match response.payload {
        OSLMessage::ProcessResp(ProcessResponse { result, .. }) => {
            println!("   ✓ Received Process response: {:?}\n", result);
        }
        _ => println!("   ✗ Unexpected response type\n"),
    }

    Ok(())
}

/// Demonstrate Network operation with request-response pattern
async fn demonstrate_network_operation(
    broker: &InMemoryMessageBroker<OSLMessage>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("🌐 Network Operation Demo");

    let client_addr = ActorAddress::named("client");
    let mut response_rx = broker.subscribe().await?;

    // Create Network request (SocketAddr type)
    let request = NetworkRequest {
        operation: NetworkOperation::TcpConnect {
            addr: std::net::SocketAddr::from_str("127.0.0.1:8080")?,
            timeout: Duration::from_secs(5),
        },
        reply_to: client_addr.clone(),
        request_id: MessageId::new(),
    };

    let envelope = MessageEnvelope::new(OSLMessage::NetworkReq(request))
        .with_sender(client_addr.clone())
        .with_reply_to(client_addr.clone());

    println!("   → Sending TcpConnect request to 127.0.0.1:8080");
    broker.publish(envelope).await?;

    println!("   ⏳ Waiting for response...");
    let response = tokio::time::timeout(Duration::from_secs(2), async {
        while let Some(msg) = response_rx.recv().await {
            if msg.reply_to.as_ref() == Some(&client_addr) {
                return Ok(msg);
            }
        }
        Err("No response received")
    })
    .await??;

    match response.payload {
        OSLMessage::NetworkResp(NetworkResponse { result, .. }) => {
            println!("   ✓ Received Network response: {:?}\n", result);
        }
        _ => println!("   ✗ Unexpected response type\n"),
    }

    Ok(())
}
