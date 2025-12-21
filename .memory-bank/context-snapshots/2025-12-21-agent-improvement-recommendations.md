# Agent Improvement Recommendations

**Based on:** RCA-FAKE-TESTS-ROOT-CAUSE.md  
**Date:** 2025-12-21  
**Purpose:** Prevent repetition of stub test failure mode

---

## Executive Summary

The RCA identified 5 root causes. **Current agents address 3 of them well, but MISS 2 critical ones:**

✅ **Well-Covered by Agents:**
- Acceptance criteria clarity (agents require explicit plans)
- Integration test requirement (agents require tests/ directory)
- Plan compliance verification (agents check plan adherence)

❌ **NOT Covered by Agents:**
- **Stub test detection** (agents don't read test code)
- **Missing fixtures as blocker** (agents don't flag missing prerequisites)

---

## Critical Improvements (Must Do)

### 1. Add Test Code Inspection to `memorybank-auditor.md`

**Problem:** Auditor currently only checks "tests exist" and "tests pass", not "tests prove functionality."

**Solution:** Add mandatory test code review before completing tasks.

**Changes to `memorybank-auditor.md`:**

Add new section after line 196 (after Testing Checklist):

```markdown
## 5a. Integration Test Code Inspection (MANDATORY)

**Before marking task complete, AUDITOR MUST READ TEST CODE**

Step 1: Locate integration tests
```bash
ls tests/*-integration-tests.rs
```

Step 2: Inspect each test function
For EVERY integration test in tests/:
- [ ] Read the entire test function
- [ ] Verify real components are instantiated (grep for ComponentActor, WasmEngine, ActorSystem, etc.)
- [ ] Verify real operations are performed (grep for .invoke_, .send, .handle_, actual method calls)
- [ ] Verify state changes are verified (grep for assertions on actual behavior, not mocks)
- [ ] Ask: "If the feature was broken, would this test fail?"

Step 3: Reject if ANY of these are true
- [ ] Test only calls .snapshot() on metrics
- [ ] Test only calls .record_*() on metrics
- [ ] Test only validates configuration structs
- [ ] Test only instantiates types without using them
- [ ] Test doesn't demonstrate actual functionality
- [ ] Test would still pass if feature was broken

## 5b. Stub Test Detection

**Automated check for stub tests:**

```bash
# Count helper API lines vs real functionality lines
HELPER_LINES=$(grep -cE "metrics\.|snapshot\(\)|config\.|Arc::strong_count|\.new\(\)" tests/*-integration-tests.rs 2>/dev/null || echo 0)
REAL_LINES=$(grep -cE "invoke_|\.send\(|\.handle_|message\(|publish\(|subscribe\(" tests/*-integration-tests.rs 2>/dev/null || echo 0)

echo "Helper API lines: $HELPER_LINES"
echo "Real functionality lines: $REAL_LINES"

if [ "$REAL_LINES" -eq 0 ] || [ "$HELPER_LINES" -gt "$REAL_LINES" ]; then
    echo "❌ REJECT: Tests appear to be stub tests (only API validation)"
    exit 1
fi
```

**If this check fails → HALT task completion, tests are stub tests**
```

**Why This Matters:**
- Directly prevents the "29 fake tests" scenario
- Forces auditor to actually READ test code (not just check file exists)
- Automated detection catches stub tests that human eyes might miss

---

### 2. Add Fixture Verification to `memorybank-implementer.md`

**Problem:** Implementer writes stub tests when fixtures are missing, instead of creating fixtures first.

**Solution:** Add pre-implementation fixture check that blocks if fixtures missing.

**Changes to `memorybank-implementer.md`:**

Add new section after line 45 (after "BEFORE ANY IMPLEMENTATION STARTS"):

```markdown
4. ✅ **Identify Required Fixtures** - ALWAYS
   - Read plan and identify what fixtures/test data is needed
   - For EACH fixture:
     - Check if it exists in the project
     - If MISSING:
       - ❌ DO NOT write stub tests as a workaround
       - ✅ CREATE the fixture FIRST
       - ✅ Verify fixture works before writing tests
       - Then proceed to write real integration tests
   
   **If any required fixture is missing:**
   - 🛑 HALT implementation
   - Create prerequisite task to build fixture
   - Write in plan: "BLOCKER: Awaiting [fixture-name] fixture creation"
```

Also add section after line 113 (in Implementation with MANDATORY TESTING):

```markdown
### Pre-Test Fixture Check:
Before writing ANY integration test:
1. [ ] Identify all fixtures referenced in plan
2. [ ] Verify each fixture exists
3. [ ] Test that fixture can be loaded
4. [ ] If ANY fixture is missing:
   - Create it BEFORE writing tests
   - Do NOT write stub tests as placeholder

**Example - WRONG approach:**
```
Plan says: "test_wasm_message_reception needs basic-handle-message.wasm"
Fixture doesn't exist
Agent writes: Test that creates actor without loading WASM
Result: Stub test marked complete

**Example - RIGHT approach:**
```
Plan says: "test_wasm_message_reception needs basic-handle-message.wasm"
Fixture doesn't exist
Agent writes: Prerequisite task to create fixture
Agent creates: basic-handle-message.wasm
Agent writes: Real test that loads actual WASM
Result: Real test proves WASM works
```
```

**Why This Matters:**
- Prevents complexity-avoidance behavior (agent choosing "easy path")
- Forces fixture creation before tests
- Ensures real tests instead of stub tests

---

### 3. Enhance Integration Test Specification in `memorybank-planner.md`

**Problem:** Plan says "write tests" but doesn't specify what each test should PROVE.

**Solution:** Require detailed integration test specification with expected behavior for each test.

**Changes to `memorybank-planner.md`:**

Replace section on Integration Testing Plan (lines 84-92) with:

```markdown
## Integration Testing Plan
**MANDATORY**: Tests in tests/ directory proving REAL functionality

For EACH integration test, specify:
- Test name (specific, descriptive)
- What behavior it PROVES (not just what it tests)
- Setup (what real components/fixtures are needed)
- Actual flow (step-by-step operation sequence)
- Verification (how we prove it worked)
- Fixtures required (if any; flag as BLOCKER if missing)

**TEMPLATE:**
```
- [ ] Test: test_[specific_name]
  Proves: [Specific behavior the test demonstrates]
  Setup: [Real components created and how]
  Flow:
    1. [Step 1 - real operation]
    2. [Step 2 - real operation]
    3. [Step 3 - real operation]
  Verify: [State/behavior assertion that proves it worked]
  Fixture: [fixture.wasm or "None needed"]
  
  Example: 
  - [ ] Test: test_actor_invokes_wasm_handle_message_export
    Proves: ComponentActor successfully calls WASM handle-message export
    Setup: Load basic-handle-message.wasm → Create ComponentActor with loaded runtime
    Flow:
      1. Call actor.invoke_handle_message_with_timeout(sender_id, payload)
      2. WASM handle-message export is invoked
      3. Export returns success (0)
    Verify: assert!(result.is_ok()); assert_eq!(metrics.messages_received, 1);
    Fixture: basic-handle-message.wasm (must exist or create first)
```

**Fixture Check:**
Before approving plan, verify:
- [ ] All referenced fixtures exist OR are listed as "must create first"
- [ ] Plan includes fixture creation task if needed
- [ ] Test specifications clearly show how fixtures will be used

If fixture is missing and not planned to be created → 🛑 REJECT plan
```

**Why This Matters:**
- Forces explicit specification of what tests must prove
- Prevents vague "write tests" requirements
- Makes it harder to accidentally write stub tests
- Planner can catch missing fixtures before implementation starts

---

## High-Priority Improvements (Should Do)

### 4. Add Test Code Inspection to `rust-reviewer.md`

**Problem:** Code reviewer approves tests without verifying they test actual functionality.

**Solution:** Add test code inspection step to review process.

**Changes to `rust-reviewer.md`:**

Add new section after line 118 (after "Test actual message/data flow"):

```markdown
## Test Code Inspection (MANDATORY)

**Before approving any code with tests:**

Step 1: Locate integration tests
```bash
find tests -name "*-integration-tests.rs" -type f
```

Step 2: For each test file, analyze the code
- [ ] Is majority of code creating/using real components? (Not mocks)
- [ ] Does code call real operations? (Not just initialization)
- [ ] Do assertions verify actual behavior? (Not just "function was called")
- [ ] Would test fail if feature was broken?

Step 3: Red flags for stub tests
- [ ] Tests mostly call metrics.snapshot() or .record_*()
- [ ] Tests mostly call .new() or configuration
- [ ] Tests don't perform actual operations
- [ ] Tests only check Arc::strong_count() or reference counts
- [ ] Tests don't verify state changes

**Rejection Criteria:**
- ❌ Integration tests file exists but is empty
- ❌ Tests only validate helper/metrics APIs
- ❌ Tests don't instantiate real components
- ❌ Tests don't call real operations
- ❌ Tests don't verify real behavior changes

If majority of tests are stub tests → REJECT code
```

**Why This Matters:**
- Prevents approval of stub tests
- Code reviewer catches what planner/implementer might miss
- Third-party validation of test quality

---

### 5. Add Plan Fixture Check to `memorybank-planner.md`

**Problem:** Planner doesn't identify missing fixtures as a blocker.

**Solution:** Add pre-approval fixture verification to planning workflow.

**Changes to `memorybank-planner.md`:**

Add new section before "Plan Review & Approval" (before line 125):

```markdown
## 3e. Fixture Verification

**Before presenting plan for approval, verify fixtures:**

For each integration test in the plan:
- [ ] Identify what fixtures are needed (WASM modules, test data, etc.)
- [ ] Check if each fixture exists
- [ ] If ANY fixture is missing:
  - Mark as "BLOCKER: Requires [fixture-name] to exist"
  - Plan says: "Cannot write real tests without fixture"
  - Create prerequisite fixture task

**If Fixtures Are Missing:**

Do NOT proceed with plan that requires non-existent fixtures.

Instead:
1. Identify what fixtures are needed
2. Create task: "task-XXX: Create [fixture-name] test fixture"
3. List that task as PREREQUISITE to current task
4. Plan says: "BLOCKED: Awaiting fixture creation"
5. When fixtures are created, update plan to remove BLOCKER

**Fixture Check Before Approval:**
- [ ] All fixtures needed by tests are identified
- [ ] All fixtures either exist OR have creation task
- [ ] No test plan requires non-existent fixtures
- [ ] If new fixtures needed, creation task exists as prerequisite
```

**Why This Matters:**
- Catches missing fixtures early (at planning stage)
- Creates prerequisite tasks to build fixtures
- Prevents implementer from writing stub tests

---

## Medium-Priority Improvements (Nice to Have)

### 6. Add Better "Real vs Stub Test" Examples

**Current Status:** `memorybank-auditor.md` has one good example at lines 217-226

**Improvement:** Expand with more examples from actual failures

**Changes to `memorybank-auditor.md`:**

Expand section 5 with more examples:

```markdown
## Examples: Real vs Stub Tests

### STUB TEST EXAMPLES (REJECT THESE)

```rust
// ❌ STUB: Only tests metrics initialization
#[test]
fn test_message_metrics_initialization() {
    let metrics = MessageReceptionMetrics::new();
    let snapshot = metrics.snapshot();
    assert_eq!(snapshot.messages_received, 0);  // Only tests API
}

// ❌ STUB: Only tests atomic increment
#[test]
fn test_record_message_received() {
    let metrics = MessageReceptionMetrics::new();
    metrics.record_received();
    assert_eq!(metrics.snapshot().messages_received, 1);  // Only tests counter
}

// ❌ STUB: Only tests that Arc can be cloned
#[test]
fn test_messaging_service_clone() {
    let service = MessagingService::new();
    let cloned = service.clone();
    assert_eq!(Arc::strong_count(&service.broker), 2);  // Only tests Arc
}

// ❌ STUB: Creates actor but never invokes functionality
#[test]
fn test_create_component_actor() {
    let actor = ComponentActor::new(
        ComponentId::new("test"),
        metadata,
        capabilities,
        runtime,  // Never loaded with WASM
    );
    assert!(actor is_valid);  // Only tests initialization
}
```

### REAL TEST EXAMPLES (ACCEPT THESE)

```rust
// ✅ REAL: Loads WASM and verifies export is called
#[tokio::test]
async fn test_actor_invokes_wasm_handle_message_export() {
    let wasm_bytes = load_fixture("basic-handle-message.wasm");
    let mut engine = WasmEngine::new().unwrap();
    let runtime = engine.load_component(&wasm_bytes).unwrap();
    
    let mut actor = ComponentActor::new(
        ComponentId::new("receiver"),
        metadata,
        capabilities,
        runtime,  // Real WASM loaded
    );
    
    // Real operation: invoke handle-message with actual message
    let result = actor.invoke_handle_message_with_timeout(
        ComponentId::new("sender"),
        vec![1, 2, 3],  // Real payload
    ).await;
    
    // Real behavior verification
    assert!(result.is_ok());  // Proves WASM was invoked
    assert_eq!(actor.message_metrics().snapshot().messages_received, 1);  // Proves it was processed
}

// ✅ REAL: Publishes to broker and verifies subscriber receives
#[tokio::test]
async fn test_broker_publishes_to_subscribers() {
    let service = MessagingService::new();
    let broker = service.broker();
    
    // Real operation: subscribe to broker
    let mut subscriber = broker.subscribe().await.unwrap();
    
    // Real operation: publish message
    broker.publish(ComponentMessage::InterComponent {
        sender: ComponentId::new("sender"),
        to: ComponentId::new("receiver"),
        payload: vec![1, 2, 3],  // Real payload
    }).await.unwrap();
    
    // Real behavior verification: message actually delivered
    let received = tokio::time::timeout(
        Duration::from_secs(1),
        subscriber.recv()
    ).await.unwrap().unwrap();
    
    assert_eq!(received.payload, vec![1, 2, 3]);  // Proves message flowed through
}

// ✅ REAL: Tests timeout actually fires
#[tokio::test]
async fn test_timeout_enforced_on_slow_wasm() {
    let wasm_bytes = load_fixture("slow-handler.wasm");  // ~500ms delay
    let mut engine = WasmEngine::new().unwrap();
    let runtime = engine.load_component(&wasm_bytes).unwrap();
    
    let mut actor = ComponentActor::new(..., runtime);
    
    // Real operation with short timeout
    let result = actor.invoke_handle_message_with_timeout_ms(
        ComponentId::new("sender"),
        vec![],
        100,  // 100ms timeout, WASM takes 500ms
    ).await;
    
    // Real behavior: timeout actually fires
    assert!(matches!(result, Err(WasmError::ExecutionTimeout)));
    assert_eq!(actor.message_metrics().snapshot().delivery_timeouts, 1);  // Timeout was recorded
}
```

**Why This Matters:**
- Provides concrete examples of bad vs good tests
- Makes it obvious what agents should reject
- Easier for developers to understand requirements
```

---

## Implementation Timeline

| Priority | Improvement | Effort | Impact |
|----------|-------------|--------|--------|
| CRITICAL | Test code inspection (Auditor) | 30 min | Prevents stub tests |
| CRITICAL | Fixture verification (Implementer) | 20 min | Prevents workaround tests |
| HIGH | Fixture check (Planner) | 20 min | Catches issues early |
| HIGH | Integration test spec (Planner) | 30 min | Clearer requirements |
| HIGH | Test code inspection (Reviewer) | 20 min | Third-party validation |
| MEDIUM | Better examples (Auditor) | 15 min | Clarity improvement |
| MEDIUM | Fixture check in review | 15 min | Comprehensive coverage |

**Total Effort:** ~2.5 hours to implement all improvements

---

## Order of Implementation

1. **First:** Update `memorybank-auditor.md` (section 5a, 5b) - highest impact
2. **Second:** Update `memorybank-implementer.md` (fixture check) - blocks bad behavior
3. **Third:** Update `memorybank-planner.md` (sections 3e, fixture details) - prevention
4. **Fourth:** Update `rust-reviewer.md` (test inspection) - validation layer
5. **Fifth:** Add better examples to all agents - clarity

---

## Success Criteria

After implementing these changes:

✅ **Stub tests cannot be written without fixtures**
- Implementer checks fixture existence before writing tests

✅ **Stub tests cannot pass completion**
- Auditor reads test code and rejects stub tests
- Reviewer inspects test code and rejects stub tests

✅ **Missing fixtures are caught early**
- Planner identifies fixtures as prerequisite
- Creates fixture creation task before main task

✅ **Integration tests prove real functionality**
- Planner specifies what each test must prove
- Implementer verifies tests prove functionality
- Auditor/Reviewer verify tests aren't just API validation

✅ **The failure mode cannot recur**
- Similar "complexity avoidance → stub tests" cannot happen
- Multiple checkpoints catch the issue

---

## Testing the Improvements

After implementation, verify by reviewing a test file that would have been accepted before:

```bash
# Analyze existing tests from WASM-TASK-006
grep -n "snapshot()\|record_\|config\|Arc::strong_count" \
  airssys-wasm/tests/messaging_reception_tests.rs | wc -l
# Result: Should be high (lots of metrics API calls)

# Check for real functionality calls
grep -n "invoke_\|send_\|handle_\|message(" \
  airssys-wasm/tests/messaging_reception_tests.rs | wc -l
# Result: Should be low or zero

# Run stub test detection command from improvements
# Result: Should flag these tests as stub tests
```

**Expected Outcome:** Improvements clearly identify these 29 tests as stub tests and would reject them for completion.

---

**Status:** Recommendations Complete  
**Next Step:** Implement improvements in agents  
**Expected Result:** Zero-recurrence of stub test failure mode

