# WASM-TASK-028: Implementation Plans

## Plan References
- **ADR-WASM-029:** Security Module Design (lines 390-451)
- **ADR-WASM-025:** Clean-Slate Rebuild Architecture
- **ADR-WASM-026:** Implementation Roadmap (Phase 4)

## Target Structure Reference

Per ADR-WASM-029:
```
security/
├── ...
└── audit.rs         # SecurityAuditLogger implementation
```

---

## Implementation Actions

### Action 1: Create `security/audit.rs`

**Objective:** Implement ConsoleSecurityAuditLogger with async logging

**File:** `airssys-wasm/src/security/audit.rs`

**Specification (ADR-WASM-029 lines 390-451):**

```rust
//! Security audit logging.

use std::sync::mpsc::{self, Sender};
use std::thread;

use crate::core::component::id::ComponentId;
use crate::core::security::traits::{SecurityAuditLogger, SecurityEvent};

/// Console-based security audit logger.
pub struct ConsoleSecurityAuditLogger {
    sender: Sender<SecurityEvent>,
}

impl ConsoleSecurityAuditLogger {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel::<SecurityEvent>();

        // Background thread for async logging
        thread::spawn(move || {
            while let Ok(event) = receiver.recv() {
                let status = if event.granted { "GRANTED" } else { "DENIED" };
                println!(
                    "[SECURITY] {} | {} | action={} resource={} | {}",
                    event.timestamp_ms,
                    event.component,
                    event.action,
                    event.resource,
                    status
                );
            }
        });

        Self { sender }
    }
}

impl SecurityAuditLogger for ConsoleSecurityAuditLogger {
    fn log_event(&self, event: SecurityEvent) {
        let _ = self.sender.send(event);
    }
}

/// Creates a security event for logging.
pub fn create_security_event(
    component: ComponentId,
    action: &str,
    resource: &str,
    granted: bool,
) -> SecurityEvent {
    SecurityEvent {
        component,
        action: action.to_string(),
        resource: resource.to_string(),
        granted,
        timestamp_ms: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64,
    }
}
```

**Tests:** 5 unit tests
- Create logger
- Create security event
- Log granted event
- Log denied event
- Thread-safety verification

---

### Action 2: Update `security/mod.rs`

**Objective:** Add audit module declaration

Add `pub mod audit;` to module declarations.

---

## Verification Commands

```bash
# 1. Build check
cargo build -p airssys-wasm

# 2. Lint check
cargo clippy -p airssys-wasm --all-targets -- -D warnings

# 3. Run audit tests
cargo test -p airssys-wasm --lib security::audit
```

---

## Success Criteria

- [ ] ConsoleSecurityAuditLogger implements SecurityAuditLogger
- [ ] Build passes with zero warnings
- [ ] Async logging works
- [ ] All unit tests pass
- [ ] create_security_event helper works
