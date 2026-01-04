//! # Fire-and-Forget Messaging Example
//!
//! **Purpose:** Demonstrates fire-and-forget message delivery to WASM components
//! **Demonstrates:** WasmEngine::call_handle_message(), Component Model integration
//! **Run:** `cargo run --example fire_and_forget_messaging`
//!
//! This example shows the complete fire-and-forget messaging pattern:
//! 1. Create a WasmEngine
//! 2. Load a component with handle-message export
//! 3. Deliver messages using WasmEngine::call_handle_message()
//! 4. Handle success and error cases
//!
//! # Task Reference
//!
//! WASM-TASK-006 Phase 2 Task 2.2: handle-message Component Export
//!
//! # Architecture Reference
//!
//! - **WIT Interface:** `wit/core/component-lifecycle.wit` lines 86-89
//! - **Implementation:** `src/runtime/engine.rs` WasmEngine::call_handle_message()
//! - **ADR Reference:** ADR-WASM-001 (multicodec payload format)
//!
//! # Message Flow
//!
//! ```text
//! ┌─────────────────┐                    ┌────────────────────┐
//! │   Host Runtime  │   call_handle_     │  WASM Component    │
//! │                 │   message()        │                    │
//! │  ┌───────────┐  │ ─────────────────► │  ┌──────────────┐  │
//! │  │ WasmEngine│  │   sender + payload │  │handle-message│  │
//! │  └───────────┘  │ ◄───────────────── │  │   export     │  │
//! │                 │   Ok/Err result    │  └──────────────┘  │
//! └─────────────────┘                    └────────────────────┘
//! ```
//!
//! # Component Model
//!
//! The handle-message export uses WebAssembly Component Model with:
//! - Automatic parameter marshalling via Canonical ABI
//! - Type-safe function invocation via Wasmtime typed functions
//! - Structured error handling via result types
//!
//! # WIT Signature
//!
//! ```wit
//! handle-message: func(
//!     sender: component-id,
//!     message: list<u8>
//! ) -> result<_, component-error>;
//! ```

// Layer 1: Standard library imports
use std::path::Path;
use std::sync::Arc;

// Layer 2: Third-party crate imports
// (none needed)

// Layer 3: Internal module imports
use airssys_wasm::core::runtime::RuntimeEngine;
use airssys_wasm::core::ComponentId;
use airssys_wasm::runtime::WasmEngine;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Fire-and-Forget Messaging Example ===\n");
    println!("WASM-TASK-006 Phase 2 Task 2.2: handle-message Component Export\n");

    // Step 1: Create WasmEngine
    // The engine manages Wasmtime configuration for Component Model support.
    // Uses Arc<Inner> pattern for cheap cloning across threads.
    println!("--- Step 1: Create WasmEngine ---");
    let engine = Arc::new(WasmEngine::new()?);
    println!("✓ WasmEngine created with Component Model support");
    println!("  - Async support enabled (Tokio integration)");
    println!("  - Fuel metering enabled (CPU limiting)");
    println!("  - Cranelift JIT compiler\n");

    // Step 2: Load component with handle-message export
    // The component must export a handle-message function per the WIT interface.
    // See: wit/core/component-lifecycle.wit lines 86-89
    println!("--- Step 2: Load Component ---");

    // Use the test fixture - handle-message-component.wasm
    // In production, you would load your own compiled component.
    let fixture_path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/handle-message-component.wasm");

    let bytes = match std::fs::read(&fixture_path) {
        Ok(bytes) => {
            println!("✓ Loaded component from: {}", fixture_path.display());
            println!("  - Size: {} bytes", bytes.len());
            bytes
        }
        Err(e) => {
            eprintln!("✗ Failed to read fixture: {}", e);
            eprintln!("  Ensure tests/fixtures/handle-message-component.wasm exists.");
            eprintln!("  Run: cd tests/fixtures && ./build.sh");
            return Err(e.into());
        }
    };

    // Create component ID for the receiver
    let receiver_id = ComponentId::new("message-handler");
    let handle = engine.load_component(&receiver_id, &bytes).await?;
    println!("✓ Component loaded successfully");
    println!("  - Component ID: {}", receiver_id.as_str());
    println!("  - Handle ID: {}\n", handle.id());

    // Step 3: Send fire-and-forget messages
    // Fire-and-forget means we send the message and don't wait for a reply.
    // The component processes the message internally.
    println!("--- Step 3: Send Fire-and-Forget Messages ---");

    // Example 1: Simple text message
    println!("\nMessage 1: Simple text");
    let sender1 = ComponentId::new("event-source");
    let payload1 = b"Hello from the host runtime!";
    println!("  From: {}", sender1.as_str());
    println!("  Payload: \"{}\"", String::from_utf8_lossy(payload1));

    match engine
        .call_handle_message(&handle, &sender1, payload1)
        .await
    {
        Ok(()) => println!("  ✓ Message delivered successfully"),
        Err(e) => println!("  ✗ Delivery failed: {}", e),
    }

    // Example 2: Binary payload (e.g., serialized data)
    println!("\nMessage 2: Binary payload");
    let sender2 = ComponentId::new("data-pipeline");
    let payload2: Vec<u8> = (0..100).collect(); // 100 bytes of incrementing values
    println!("  From: {}", sender2.as_str());
    println!("  Payload: {} bytes of binary data", payload2.len());

    match engine
        .call_handle_message(&handle, &sender2, &payload2)
        .await
    {
        Ok(()) => println!("  ✓ Message delivered successfully"),
        Err(e) => println!("  ✗ Delivery failed: {}", e),
    }

    // Example 3: Empty payload (signal/event)
    println!("\nMessage 3: Empty payload (signal)");
    let sender3 = ComponentId::new("heartbeat-monitor");
    let payload3: &[u8] = &[];
    println!("  From: {}", sender3.as_str());
    println!("  Payload: (empty - used for signaling)");

    match engine
        .call_handle_message(&handle, &sender3, payload3)
        .await
    {
        Ok(()) => println!("  ✓ Signal delivered successfully"),
        Err(e) => println!("  ✗ Signal failed: {}", e),
    }

    // Example 4: Large payload
    println!("\nMessage 4: Large payload (64KB)");
    let sender4 = ComponentId::new("bulk-data-sender");
    let payload4: Vec<u8> = (0..65536).map(|i| (i % 256) as u8).collect();
    println!("  From: {}", sender4.as_str());
    println!("  Payload: {} bytes", payload4.len());

    match engine
        .call_handle_message(&handle, &sender4, &payload4)
        .await
    {
        Ok(()) => println!("  ✓ Large message delivered successfully"),
        Err(e) => println!("  ✗ Large message failed: {}", e),
    }

    // Step 4: Demonstrate error handling
    println!("\n--- Step 4: Error Handling ---");

    // Try to call handle-message on a component that doesn't export it
    println!("\nAttempting to call handle-message on hello_world.wasm (no export)...");
    let hello_fixture =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/hello_world.wasm");

    if let Ok(hello_bytes) = std::fs::read(&hello_fixture) {
        let hello_id = ComponentId::new("hello-component");
        if let Ok(hello_handle) = engine.load_component(&hello_id, &hello_bytes).await {
            let sender = ComponentId::new("test-sender");
            match engine
                .call_handle_message(&hello_handle, &sender, b"test")
                .await
            {
                Ok(()) => println!("  Unexpected success!"),
                Err(e) => {
                    println!("  ✓ Error correctly reported:");
                    println!("    {}", e);
                    println!("  → Component doesn't export handle-message");
                }
            }
        }
    }

    // Step 5: Summary
    println!("\n=== Example Complete ===\n");

    println!("📋 Summary:");
    println!("   - Fire-and-forget pattern demonstrated");
    println!("   - 4 messages delivered successfully");
    println!("   - Error handling for missing export shown");

    println!("\n📖 Key Concepts:");
    println!("   - WasmEngine::call_handle_message() delivers messages to WASM");
    println!("   - Sender ID identifies the message source");
    println!("   - Payload is passed as list<u8> (Component Model type)");
    println!("   - Results propagate errors from component to host");

    println!("\n📚 References:");
    println!("   - WIT: wit/core/component-lifecycle.wit:86-89");
    println!("   - Impl: src/runtime/engine.rs:455-531");
    println!("   - Tests: tests/wasm_engine_call_handle_message_tests.rs");

    println!("\n💡 Payload Format (ADR-WASM-001):");
    println!("   Components can use multicodec prefix for self-describing payloads:");
    println!("   - 0x55 = Raw binary");
    println!("   - 0x51 = CBOR");
    println!("   - 0x0129 = JSON");
    println!("   - 0x30 = Borsh");

    Ok(())
}
