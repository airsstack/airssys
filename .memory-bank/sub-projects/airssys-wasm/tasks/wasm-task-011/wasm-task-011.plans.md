# WASM-TASK-011: Implementation Plans

## Plan References

- **ADR-WASM-027:** WIT Interface Design (Validation Command section)
- **ADR-WASM-026:** Implementation Roadmap (Phase 1, Task 011)

## Implementation Actions

### Action 1: Run Complete Package Validation

**Objective:** Validate entire WIT package

**Steps:**
1. Run `wasm-tools component wit wit/core/`
2. Check for any errors or warnings
3. Verify all interfaces are recognized

**Reference:** ADR-WASM-027, lines 521-523

**Verification:**
```bash
cd airssys-wasm
wasm-tools component wit wit/core/
```

### Action 2: Verify File Count and Structure

**Objective:** Ensure all required files exist

**Steps:**
1. Verify 8 interface files exist (.wit files)
2. Verify deps.toml exists
3. Check directory structure

**Expected files:**
- types.wit
- errors.wit
- capabilities.wit
- component-lifecycle.wit
- host-messaging.wit
- host-services.wit
- storage.wit
- world.wit
- deps.toml

**Verification:**
```bash
ls -1 wit/core/*.wit | wc -l  # Should be 8
test -f wit/deps.toml && echo "✓ deps.toml exists"
```

### Action 3: Verify Interface Cross-References

**Objective:** Ensure all use statements resolve

**Steps:**
1. Check that errors.wit can import from types.wit
2. Check that all Layer 2+ files can import from Layer 0/1
3. Verify world.wit references all interfaces

## Verification Commands

```bash
cd airssys-wasm

# 1. Full validation
wasm-tools component wit wit/core/ && echo "✓ WIT package validated"

# 2. File count
test $(ls -1 wit/core/*.wit | wc -l) -eq 8 && echo "✓ All 8 WIT files present"

# 3. Package config
test -f wit/deps.toml && echo "✓ Package config exists"
```

## Success Criteria

- All verification commands pass
- No WIT errors or warnings
- Package structure complete
