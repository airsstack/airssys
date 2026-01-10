# WASM-TASK-030: Implementation Plans

## Plan References
- **ADR-WASM-029:** Security Module Design
- **ADR-WASM-025:** Clean-Slate Rebuild Architecture
- **ADR-WASM-026:** Implementation Roadmap (Phase 4)
- **PROJECTS_STANDARD.md:** §6.4 Quality Gates

## Overview

Comprehensive unit testing for the security module following Phase 4 implementation.

---

## Implementation Actions

### Action 1: Review and Enhance capability/types.rs Tests

**Objective:** Ensure PatternMatcher has comprehensive tests

**Test Coverage:**
- Wildcard matching (`*`)
- Prefix patterns (`org.example/*`)
- Exact matching
- Edge cases (empty strings, special characters)
- Non-matching patterns

---

### Action 2: Review and Enhance capability/set.rs Tests

**Objective:** Ensure CapabilitySet has comprehensive tests

**Test Coverage:**
- Create empty set
- Add each permission type
- Check permission granted
- Check permission denied
- Pattern-based permission matching
- Multiple permissions

---

### Action 3: Review and Enhance capability/validator.rs Tests

**Objective:** Ensure CapabilityValidator has comprehensive tests

**Test Coverage:**
- Create validator
- Register/unregister components
- Validate messaging capability (granted/denied)
- Validate storage capability (granted/denied)
- can_send_to (granted/denied)
- Unregistered component error
- Thread-safety with concurrent access

---

### Action 4: Review and Enhance policy/ Tests

**Objective:** Ensure PolicyEngine and rules have comprehensive tests

**Test Coverage:**
- Create policy and rules
- Policy applies_to matching
- Policy evaluate Allow/Deny
- PolicyEngine with multiple policies
- Complex rule evaluation scenarios

---

### Action 5: Review and Enhance audit.rs Tests

**Objective:** Ensure audit logger has comprehensive tests

**Test Coverage:**
- Create logger
- Create security event helper
- Log events (granted/denied)
- Verify thread handles events

---

### Action 6: Measure Code Coverage

**Objective:** Verify >80% coverage

```bash
# Install cargo-tarpaulin if needed
cargo install cargo-tarpaulin

# Run coverage for security module
cargo tarpaulin -p airssys-wasm --lib --ignore-tests -- --test-threads=1 2>&1 | grep -A5 "security"
```

---

## Verification Commands

```bash
# 1. Run all security tests
cargo test -p airssys-wasm --lib security

# 2. Lint check
cargo clippy -p airssys-wasm --all-targets -- -D warnings

# 3. Coverage check
cargo tarpaulin -p airssys-wasm --lib
```

---

## Success Criteria

- [ ] All security module tests pass
- [ ] >80% code coverage
- [ ] Edge cases covered
- [ ] Error paths tested
- [ ] Thread-safety verified
- [ ] Phase 4 ready for completion
