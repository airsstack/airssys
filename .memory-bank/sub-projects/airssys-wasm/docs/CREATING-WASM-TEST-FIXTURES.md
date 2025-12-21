# CREATING REAL WASM BINARY TEST FIXTURES

## Summary: YES, 100% Possible! Multiple Approaches Available

You already have:
✅ WAT files (WebAssembly Text format)
✅ Build script to compile them
✅ One test fixture (hello_world.wasm)

We can create realistic test fixtures in 3 ways:

---

## APPROACH 1: Write WAT Files (WebAssembly Text Format) ⭐ RECOMMENDED

### What is WAT?

WAT is human-readable WebAssembly. You write it in text format and compile it to WASM binary.

**Example 1: Minimal handle-message export**

```wast
;; tests/fixtures/handle-message-component.wat
;; Component with handle-message export for testing message reception

(component
  (core module $HandleMessage
    ;; A simple memory for testing (10 pages = 640KB)
    (memory (export "memory") 10)
    
    ;; Store data function - test can write message data here
    (func $store-data (export "store-data") (param $ptr i32) (param $len i32)
      ;; This function just returns success
      ;; In real WASM, this would process the message
      i32.const 0  ;; Return code 0 = success
    )
    
    ;; The main handle-message export
    ;; Parameters: sender_id (i32), message_ptr (i32), message_len (i32)
    ;; Returns: error_code (i32) - 0 = success
    (func $handle_message (export "handle-message") 
      (param $sender_id i32) 
      (param $msg_ptr i32) 
      (param $msg_len i32)
      (result i32)
      ;; Read message from ptr/len
      ;; Process it (in test: just return success)
      (call $store-data (local.get $msg_ptr) (local.get $msg_len))
    )
    
    ;; Initialization export
    (func (export "_start"))
  )
  
  ;; Instantiate core module
  (core instance $m (instantiate $HandleMessage))
  
  ;; Lift handle-message to component level
  (func (export "handle-message") 
    (param "sender" (ref extern)) 
    (param "message" (list u8))
    (result (result (tuple) (tuple)))
    ;; Call core function through component interface
    (canon lift (core func $m "handle-message"))
  )
  
  ;; Lift _start to component level  
  (func (export "_start")
    (canon lift (core func $m "_start"))
  )
)
```

**Example 2: Slow handle-message (for timeout testing)**

```wast
;; tests/fixtures/slow-handle-message-component.wat
;; Component that takes 1 second to process (for timeout tests)

(component
  (core module $SlowHandler
    (memory (export "memory") 1)
    
    ;; Spin loop to delay execution
    (func $delay (param $iterations i32)
      (local $i i32)
      (loop $continue
        (local.set $i (i32.add (local.get $i) (i32.const 1)))
        (if (i32.lt_u (local.get $i) (local.get $iterations))
          (then (br $continue))
        )
      )
    )
    
    ;; Handle message slowly
    (func $handle_message (export "handle-message")
      (param $sender_id i32)
      (param $msg_ptr i32) 
      (param $msg_len i32)
      (result i32)
      ;; Delay for ~1 second (approximate)
      (call $delay (i32.const 100000000))
      i32.const 0  ;; Return success after delay
    )
    
    (func (export "_start"))
  )
  
  (core instance $m (instantiate $SlowHandler))
  
  (func (export "handle-message")
    (param "sender" (ref extern))
    (param "message" (list u8))
    (result (result (tuple) (tuple)))
    (canon lift (core func $m "handle-message"))
  )
  
  (func (export "_start")
    (canon lift (core func $m "_start"))
  )
)
```

**Example 3: Failing handle-message (for error testing)**

```wast
;; tests/fixtures/failing-handle-message-component.wat
;; Component that always rejects messages

(component
  (core module $FailingHandler
    (memory (export "memory") 1)
    
    (func $handle_message (export "handle-message")
      (param $sender_id i32)
      (param $msg_ptr i32)
      (param $msg_len i32)
      (result i32)
      i32.const 99  ;; Return error code 99 (rejection)
    )
    
    (func (export "_start"))
  )
  
  (core instance $m (instantiate $FailingHandler))
  
  (func (export "handle-message")
    (param "sender" (ref extern))
    (param "message" (list u8))
    (result (result (tuple) (tuple)))
    (canon lift (core func $m "handle-message"))
  )
  
  (func (export "_start")
    (canon lift (core func $m "_start"))
  )
)
```

### Compilation

You have `build.sh` that does this:

```bash
#!/bin/bash
# Compile WAT files to WASM binary
for wat_file in tests/fixtures/*.wat; do
    wasm_file="${wat_file%.wat}.wasm"
    wasm-tools parse "$wat_file" -o "$wasm_file"
done
```

**Commands:**

```bash
# Compile ALL WAT files
cd /Users/hiraq/Projects/airsstack/airssys
./airssys-wasm/tests/fixtures/build.sh

# Or compile one
wasm-tools parse airssys-wasm/tests/fixtures/handle-message-component.wat \
  -o airssys-wasm/tests/fixtures/handle-message-component.wasm

# Verify it compiled
file airssys-wasm/tests/fixtures/handle-message-component.wasm
```

### Advantages

✅ **Easy to write** - text format is readable  
✅ **Version control friendly** - text in git  
✅ **Customizable** - change behavior easily  
✅ **Debuggable** - comments explain what's happening  
✅ **No external tools** - wasm-tools is just one cargo install  

---

## APPROACH 2: Compile Rust to WASM (More Complex)

### What You'd Do

Write Rust code, compile to WASM:

```rust
// tests/fixtures/src/lib.rs
#[no_mangle]
pub extern "C" fn handle_message(
    sender_id: i32,
    msg_ptr: i32,
    msg_len: i32,
) -> i32 {
    // Process message
    0  // Return success
}

#[no_mangle]
pub extern "C" fn _start() {}
```

Compile with:
```bash
rustup target add wasm32-unknown-unknown
cargo build --target wasm32-unknown-unknown --release \
  -p test-wasm-fixtures
```

### Advantages
✅ More sophisticated logic possible  
✅ Access to Rust ecosystem  

### Disadvantages
❌ Slower to build  
❌ Requires cargo setup  
❌ Overkill for simple test cases  

---

## APPROACH 3: Use Existing hello_world.wasm (Quick Start)

You already have:
```
airssys-wasm/tests/fixtures/hello_world.wasm (139 bytes)
```

**Advantages:**
✅ Already exists  
✅ No compilation needed  

**Disadvantages:**
❌ Doesn't have handle-message export  
❌ Not suitable for message reception tests  

---

## RECOMMENDATION: Hybrid Approach

1. **Use WAT files for most tests** ← START HERE
   - Write simple WAT files for each test scenario
   - Build them with existing build.sh
   - Fast iteration, easy debugging

2. **Use Rust WASM for complex scenarios** (later if needed)
   - Only if WAT can't express the logic needed

3. **Keep hello_world.wasm** for basic tests

---

## How to Create Test Fixtures RIGHT NOW

### Step 1: Create WAT Files

Create these files in `airssys-wasm/tests/fixtures/`:

**File 1: basic-handle-message.wat**
```wast
(component
  (core module $BasicHandler
    (memory (export "memory") 1)
    
    (func $handle_message (export "handle-message")
      (param $sender_id i32) (param $msg_ptr i32) (param $msg_len i32)
      (result i32)
      i32.const 0  ;; Success
    )
    
    (func (export "_start"))
  )
  
  (core instance $m (instantiate $BasicHandler))
  
  (func (export "handle-message")
    (param "sender" (ref extern)) (param "message" (list u8))
    (result (result (tuple) (tuple)))
    (canon lift (core func $m "handle-message"))
  )
  
  (func (export "_start")
    (canon lift (core func $m "_start"))
  )
)
```

**File 2: slow-handler.wat**
```wast
(component
  (core module $SlowHandler
    (memory (export "memory") 1)
    
    (func $delay (param $n i32)
      (local $i i32)
      (loop $loop
        (if (i32.lt_u (local.get $i) (local.get $n))
          (then
            (local.set $i (i32.add (local.get $i) (i32.const 1)))
            (br $loop)
          )
        )
      )
    )
    
    (func $handle_message (export "handle-message")
      (param $sender_id i32) (param $msg_ptr i32) (param $msg_len i32)
      (result i32)
      (call $delay (i32.const 50000000))  ;; ~500ms delay
      i32.const 0
    )
    
    (func (export "_start"))
  )
  
  (core instance $m (instantiate $SlowHandler))
  
  (func (export "handle-message")
    (param "sender" (ref extern)) (param "message" (list u8))
    (result (result (tuple) (tuple)))
    (canon lift (core func $m "handle-message"))
  )
  
  (func (export "_start")
    (canon lift (core func $m "_start"))
  )
)
```

**File 3: rejecting-handler.wat**
```wast
(component
  (core module $RejectingHandler
    (memory (export "memory") 1)
    
    (func $handle_message (export "handle-message")
      (param $sender_id i32) (param $msg_ptr i32) (param $msg_len i32)
      (result i32)
      i32.const 99  ;; Error code
    )
    
    (func (export "_start"))
  )
  
  (core instance $m (instantiate $RejectingHandler))
  
  (func (export "handle-message")
    (param "sender" (ref extern)) (param "message" (list u8))
    (result (result (tuple) (tuple)))
    (canon lift (core func $m "handle-message"))
  )
  
  (func (export "_start")
    (canon lift (core func $m "_start"))
  )
)
```

### Step 2: Build Them

```bash
cd /Users/hiraq/Projects/airsstack/airssys
./airssys-wasm/tests/fixtures/build.sh
```

### Step 3: Load in Tests

```rust
#[tokio::test]
async fn test_message_reception() {
    // Load fixture
    let wasm_bytes = std::fs::read(
        "airssys-wasm/tests/fixtures/basic-handle-message.wasm"
    ).unwrap();
    
    // Load component
    let mut engine = WasmEngine::new().unwrap();
    let runtime = engine.load_component(&wasm_bytes).unwrap();
    
    // Create actor with real WASM
    let mut actor = ComponentActor::new(
        ComponentId::new("receiver"),
        metadata,
        capabilities,
        runtime,
    );
    
    // Send message - REAL TEST
    let result = actor.invoke_handle_message_with_timeout(
        ComponentId::new("sender"),
        vec![1, 2, 3],
    ).await;
    
    // Verify it worked
    assert!(result.is_ok());
}
```

---

## Quick Start Command

```bash
# Install wasm-tools if not already
cargo install wasm-tools

# Go to project
cd /Users/hiraq/Projects/airsstack/airssys

# Compile existing fixtures
./airssys-wasm/tests/fixtures/build.sh

# Verify
ls -l airssys-wasm/tests/fixtures/*.wasm
```

---

## Size Reference

These WASM files are TINY:

```
hello_world.wasm          139 bytes
basic-handle-message.wasm ~300-400 bytes
slow-handler.wasm         ~400-500 bytes
rejecting-handler.wasm    ~300 bytes
```

All test fixtures combined < 2KB!

---

## Summary: What You Need to Do

To create REAL test fixtures:

1. **Write 3-4 simple WAT files** (30 min)
   - basic-handle-message.wat (success case)
   - slow-handler.wat (timeout testing)
   - rejecting-handler.wat (error testing)
   - Optional: echo-handler.wat (returns message content)

2. **Run build.sh** (1 min)
   ```bash
   ./airssys-wasm/tests/fixtures/build.sh
   ```

3. **Use in tests** (2-3 hours)
   - Load WASM bytes
   - Create ComponentActor with real WASM
   - Send actual messages
   - Verify behavior

**Total: ~3-4 hours to get REAL, working integration tests**

---

## Yes/No Question Summary

**Is it possible to create REAL WASM BINARY FILE TO BE USED AS TEST FIXTURES?**

✅ **YES, 100% ABSOLUTELY POSSIBLE**

You can:
- Write WAT files (easiest)
- Compile them to WASM binary
- Use them in tests
- All within 3-4 hours

The only reason the tests say "requires test WASM fixtures" is because nobody wrote them yet. It's not hard - it's just a few lines of WAT per fixture.

