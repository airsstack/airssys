# ADR-WASM-001: Multicodec Compatibility Strategy

**Status:** Accepted  
**Date:** 2025-10-19  
**Decision Makers:** Architecture Team  
**Related:** KNOWLEDGE-WASM-006 (Multiformat Strategy), KNOWLEDGE-WASM-011 (Serialization Strategy)

---

## Context

The airssys-wasm framework needs to support inter-component communication where components may be written in different programming languages (Rust, JavaScript, Python, Go, etc.) and may use different serialization formats. The framework uses multicodec prefixes (from Protocol Labs multiformats specification) to enable self-describing data.

### The Problem

When Component A sends a message to Component B, they may support different serialization codecs:

```
Component A (Rust)       → Supports: borsh, MessagePack
Component B (JavaScript) → Supports: borsh only

Scenario: Component A sends MessagePack message to Component B
Question: Should the host runtime translate MessagePack → borsh?
```

### Key Questions

1. **Should the host runtime provide codec translation?**
   - Translate between formats automatically?
   - Or let components handle their own codec support?

2. **What codecs should the host runtime support?**
   - All possible codecs (borsh, bincode, MessagePack, Protobuf, etc.)?
   - Or just validate multicodec prefixes without implementation?

3. **How to handle codec incompatibility?**
   - Fail fast with clear errors?
   - Silent translation with potential data loss?
   - Negotiate common codec?

### Requirements

**Functional:**
- Components can use different serialization formats
- Self-describing data via multicodec prefixes
- Cross-language component communication
- Clear error handling for incompatible codecs

**Non-Functional:**
- Performance: Minimal overhead for message routing
- Simplicity: Maintainable host runtime
- Safety: No silent data corruption
- Flexibility: Components can add new codecs

---

## Decision

**The host runtime will validate codec compatibility but will NOT translate between codecs.**

### Approach: Codec Compatibility Validation (No Translation)

**Host Responsibilities:**
1. ✅ Parse multicodec prefixes (detect codec from message bytes)
2. ✅ Maintain multicodec ID registry (known codec identifiers)
3. ✅ Validate component codec declarations (from Component.toml manifests)
4. ✅ Check compatibility at message send time (fail fast if incompatible)
5. ✅ Provide clear error messages (indicate supported codecs)
6. ✅ Help discover common codecs (suggest compatible formats)
7. ✅ Route messages as opaque bytes (no decoding/encoding)

**Host Does NOT:**
1. ❌ Translate between codecs (no MessagePack → borsh conversion)
2. ❌ Implement codec serialization/deserialization
3. ❌ Depend on codec libraries (borsh, bincode, etc.)
4. ❌ Force components to use specific codec
5. ❌ Silent codec conversion (no hiding incompatibility)

**Component Responsibilities:**
1. ✅ Declare supported codecs in manifest (Component.toml)
2. ✅ Implement serialization/deserialization for their codecs
3. ✅ Choose codec when sending messages
4. ✅ Handle decode errors gracefully
5. ✅ Use compatible codecs for inter-component messaging

---

## Rationale

### Why NOT Translate Between Codecs

**1. Technical Complexity & Data Loss Risk**

Type-safe codec translation is nearly impossible without schema knowledge:

```rust
// Problem: Host doesn't know the type being serialized
let msgpack_bytes = [...]; // MessagePack encoded data

// To translate, host would need to:
// 1. Deserialize MessagePack → ??? (what type?)
// 2. Re-serialize ??? → borsh

// Without type information, translation is impossible or lossy
```

**Example of Potential Data Loss:**
```javascript
// JavaScript component with MessagePack
{
  name: "test",
  count: undefined,  // JavaScript undefined
}
// MessagePack: encodes undefined as nil

// If host translates to borsh:
// - borsh has no nil type
// - Represents undefined as Option::None? (requires type info)
// - Skip field entirely? (changes structure)
// - Error? (breaks compatibility)
// Result: AMBIGUOUS, potential data corruption
```

**2. Performance Overhead**

Translation adds significant overhead:

```
Without Translation:
  Component A → serialize (100ns) 
  → Host routes (pass-through) 
  → Component B → deserialize (100ns)
  Total: 200ns

With Translation:
  Component A → serialize (100ns)
  → Host deserialize (100ns)
  → Host analyze type (???)
  → Host re-serialize (100ns)
  → Component B → deserialize (100ns)
  Total: 400ns + type analysis

Performance hit: 2-3x slower (150% overhead)
```

At scale:
- 100,000 messages/sec: +30ms latency from translation overhead
- Throughput reduction: 50%

**3. Tight Coupling & Maintenance Burden**

If host translates, it must:
- Depend on ALL codec libraries (bloated dependencies)
- Implement translation logic for each codec pair (N² complexity)
- Update when codecs change (breaking changes)
- Maintain complex translation code (2000+ lines vs 200 lines)

**4. Production Safety & Clear Errors**

**Fail Fast Approach (Our Decision):**
```
Component A sends MessagePack to Component B
Host checks: Component B supports [borsh] only
ERROR: "Component B doesn't support MessagePack (0x0201).
        Supported: borsh (0x0701).
        Use borsh for compatibility."
```

**Silent Translation Approach (Rejected):**
```
Component A sends MessagePack to Component B
Host silently translates MessagePack → borsh
Potential issues:
  - Data loss (undefined → ?)
  - Type mismatches (silent corruption)
  - Performance degradation (no visibility)
  - Debugging nightmare ("it works in dev, breaks in prod")
```

### Why This Approach Works

**1. Clear Responsibility Model**

```
Components: Own their codec implementations
Host:       Validate compatibility, route bytes
Pattern:    Fail fast with clear error messages
```

**2. Alignment with Industry Practices**

**HTTP Content Negotiation:**
```http
Accept: application/json, application/xml
Content-Type: application/json
# Server picks compatible format, no translation
```

**gRPC:**
- Single format (Protocol Buffers)
- No negotiation or translation
- Simple: one codec for all

**NEAR Protocol / Solana:**
- All smart contracts use borsh (mandatory)
- No codec negotiation
- Production-proven simplicity

**3. Performance & Simplicity**

```rust
// Host implementation (simple)
pub fn route_message(
    sender: ComponentId,
    receiver: ComponentId,
    message: Vec<u8>,
) -> Result<()> {
    // 1. Detect codec (just read prefix)
    let codec = Multicodec::from_prefix(&message)?;
    
    // 2. Check compatibility
    self.compatibility_checker.check(sender, receiver, codec)?;
    
    // 3. Route bytes (no decode/encode!)
    self.deliver_to_component(receiver, message).await
}
```

No translation = no overhead, no complexity, no data loss risk.

---

## Implementation

### Component Manifest Declaration

```toml
# Component.toml

[component]
name = "my-component"
version = "1.0.0"

[serialization]
# Codecs this component can decode (receive messages)
supported_codecs = ["borsh", "bincode"]

# Codec this component prefers when sending messages
preferred_codec = "borsh"
```

### Host Multicodec Registry

```rust
// src/core/multicodec/mod.rs

/// Multicodec ID registry (no implementation, just IDs)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u16)]
pub enum Multicodec {
    /// borsh - Deterministic, cross-language
    Borsh = 0x0701,
    
    /// bincode - Fast Rust-only serialization
    Bincode = 0x0702,
    
    /// MessagePack (can add without dependency!)
    MessagePack = 0x0201,
    
    /// Protocol Buffers
    Protobuf = 0x0050,
}

impl Multicodec {
    /// Parse from prefix (validation only, no decoding)
    pub fn from_prefix(bytes: &[u8]) -> Result<(Self, usize), MulticodecError> {
        if bytes.len() < 2 {
            return Err(MulticodecError::InsufficientBytes);
        }
        
        let code = u16::from_be_bytes([bytes[0], bytes[1]]);
        match code {
            0x0701 => Ok((Self::Borsh, 2)),
            0x0702 => Ok((Self::Bincode, 2)),
            0x0201 => Ok((Self::MessagePack, 2)),
            0x0050 => Ok((Self::Protobuf, 2)),
            _ => Err(MulticodecError::Unknown(code)),
        }
    }
    
    pub fn name(&self) -> &'static str {
        match self {
            Self::Borsh => "borsh",
            Self::Bincode => "bincode",
            Self::MessagePack => "messagepack",
            Self::Protobuf => "protobuf",
        }
    }
}
```

### Compatibility Checker

```rust
// src/core/multicodec/compatibility.rs

pub struct CodecCompatibilityChecker {
    component_support: HashMap<ComponentId, ComponentCodecSupport>,
}

impl CodecCompatibilityChecker {
    /// Check if sender and receiver can communicate
    pub fn check_compatibility(
        &self,
        sender: &ComponentId,
        receiver: &ComponentId,
        message_codec: Multicodec,
    ) -> Result<(), CompatibilityError> {
        let receiver_support = self.component_support
            .get(receiver)
            .ok_or(CompatibilityError::ReceiverNotFound)?;
        
        if !receiver_support.supported_codecs.contains(&message_codec) {
            return Err(CompatibilityError::UnsupportedCodec {
                sender: sender.clone(),
                receiver: receiver.clone(),
                message_codec,
                receiver_supports: receiver_support.supported_codecs.clone(),
            });
        }
        
        Ok(())
    }
    
    /// Find common codec between two components
    pub fn find_common_codec(
        &self,
        component_a: &ComponentId,
        component_b: &ComponentId,
    ) -> Result<Multicodec, CompatibilityError> {
        let a_support = self.component_support.get(component_a)
            .ok_or(CompatibilityError::ComponentNotFound)?;
        let b_support = self.component_support.get(component_b)
            .ok_or(CompatibilityError::ComponentNotFound)?;
        
        // Find intersection
        let common: Vec<_> = a_support.supported_codecs
            .intersection(&b_support.supported_codecs)
            .copied()
            .collect();
        
        if common.is_empty() {
            return Err(CompatibilityError::NoCommonCodec {
                component_a: component_a.clone(),
                component_b: component_b.clone(),
                a_supports: a_support.supported_codecs.clone(),
                b_supports: b_support.supported_codecs.clone(),
            });
        }
        
        // Prefer borsh if available (deterministic, cross-language)
        if common.contains(&Multicodec::Borsh) {
            return Ok(Multicodec::Borsh);
        }
        
        // Otherwise, return first common codec
        Ok(common[0])
    }
}
```

### Error Handling

```rust
#[derive(Debug, Error)]
pub enum CompatibilityError {
    #[error(
        "Component '{receiver}' does not support codec {message_codec:?}.\n\
         Supported codecs: {receiver_supports:?}\n\
         Suggestion: Use borsh (0x0701) for cross-language compatibility"
    )]
    UnsupportedCodec {
        sender: ComponentId,
        receiver: ComponentId,
        message_codec: Multicodec,
        receiver_supports: HashSet<Multicodec>,
    },
    
    #[error(
        "No common codec between components.\n\
         Component A '{component_a}' supports: {a_supports:?}\n\
         Component B '{component_b}' supports: {b_supports:?}\n\
         Suggestion: Add borsh support to both components"
    )]
    NoCommonCodec {
        component_a: ComponentId,
        component_b: ComponentId,
        a_supports: HashSet<Multicodec>,
        b_supports: HashSet<Multicodec>,
    },
}
```

### Message Routing

```rust
// src/core/messaging/router.rs

pub struct MessageRouter {
    compatibility_checker: Arc<CodecCompatibilityChecker>,
}

impl MessageRouter {
    pub async fn route_message(
        &self,
        sender: ComponentId,
        receiver: ComponentId,
        message: Vec<u8>,
    ) -> Result<(), MessageError> {
        // 1. Detect codec from multicodec prefix (no decoding!)
        let (codec, _prefix_len) = Multicodec::from_prefix(&message)
            .map_err(MessageError::InvalidMulticodec)?;
        
        // 2. Validate compatibility
        self.compatibility_checker
            .check_compatibility(&sender, &receiver, codec)
            .map_err(MessageError::Compatibility)?;
        
        // 3. Route message as opaque bytes (no translation!)
        self.deliver_to_component(receiver, message).await
            .map_err(MessageError::Delivery)?;
        
        Ok(())
    }
}
```

### Developer Workflow

**Install-time validation:**
```bash
$ airssys-wasm install component-a
✅ Installed successfully
   Component: component-a v1.0.0
   Supported codecs: borsh, messagepack
   Preferred codec: borsh

$ airssys-wasm install component-b
✅ Installed successfully
   Component: component-b v1.0.0
   Supported codecs: borsh
   Preferred codec: borsh

$ airssys-wasm compatibility component-a component-b
✅ Compatible via: borsh
   Common codecs: [borsh]
```

**Runtime validation:**
```rust
// Component A sends borsh message to Component B
send_message(component_a, component_b, borsh_message);
// ✅ Success - both support borsh

// Component A sends MessagePack to Component B
send_message(component_a, component_b, msgpack_message);
// ❌ Error: Component B doesn't support MessagePack (0x0201).
//           Supported: borsh (0x0701).
//           Use borsh for compatibility.
```

---

## Consequences

### Positive

1. ✅ **Simple Host Runtime**
   - No codec translation logic (200 lines vs 2000+)
   - No codec library dependencies (lightweight)
   - Clear separation of concerns
   - Easy to maintain and test

2. ✅ **No Performance Overhead**
   - Messages routed as opaque bytes
   - No serialize → deserialize → re-serialize cycle
   - Direct pass-through: ~200ns per message
   - Optimal throughput

3. ✅ **No Data Loss Risk**
   - No ambiguous type conversions
   - No silent data corruption
   - Components control their data format
   - Type safety preserved

4. ✅ **Clear Error Messages**
   - Fail fast at compatibility check
   - Explicit codec mismatch errors
   - Actionable error messages with suggestions
   - Production-safe (no silent failures)

5. ✅ **Component Flexibility**
   - Components choose their codecs
   - Can add new codecs without host changes
   - Cross-language support enabled
   - No framework lock-in

6. ✅ **Extensibility**
   - New codecs: Just add multicodec ID
   - No host code changes needed
   - Components implement codec support
   - Registry-based extensibility

### Negative

1. ⚠️ **Components Must Implement Codec Support**
   - Components need to handle multiple codecs if interop needed
   - Developers must understand codec compatibility
   - More work than "host does everything"
   - **Mitigation**: Provide clear documentation and codec helpers

2. ⚠️ **Runtime Compatibility Errors**
   - Errors only at message send time (not compile-time)
   - Requires good error messages
   - **Mitigation**: Install-time compatibility checks, CLI tools

3. ⚠️ **Developer Education**
   - Need to teach codec selection
   - Need to explain multicodec
   - **Mitigation**: Good documentation, examples, CLI helpers

### Neutral

1. 📝 **Component Manifest Required**
   - Components must declare codec support
   - Additional configuration burden
   - But: Enables validation and tooling

2. 📝 **Codec Standardization Encouraged**
   - Encourages borsh for cross-language (good!)
   - Some components may over-standardize
   - But: Simplifies ecosystem

---

## Alternatives Considered

### Alternative 1: Host Translates Between Codecs (Rejected)

**Approach:**
```rust
pub async fn route_message(sender, receiver, message) {
    let sender_codec = detect_codec(message);
    let receiver_codec = get_preferred_codec(receiver);
    
    if sender_codec != receiver_codec {
        // Translate!
        let decoded = decode(message, sender_codec)?;
        let encoded = encode(decoded, receiver_codec)?;
        deliver(receiver, encoded).await
    }
}
```

**Why Rejected:**
- ❌ Requires type information (impossible without schema)
- ❌ Data loss risk (ambiguous conversions)
- ❌ Performance overhead (2-3x slower)
- ❌ Complex implementation (2000+ lines)
- ❌ Tight coupling to codec libraries
- ❌ Silent failures possible

**Decision:** Too risky, too complex, marginal benefit.

### Alternative 2: Mandatory Single Codec (Rejected)

**Approach:**
```rust
// Force all components to use borsh only
pub fn send_message(receiver, message) {
    assert!(is_borsh(message), "Only borsh allowed");
    deliver(receiver, message).await
}
```

**Why Rejected:**
- ❌ No flexibility for components
- ❌ Forces language-specific choice
- ❌ Can't optimize with bincode internally
- ❌ Breaks if borsh has issues
- ✅ Simple, but too restrictive

**Decision:** Too restrictive for general-purpose framework.

### Alternative 3: Compile-Time Codec Negotiation (Rejected)

**Approach:**
```rust
// Generate code at component install time
// to ensure codec compatibility
fn install(component_a, component_b) {
    let common = find_common_codec(a, b)?;
    codegen_wrapper(component_a, common);
    codegen_wrapper(component_b, common);
}
```

**Why Rejected:**
- ⚠️ Complex code generation
- ⚠️ Dynamic component loading issues
- ⚠️ Reduces runtime flexibility
- ✅ Would catch errors early
- But: Too complex for the benefit

**Decision:** Runtime validation is simpler and sufficient.

---

## Follow-up Actions

### Immediate (Phase 1 - v1.0)

1. ✅ Implement `Multicodec` enum with ID registry
2. ✅ Implement `CodecCompatibilityChecker`
3. ✅ Implement `MessageRouter` with validation
4. ✅ Add Component.toml serialization section
5. ✅ Implement compatibility CLI commands
6. ✅ Write developer documentation
7. ✅ Create example components demonstrating codec usage

### Short-term (Phase 1.5 - v1.5)

1. 📝 Monitor codec compatibility issues in production
2. 📝 Collect feedback from component developers
3. 📝 Consider adding MessagePack if web ecosystem needs it
4. 📝 Evaluate codec negotiation helpers if needed

### Long-term (Phase 2 - v2.0)

1. 📝 Evaluate schema-based codecs (Protocol Buffers) if schema evolution becomes pain point
2. 📝 Consider compile-time compatibility validation if runtime errors frequent
3. 📝 Assess need for codec translation if strongly requested by community
4. 📝 Monitor industry trends in serialization formats

---

## References

### Related Documentation

- **KNOWLEDGE-WASM-006**: Multiformat Strategy (multicodec specification)
- **KNOWLEDGE-WASM-011**: Serialization Strategy - bincode vs borsh
- **KNOWLEDGE-WASM-005**: Inter-Component Messaging Architecture

### Standards & Specifications

- **Protocol Labs Multiformats**: https://github.com/multiformats/multiformats
- **Multicodec Table**: https://github.com/multiformats/multicodec/blob/master/table.csv
- **borsh Specification**: https://borsh.io/
- **bincode Documentation**: https://github.com/bincode-org/bincode

### Industry Examples

- **HTTP Content Negotiation**: RFC 7231 (Accept/Content-Type headers)
- **gRPC**: Single codec approach (Protocol Buffers mandatory)
- **NEAR Protocol**: borsh-only smart contracts (production-proven)
- **Solana**: borsh serialization standard

---

## Decision Log

| Date | Decision | Participants |
|------|----------|--------------|
| 2025-10-19 | Initial decision: No host translation, validation only | Architecture Team |
| 2025-10-19 | Approved: Multicodec ID registry approach | Architecture Team |
| 2025-10-19 | Approved: Fail-fast compatibility checking | Architecture Team |

---

**Status:** ✅ **Accepted**  
**Implementation Priority:** Critical (Phase 1 Foundation)  
**Next Review:** After 6 months production use (Q2 2026)
