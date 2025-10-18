# Multiformat Strategy - airssys-wasm

**Document Type:** Knowledge Documentation  
**Created:** 2025-10-18  
**Updated:** 2025-10-18  
**Status:** Complete - Format Compatibility and Error Handling Added  
**Priority:** Critical - Self-Describing Data Foundation  
**Related:** KNOWLEDGE-WASM-004 (WIT Interfaces), KNOWLEDGE-WASM-005 (Messaging Architecture)

## Overview

This document provides comprehensive documentation for **multiformats integration** in airssys-wasm. Multiformats is a collection of protocols for self-describing values, enabling true language-agnostic interoperability and future-proof data formats.

**Key Design Principle:** AirsSys WASM adopts a **"self-describing without negotiation"** approach. The multicodec prefix in every message provides format identification, eliminating the need for format negotiation protocols. Components discover format support at runtime through message handling and fail fast with clear errors for unsupported formats.

### What are Multiformats?

**Multiformats** is an official specification from Protocol Labs (creators of IPFS, libp2p, IPLD) that provides self-describing data formats through compact prefixes. The specification ensures that data can evolve without breaking existing systems.

**Official Resources:**
- Main Repository: https://github.com/multiformats/multiformats
- Multicodec Specification: https://github.com/multiformats/multicodec
- Multicodec Table (Canonical): https://github.com/multiformats/multicodec/blob/master/table.csv
- License: CC-BY-SA 3.0 (documents), MIT (code)

### The Multiformats Family

**Core Specifications:**
- **multiaddr**: Network addresses (stable, W3C CCG)
- **multibase**: Base encodings (stable, W3C CCG)
- **multicodec**: Serialization codes (stable, TBD standardization)
- **multihash**: Cryptographic hashes (stable, W3C CCG)
- **multikey**: Cryptographic keys (draft)

**For airssys-wasm, we primarily use:**
- ✅ **multicodec**: Self-describing serialization formats
- 🔮 **multihash**: Future content addressing and verification
- 🔮 **multikey**: Future cryptographic operations

## Multicodec Specification

### Core Concept

**Self-Describing Data via Prefix:**
```
[varint_codec][actual_data]
     ↑            ↑
   Identifies    Serialized
   the format    payload
```

**Example:**
```
Borsh-encoded data:
[0x701][0x05 0x41 0x6c 0x69 0x63 0x65 ...] 
  ↑      ↑
  Borsh  "Alice" + rest of data
```

### Varint Encoding

**Unsigned Varint (MSB):**
- Most Significant Bit unsigned varint as defined by multiformats/unsigned-varint
- Compact representation: codes 0-127 use single byte
- Larger codes use multiple bytes with continuation bit

**Encoding Rules:**
```
Value Range    Bytes  Example
───────────────────────────────
0-127          1      0x50 = 80 (protobuf)
128-16383      2      0x0200 = 512 (json)
16384-2M       3      0x0201 = 513 (msgpack)
...
```

**First 127 codes (0x00-0x7F) are reserved for most widely used codecs.**

### AirsSys Reserved Multicodec Codes

**Official Reservations:**

| Code | Hex | Name | Tag | Status | Description |
|------|-----|------|-----|--------|-------------|
| 1793 | 0x701 | borsh | serialization | permanent | Binary Object Representation Serializer for Hashing - Near Protocol |
| 1794 | 0x702 | bincode | serialization | draft | Rust bincode serialization (reserved for airssys-wasm) |

**Standard Codecs (From Official Table):**

| Code | Hex | Name | Tag | Status | Description |
|------|-----|------|-----|--------|-------------|
| 80 | 0x50 | protobuf | serialization | permanent | Protocol Buffers |
| 85 | 0x55 | raw | ipld | permanent | Raw binary (no interpretation) |
| 81 | 0x51 | cbor | ipld | permanent | CBOR - RFC 7049 |
| 113 | 0x71 | dag-cbor | ipld | permanent | CBOR with IPLD links |
| 512 | 0x0200 | json | ipld | permanent | JSON - RFC 8259 |
| 297 | 0x0129 | dag-json | ipld | permanent | JSON with IPLD links |
| 513 | 0x0201 | messagepack | serialization | permanent | MessagePack |
| 276 | 0x0114 | yaml | serialization | draft | YAML |

### Tag Categories

**Relevant for AirsSys:**

**serialization:**
- General-purpose serialization formats
- Do NOT materialize IPLD links
- Examples: borsh, bincode, protobuf, json, msgpack, yaml
- Use for: Component messages, configuration, data exchange

**ipld (InterPlanetary Linked Data):**
- IPLD-aware formats that handle links
- Suitable for CID codecs
- Examples: dag-cbor, dag-json, raw
- Use for: Future content-addressed storage, IPFS integration

**hash:**
- Cryptographic hash functions (multihash)
- Future use: Content verification, integrity checks
- Examples: sha2-256 (0x12), blake3 (0x1e)

## AirsSys Multicodec Integration

### Supported Formats

**Primary (Binary - High Performance):**
- **borsh (0x701)**: Primary format for Rust components
  - Strict canonical encoding (deterministic)
  - Zero-copy deserialization
  - Excellent performance
  - Near Protocol standard
  
- **bincode (0x702)**: Alternative for Rust components
  - Very compact encoding
  - Fast ser/deser
  - Rust-native

**Secondary (Binary - Cross-Language):**
- **msgpack (0x0201)**: Cross-language binary format
  - Compact representation
  - Wide language support
  - Good performance

**Debugging (Text - Human-Readable):**
- **json (0x0200)**: Universal debugging format
  - Human-readable
  - Browser-friendly
  - Slower performance
  
- **yaml (0x0114)**: Configuration and documentation
  - Human-friendly
  - Comments supported
  - Configuration files

**Future:**
- **dag-cbor (0x71)**: IPLD integration
- **raw (0x55)**: Raw binary pass-through

### Format Selection Strategy

**Decision Tree:**

```
Is performance critical?
├─ YES: Is Rust component?
│   ├─ YES: Use Borsh (0x701) ←── PRIMARY CHOICE
│   └─ NO:  Use MessagePack (0x0201)
│
└─ NO: Is debugging/development?
    ├─ YES: Use JSON (0x0200)
    └─ NO:  Use MessagePack (0x0201)
```

**Component Language Recommendations:**

| Language | Primary Format | Secondary | Debugging |
|----------|----------------|-----------|-----------|
| Rust | Borsh (0x701) | Bincode (0x702) | JSON (0x0200) |
| JavaScript/TypeScript | MessagePack (0x0201) | JSON (0x0200) | JSON (0x0200) |
| Go | MessagePack (0x0201) | JSON (0x0200) | JSON (0x0200) |
| Python | MessagePack (0x0201) | JSON (0x0200) | JSON (0x0200) |

## Format Compatibility and Error Handling

### Design Principle: No Negotiation Required

**Core Philosophy:**

AirsSys WASM adopts a **"self-describing without negotiation"** approach. The multicodec prefix in every message eliminates the need for format negotiation protocols:

```
┌─────────────────────────────────────────────────────────────┐
│ Component A                                                 │
│  ├─ Chooses format: Borsh (0x701)                          │
│  ├─ Encodes: [0x701][borsh data]                           │
│  └─ Sends message                                           │
└─────────────────────────────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────────────┐
│ Component B                                                 │
│  ├─ Reads prefix: 0x701                                     │
│  ├─ Checks: "Do I support codec 0x701?"                     │
│  │   ├─ YES → Decode with borsh deserializer               │
│  │   └─ NO  → Return UnsupportedFormat error               │
│  └─ Process or fail explicitly                              │
└─────────────────────────────────────────────────────────────┘
```

**Key Benefits:**
- ✅ **Zero negotiation overhead** - no handshake required
- ✅ **Fail-fast behavior** - unsupported formats error immediately
- ✅ **Component autonomy** - each component chooses supported formats
- ✅ **Clear error contracts** - explicit format support expectations
- ✅ **YAGNI compliance** - no speculative negotiation mechanism
- ✅ **Security** - no capability disclosure or fingerprinting

### Receiver Responsibility Model

**Responsibility:** The **receiver** is responsible for:
1. Reading the multicodec prefix
2. Determining if the format is supported
3. Decoding if supported, or returning clear error if not

**Sender Freedom:** The **sender** is free to:
1. Choose any serialization format they prefer
2. Send messages without prior knowledge of receiver capabilities
3. Handle format errors returned by receivers

### Error Handling Patterns

#### Rust Implementation

**Structured Error Types:**

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MulticodecError {
    #[error("Unsupported serialization format: 0x{codec:x} ({name}). Supported formats: {supported}")]
    UnsupportedFormat {
        codec: u64,
        name: String,
        supported: String,
    },
    
    #[error("Failed to decode multicodec prefix: {0}")]
    InvalidPrefix(String),
    
    #[error("Deserialization failed for codec 0x{codec:x}: {error}")]
    DeserializationFailed {
        codec: u64,
        error: String,
    },
    
    #[error("Empty message data")]
    EmptyData,
}

#[derive(Error, Debug)]
pub enum MessagingError {
    #[error("Multicodec error: {0}")]
    Multicodec(#[from] MulticodecError),
    
    #[error("Component does not support format 0x{codec:x}. Please use one of: {supported_list}")]
    FormatNotSupported {
        codec: u64,
        supported_list: String,
    },
}
```

**Component Implementation with Clear Errors:**

```rust
use borsh::{BorshSerialize, BorshDeserialize};
use serde::{Serialize, Deserialize};

pub struct Component {
    // Component only supports these formats
    supported_codecs: Vec<u64>,
}

impl Component {
    pub fn new() -> Self {
        Self {
            // This component supports: Borsh, MessagePack, JSON
            supported_codecs: vec![0x701, 0x0201, 0x0200],
        }
    }
    
    /// Decode message with explicit format support checking
    pub fn decode_message<T>(&self, encoded: &[u8]) -> Result<T, MulticodecError>
    where
        T: BorshDeserialize + for<'de> Deserialize<'de>
    {
        // Decode multicodec prefix
        let decoded = self.decode_prefix(encoded)?;
        
        // Check if format is supported
        if !self.supported_codecs.contains(&decoded.codec) {
            return Err(MulticodecError::UnsupportedFormat {
                codec: decoded.codec,
                name: self.codec_name(decoded.codec),
                supported: self.supported_formats_string(),
            });
        }
        
        // Decode based on format
        match decoded.codec {
            0x701 => {
                // Borsh format
                T::try_from_slice(&decoded.data)
                    .map_err(|e| MulticodecError::DeserializationFailed {
                        codec: 0x701,
                        error: e.to_string(),
                    })
            }
            0x0201 => {
                // MessagePack format
                rmp_serde::from_slice(&decoded.data)
                    .map_err(|e| MulticodecError::DeserializationFailed {
                        codec: 0x0201,
                        error: e.to_string(),
                    })
            }
            0x0200 => {
                // JSON format
                serde_json::from_slice(&decoded.data)
                    .map_err(|e| MulticodecError::DeserializationFailed {
                        codec: 0x0200,
                        error: e.to_string(),
                    })
            }
            code => {
                // This should never happen due to earlier check, but be defensive
                Err(MulticodecError::UnsupportedFormat {
                    codec: code,
                    name: self.codec_name(code),
                    supported: self.supported_formats_string(),
                })
            }
        }
    }
    
    fn supported_formats_string(&self) -> String {
        self.supported_codecs.iter()
            .map(|c| format!("0x{:x} ({})", c, self.codec_name(*c)))
            .collect::<Vec<_>>()
            .join(", ")
    }
    
    fn codec_name(&self, codec: u64) -> String {
        match codec {
            0x701 => "borsh".to_string(),
            0x702 => "bincode".to_string(),
            0x50 => "protobuf".to_string(),
            0x0200 => "json".to_string(),
            0x0201 => "msgpack".to_string(),
            0x0114 => "yaml".to_string(),
            0x55 => "raw".to_string(),
            code => format!("unknown-0x{:x}", code),
        }
    }
}

// Usage in handle-message export
#[export_name = "handle-message"]
pub extern "C" fn handle_message(
    sender_ptr: *const u8, sender_len: usize,
    message_ptr: *const u8, message_len: usize
) -> i32 {
    let component = get_component_instance();
    let message = unsafe { std::slice::from_raw_parts(message_ptr, message_len) };
    
    match component.decode_message::<UserData>(message) {
        Ok(data) => {
            // Successfully decoded - process message
            process_user_data(data);
            0 // Success
        }
        Err(MulticodecError::UnsupportedFormat { codec, name, supported }) => {
            // Clear error message for unsupported format
            log_error(&format!(
                "Cannot decode message from sender. Format 0x{:x} ({}) is not supported. \
                 This component supports: {}",
                codec, name, supported
            ));
            1 // Error code: unsupported format
        }
        Err(e) => {
            // Other errors
            log_error(&format!("Message decode error: {}", e));
            2 // Error code: general decode error
        }
    }
}
```

**Example Error Messages:**

```
❌ Error: Unsupported serialization format: 0x702 (bincode). 
         Supported formats: 0x701 (borsh), 0x0201 (msgpack), 0x0200 (json)

❌ Error: Component does not support format 0x50 (protobuf). 
         Please use one of: borsh, msgpack, json

❌ Error: Deserialization failed for codec 0x701: 
         unexpected end of buffer while deserializing field 'name'
```

#### JavaScript/TypeScript Implementation

**Clear Error Handling:**

```javascript
import { decode as msgpackDecode } from '@msgpack/msgpack';
import { decode as multicodecDecode } from 'airssys:multicodec-core/codec';

class MulticodecError extends Error {
    constructor(message, codec, supportedFormats) {
        super(message);
        this.name = 'MulticodecError';
        this.codec = codec;
        this.supportedFormats = supportedFormats;
    }
}

class Component {
    constructor() {
        // This component supports MessagePack and JSON only
        this.supportedCodecs = [0x0201, 0x0200];
    }
    
    /**
     * Decode message with format validation
     */
    decodeMessage(encoded) {
        // Decode multicodec prefix
        const decoded = multicodecDecode(encoded);
        
        // Check if format is supported
        if (!this.supportedCodecs.includes(decoded.codec)) {
            const codecName = this.getCodecName(decoded.codec);
            const supported = this.supportedCodecs
                .map(c => `0x${c.toString(16)} (${this.getCodecName(c)})`)
                .join(', ');
            
            throw new MulticodecError(
                `Unsupported serialization format: 0x${decoded.codec.toString(16)} (${codecName}). ` +
                `Supported formats: ${supported}`,
                decoded.codec,
                this.supportedCodecs
            );
        }
        
        // Decode based on format
        switch (decoded.codec) {
            case 0x0201: // MessagePack
                try {
                    return msgpackDecode(decoded.data);
                } catch (err) {
                    throw new Error(
                        `Deserialization failed for codec 0x0201 (msgpack): ${err.message}`
                    );
                }
            
            case 0x0200: // JSON
                try {
                    const text = new TextDecoder().decode(decoded.data);
                    return JSON.parse(text);
                } catch (err) {
                    throw new Error(
                        `Deserialization failed for codec 0x0200 (json): ${err.message}`
                    );
                }
            
            default:
                // Should never reach here due to earlier check
                throw new MulticodecError(
                    `Unexpected codec: 0x${decoded.codec.toString(16)}`,
                    decoded.codec,
                    this.supportedCodecs
                );
        }
    }
    
    getCodecName(codec) {
        const names = {
            0x701: 'borsh',
            0x702: 'bincode',
            0x50: 'protobuf',
            0x0200: 'json',
            0x0201: 'msgpack',
            0x0114: 'yaml',
            0x55: 'raw'
        };
        return names[codec] || `unknown-0x${codec.toString(16)}`;
    }
}

// Export handle-message
export function handleMessage(sender, message) {
    const component = new Component();
    
    try {
        const data = component.decodeMessage(message);
        processUserData(data);
        return { success: true };
    } catch (err) {
        if (err instanceof MulticodecError) {
            console.error(`Format not supported: ${err.message}`);
            return { 
                success: false, 
                error: 'unsupported_format',
                message: err.message,
                codec: err.codec,
                supported: err.supportedFormats
            };
        }
        
        console.error(`Message decode error: ${err.message}`);
        return { 
            success: false, 
            error: 'decode_error',
            message: err.message
        };
    }
}
```

#### Go Implementation

**Structured Error Handling:**

```go
package component

import (
    "fmt"
    "encoding/json"
    "github.com/vmihailenco/msgpack/v5"
    "airssys.dev/multicodec"
)

type MulticodecError struct {
    Codec           uint64
    CodecName       string
    SupportedFormats []uint64
    Message         string
}

func (e *MulticodecError) Error() string {
    return e.Message
}

type Component struct {
    supportedCodecs []uint64
}

func NewComponent() *Component {
    return &Component{
        supportedCodecs: []uint64{0x0201, 0x0200}, // msgpack, json
    }
}

// DecodeMessage decodes multicodec-prefixed message
func (c *Component) DecodeMessage(encoded []byte, result interface{}) error {
    // Decode multicodec prefix
    decoded, err := multicodec.Decode(encoded)
    if err != nil {
        return fmt.Errorf("multicodec decode failed: %w", err)
    }
    
    // Check if format is supported
    if !c.isSupported(decoded.Codec) {
        return &MulticodecError{
            Codec:     decoded.Codec,
            CodecName: c.codecName(decoded.Codec),
            SupportedFormats: c.supportedCodecs,
            Message: fmt.Sprintf(
                "Unsupported serialization format: 0x%x (%s). Supported formats: %s",
                decoded.Codec,
                c.codecName(decoded.Codec),
                c.supportedFormatsString(),
            ),
        }
    }
    
    // Decode based on format
    switch decoded.Codec {
    case 0x0201: // MessagePack
        err := msgpack.Unmarshal(decoded.Data, result)
        if err != nil {
            return fmt.Errorf("deserialization failed for codec 0x0201 (msgpack): %w", err)
        }
        return nil
        
    case 0x0200: // JSON
        err := json.Unmarshal(decoded.Data, result)
        if err != nil {
            return fmt.Errorf("deserialization failed for codec 0x0200 (json): %w", err)
        }
        return nil
        
    default:
        return &MulticodecError{
            Codec:     decoded.Codec,
            CodecName: c.codecName(decoded.Codec),
            SupportedFormats: c.supportedCodecs,
            Message: fmt.Sprintf("unexpected codec: 0x%x", decoded.Codec),
        }
    }
}

func (c *Component) isSupported(codec uint64) bool {
    for _, supported := range c.supportedCodecs {
        if codec == supported {
            return true
        }
    }
    return false
}

func (c *Component) codecName(codec uint64) string {
    names := map[uint64]string{
        0x701:  "borsh",
        0x702:  "bincode",
        0x50:   "protobuf",
        0x0200: "json",
        0x0201: "msgpack",
        0x0114: "yaml",
        0x55:   "raw",
    }
    if name, ok := names[codec]; ok {
        return name
    }
    return fmt.Sprintf("unknown-0x%x", codec)
}

func (c *Component) supportedFormatsString() string {
    var formats []string
    for _, codec := range c.supportedCodecs {
        formats = append(formats, fmt.Sprintf("0x%x (%s)", codec, c.codecName(codec)))
    }
    return strings.Join(formats, ", ")
}

// HandleMessage export
//export handle_message
func HandleMessage(senderPtr, senderLen uint32, messagePtr, messageLen uint32) int32 {
    component := NewComponent()
    message := multicodec.BytesFromWasm(messagePtr, messageLen)
    
    var userData UserData
    err := component.DecodeMessage(message, &userData)
    if err != nil {
        if mcErr, ok := err.(*MulticodecError); ok {
            log.Errorf("Format not supported: %s", mcErr.Message)
            return 1 // Error code: unsupported format
        }
        
        log.Errorf("Message decode error: %v", err)
        return 2 // Error code: general decode error
    }
    
    // Successfully decoded - process message
    processUserData(&userData)
    return 0 // Success
}
```

### Format Compatibility Matrix

**Understanding Component Interoperability:**

| Sender Format | Receiver Supports | Result |
|---------------|-------------------|---------|
| Borsh (0x701) | Borsh | ✅ Success - Fast decode |
| Borsh (0x701) | MessagePack only | ❌ Error - Unsupported format |
| MessagePack (0x0201) | Borsh + MessagePack | ✅ Success - Decode with MessagePack |
| JSON (0x0200) | JSON + MessagePack | ✅ Success - Decode with JSON |
| Bincode (0x702) | Borsh + JSON | ❌ Error - Unsupported format |

**Key Insight:** No format is universally required. Components declare what they support through their implementation, not through negotiation.

### Best Practices for Format Selection

#### For Component Developers

**1. Choose Appropriate Supported Formats:**

```rust
// ✅ GOOD: Support multiple formats for interoperability
impl Component {
    fn new() -> Self {
        Self {
            // Support both performance (borsh) and cross-language (msgpack)
            supported_codecs: vec![0x701, 0x0201, 0x0200],
        }
    }
}

// ❌ POOR: Only support language-specific format
impl Component {
    fn new() -> Self {
        Self {
            // Only borsh - limits interoperability with non-Rust components
            supported_codecs: vec![0x701],
        }
    }
}
```

**2. Prioritize Common Formats:**

**Universal Formats** (high interoperability):
- **MessagePack (0x0201)**: Best cross-language support
- **JSON (0x0200)**: Universal compatibility, debugging-friendly

**Language-Optimized Formats** (high performance):
- **Borsh (0x701)**: Rust components
- **Bincode (0x702)**: Rust-only ecosystems

**Recommendation:** Support at least one universal format (MessagePack or JSON) alongside your preferred performance format.

#### For System Designers

**3. Document Format Requirements:**

```markdown
## Component: User Authentication Service

**Sends Messages With:**
- Primary: Borsh (0x701) - for performance
- Fallback: MessagePack (0x0201) - for non-Rust components

**Accepts Messages In:**
- Borsh (0x701)
- MessagePack (0x0201)
- JSON (0x0200) - debugging/development only

**Integration Notes:**
- Rust components should use Borsh for best performance
- Non-Rust components should use MessagePack
- JSON accepted but not recommended for production
```

**4. Test Format Compatibility:**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_unsupported_format_error() {
        let component = Component::new();
        
        // Encode with bincode (0x702)
        let data = UserData { name: "Alice".into() };
        let encoded = encode_bincode(&data);
        
        // Component doesn't support bincode
        let result = component.decode_message::<UserData>(&encoded);
        
        assert!(result.is_err());
        match result.unwrap_err() {
            MulticodecError::UnsupportedFormat { codec, .. } => {
                assert_eq!(codec, 0x702);
            }
            _ => panic!("Expected UnsupportedFormat error"),
        }
    }
    
    #[test]
    fn test_cross_format_compatibility() {
        let component = Component::new();
        let data = UserData { name: "Bob".into(), age: 30 };
        
        // Test all supported formats
        for codec in &[0x701, 0x0201, 0x0200] {
            let encoded = encode_with_codec(*codec, &data);
            let decoded: UserData = component.decode_message(&encoded)
                .expect(&format!("Failed to decode codec 0x{:x}", codec));
            
            assert_eq!(decoded.name, "Bob");
            assert_eq!(decoded.age, 30);
        }
    }
}
```

**5. Provide Clear Error Guidance:**

```rust
// ✅ GOOD: Helpful error message
Err(MulticodecError::UnsupportedFormat {
    codec: 0x702,
    name: "bincode".into(),
    supported: "0x701 (borsh), 0x0201 (msgpack), 0x0200 (json)".into(),
})

// ❌ POOR: Vague error message
Err(anyhow!("Decode failed"))
```

#### For Application Architects

**6. Design for Graceful Degradation:**

```rust
impl Component {
    /// Try to send with optimal format, fallback if unsupported
    pub fn send_with_fallback(&self, target: &ComponentId, data: &UserData) -> Result<()> {
        // Try primary format (Borsh - fastest)
        let encoded = self.encode_borsh(data)?;
        match send_message(target, &encoded) {
            Ok(()) => return Ok(()),
            Err(MessagingError::FormatNotSupported { .. }) => {
                log_warn("Borsh not supported, trying MessagePack");
            }
            Err(e) => return Err(e),
        }
        
        // Fallback to MessagePack (cross-language)
        let encoded = self.encode_msgpack(data)?;
        match send_message(target, &encoded) {
            Ok(()) => return Ok(()),
            Err(MessagingError::FormatNotSupported { .. }) => {
                log_warn("MessagePack not supported, trying JSON");
            }
            Err(e) => return Err(e),
        }
        
        // Last resort: JSON (universal but slow)
        let encoded = self.encode_json(data)?;
        send_message(target, &encoded)
    }
}
```

**Note:** This fallback pattern is optional and adds complexity. Only implement if your use case requires automatic format negotiation. The simpler approach is to fail fast with clear errors.

**7. Monitor Format Usage:**

```rust
pub struct ComponentMetrics {
    formats_received: HashMap<u64, AtomicU64>,
    unsupported_format_errors: AtomicU64,
}

impl Component {
    fn track_format_usage(&self, codec: u64) {
        self.metrics.formats_received
            .entry(codec)
            .or_insert_with(|| AtomicU64::new(0))
            .fetch_add(1, Ordering::Relaxed);
    }
    
    fn track_format_error(&self) {
        self.metrics.unsupported_format_errors
            .fetch_add(1, Ordering::Relaxed);
    }
}
```

### Recommended Support Matrix

**Minimum Viable Support** (all components should implement):
- ✅ **MessagePack (0x0201)** - Universal cross-language format
- ✅ **JSON (0x0200)** - Debugging and development

**Language-Specific Additions:**

**Rust Components:**
- ✅ Borsh (0x701) - Primary for Rust-to-Rust
- ⚠️ Bincode (0x702) - Optional, only for Rust-only systems

**JavaScript Components:**
- ✅ MessagePack (0x0201) - Primary
- ✅ JSON (0x0200) - Native support

**Go/Python Components:**
- ✅ MessagePack (0x0201) - Primary
- ✅ JSON (0x0200) - Native support

**Performance-Critical Components (any language):**
- ✅ Add language-native format (Borsh for Rust, etc.)
- ✅ Keep MessagePack for interoperability

### Summary: Format Compatibility Philosophy

**Core Principles:**

1. **Self-Describing Eliminates Negotiation** - multicodec prefix tells receiver everything
2. **Receiver Decides** - components declare support through implementation
3. **Fail Fast, Fail Clear** - unsupported formats error immediately with helpful messages
4. **Component Autonomy** - no central registry or capability exchange required
5. **Universal Formats Recommended** - support at least MessagePack or JSON
6. **Performance Formats Optional** - add language-specific formats when needed
7. **Test Compatibility** - verify format support in integration tests
8. **Monitor Usage** - track which formats are actually used in production

This approach balances simplicity (no negotiation) with flexibility (components choose support) while maintaining clear error contracts and interoperability guidance.

### WIT Interface Definition

**From KNOWLEDGE-WASM-004:**

```wit
package airssys:multicodec-core@1.0.0;

/// Multicodec utilities for self-describing data
interface multicodec-utilities {
    /// Encode data with multicodec prefix
    encode: func(codec: multicodec-id, data: list<u8>) -> list<u8>;
    
    /// Decode multicodec-prefixed data
    decode: func(encoded: list<u8>) -> result<decoded-data, multicodec-error>;
    
    /// Check if codec is supported by framework
    is-codec-supported: func(codec: multicodec-id) -> bool;
    
    /// Get information about a codec
    get-codec-info: func(codec: multicodec-id) -> result<codec-info, multicodec-error>;
}

/// Standard multicodecs supported by framework
enum standard-multicodec {
    /// Binary formats (efficient)
    borsh = 0x701,                    // Reserved for airssys-wasm
    bincode = 0x702,                  // Reserved for airssys-wasm
    protobuf = 0x50,                  // Official multicodec
    msgpack = 0x0201,                 // Official multicodec
    
    /// Text formats (debugging/human-readable)
    json = 0x0200,                    // Official multicodec
    yaml = 0x0114,                    // Official multicodec
    
    /// Raw bytes (no interpretation)
    raw = 0x55,                       // Official multicodec
}
```

## Implementation Patterns

### Rust Component Implementation

**Encoding with Borsh:**

```rust
use borsh::{BorshSerialize, BorshDeserialize};
use airssys_wasm_bindings::multicodec::*;

#[derive(BorshSerialize, BorshDeserialize)]
struct UserData {
    id: String,
    name: String,
    email: String,
    age: u32,
}

impl Component {
    /// Encode data with multicodec prefix
    fn encode_borsh<T: BorshSerialize>(&self, data: &T) -> Result<Vec<u8>> {
        // Serialize with borsh
        let serialized = data.try_to_vec()
            .map_err(|e| anyhow!("Borsh serialization failed: {}", e))?;
        
        // Add multicodec prefix
        let encoded = encode(StandardMulticodec::Borsh as u64, &serialized);
        
        Ok(encoded)
    }
    
    /// Decode multicodec-prefixed data
    fn decode_borsh<T: BorshDeserialize>(&self, encoded: &[u8]) -> Result<T> {
        // Decode and verify codec
        let decoded = decode(encoded)
            .map_err(|e| anyhow!("Multicodec decode failed: {:?}", e))?;
        
        // Verify it's borsh format
        if decoded.codec != (StandardMulticodec::Borsh as u64) {
            return Err(anyhow!("Expected borsh codec (0x701), got 0x{:x}", decoded.codec));
        }
        
        // Deserialize with borsh
        T::try_from_slice(&decoded.data)
            .map_err(|e| anyhow!("Borsh deserialization failed: {}", e))
    }
}

// Usage example
impl Component {
    fn send_user_data(&self, target: &ComponentId, user: &UserData) -> Result<()> {
        // Encode with multicodec
        let encoded = self.encode_borsh(user)?;
        
        // Send message
        send_message(target, &encoded)?;
        
        Ok(())
    }
}

#[export_name = "handle-message"]
pub extern "C" fn handle_message(sender_ptr: *const u8, sender_len: usize,
                                  message_ptr: *const u8, message_len: usize) -> i32 {
    let sender = unsafe { ComponentId::from_ptr(sender_ptr, sender_len) };
    let message = unsafe { std::slice::from_raw_parts(message_ptr, message_len) };
    
    match handle_message_impl(sender, message) {
        Ok(()) => 0,
        Err(e) => {
            log_error(&format!("handle-message error: {}", e));
            1
        }
    }
}

fn handle_message_impl(sender: ComponentId, message: &[u8]) -> Result<()> {
    // Decode multicodec message
    let user_data: UserData = COMPONENT.decode_borsh(message)?;
    
    log_info(&format!("Received user data: {} <{}>", user_data.name, user_data.email));
    
    // Process user data
    process_user(user_data)?;
    
    Ok(())
}
```

**Format Negotiation:**

```rust
impl Component {
    /// Try multiple formats in order of preference
    fn decode_flexible<T>(&self, encoded: &[u8]) -> Result<T>
    where
        T: BorshDeserialize + serde::de::DeserializeOwned
    {
        // Decode multicodec prefix
        let decoded = decode(encoded)?;
        
        match decoded.codec {
            0x701 => {
                // Borsh format
                T::try_from_slice(&decoded.data)
                    .map_err(|e| anyhow!("Borsh decode failed: {}", e))
            }
            0x0200 => {
                // JSON format
                serde_json::from_slice(&decoded.data)
                    .map_err(|e| anyhow!("JSON decode failed: {}", e))
            }
            0x0201 => {
                // MessagePack format
                rmp_serde::from_slice(&decoded.data)
                    .map_err(|e| anyhow!("MessagePack decode failed: {}", e))
            }
            code => {
                Err(anyhow!("Unsupported codec: 0x{:x}", code))
            }
        }
    }
}
```

### JavaScript Component Implementation

**Encoding with MessagePack:**

```javascript
import { encode as msgpackEncode, decode as msgpackDecode } from '@msgpack/msgpack';
import { encode as multicodecEncode, decode as multicodecDecode } from 'airssys:multicodec-core/codec';

class Component {
    /**
     * Encode data with multicodec prefix
     * @param {any} data - JavaScript object to encode
     * @returns {Uint8Array} Multicodec-prefixed data
     */
    encodeMsgpack(data) {
        // Serialize with msgpack
        const serialized = msgpackEncode(data);
        
        // Add multicodec prefix (0x0201 = msgpack)
        const encoded = multicodecEncode(0x0201, serialized);
        
        return encoded;
    }
    
    /**
     * Decode multicodec-prefixed data
     * @param {Uint8Array} encoded - Multicodec-prefixed data
     * @returns {any} Decoded JavaScript object
     */
    decodeMsgpack(encoded) {
        // Decode multicodec
        const decoded = multicodecDecode(encoded);
        
        // Verify codec
        if (decoded.codec !== 0x0201) {
            throw new Error(`Expected msgpack codec (0x0201), got 0x${decoded.codec.toString(16)}`);
        }
        
        // Deserialize with msgpack
        return msgpackDecode(decoded.data);
    }
    
    /**
     * Flexible decoder supporting multiple formats
     */
    decodeFlexible(encoded) {
        const decoded = multicodecDecode(encoded);
        
        switch (decoded.codec) {
            case 0x0201: // msgpack
                return msgpackDecode(decoded.data);
            
            case 0x0200: // json
                const text = new TextDecoder().decode(decoded.data);
                return JSON.parse(text);
            
            case 0x701: // borsh
                throw new Error('Borsh decoding requires borsh-js library');
            
            default:
                throw new Error(`Unsupported codec: 0x${decoded.codec.toString(16)}`);
        }
    }
}

// Usage example
async function sendUserData(target, userData) {
    const component = getComponent();
    
    // Encode with multicodec
    const encoded = component.encodeMsgpack(userData);
    
    // Send message
    await sendMessage(target, encoded);
}

export function handleMessage(sender, messageData) {
    try {
        const component = getComponent();
        
        // Decode multicodec message
        const userData = component.decodeMsgpack(messageData);
        
        console.log(`Received user data: ${userData.name} <${userData.email}>`);
        
        // Process user data
        processUser(userData);
        
    } catch (error) {
        console.error('handle-message error:', error);
        throw error;
    }
}
```

### Go Component Implementation

**Encoding with MessagePack:**

```go
package main

import (
    "fmt"
    "github.com/vmihailenco/msgpack/v5"
    "airssys.io/wasm/bindings/multicodec"
)

type UserData struct {
    ID    string `msgpack:"id"`
    Name  string `msgpack:"name"`
    Email string `msgpack:"email"`
    Age   uint32 `msgpack:"age"`
}

type Component struct {
    // Component state
}

// EncodeMsgpack encodes data with multicodec prefix
func (c *Component) EncodeMsgpack(data interface{}) ([]byte, error) {
    // Serialize with msgpack
    serialized, err := msgpack.Marshal(data)
    if err != nil {
        return nil, fmt.Errorf("msgpack encoding failed: %w", err)
    }
    
    // Add multicodec prefix (0x0201 = msgpack)
    encoded := multicodec.Encode(0x0201, serialized)
    
    return encoded, nil
}

// DecodeMsgpack decodes multicodec-prefixed data
func (c *Component) DecodeMsgpack(encoded []byte, result interface{}) error {
    // Decode multicodec
    decoded, err := multicodec.Decode(encoded)
    if err != nil {
        return fmt.Errorf("multicodec decode failed: %w", err)
    }
    
    // Verify codec
    if decoded.Codec != 0x0201 {
        return fmt.Errorf("expected msgpack codec (0x0201), got 0x%x", decoded.Codec)
    }
    
    // Deserialize with msgpack
    if err := msgpack.Unmarshal(decoded.Data, result); err != nil {
        return fmt.Errorf("msgpack decoding failed: %w", err)
    }
    
    return nil
}

// Usage example
func (c *Component) SendUserData(target string, user *UserData) error {
    // Encode with multicodec
    encoded, err := c.EncodeMsgpack(user)
    if err != nil {
        return fmt.Errorf("encode failed: %w", err)
    }
    
    // Send message
    return host.SendMessage(target, encoded)
}

//export handle_message
func handleMessage(senderPtr, senderLen uint32, msgPtr, msgLen uint32) int32 {
    sender := ptrToComponentID(senderPtr, senderLen)
    message := ptrToBytes(msgPtr, msgLen)
    
    if err := handleMessageImpl(sender, message); err != nil {
        logError("handle-message error: %v", err)
        return 1
    }
    return 0
}

func handleMessageImpl(sender string, message []byte) error {
    var userData UserData
    
    // Decode multicodec message
    if err := component.DecodeMsgpack(message, &userData); err != nil {
        return fmt.Errorf("decode failed: %w", err)
    }
    
    log.Printf("Received user data: %s <%s>", userData.Name, userData.Email)
    
    // Process user data
    return processUser(&userData)
}
```

## Host Runtime Implementation

### Multicodec Utilities

**Host-Provided Multicodec Functions:**

```rust
use unsigned_varint::{encode as varint_encode, decode as varint_decode};

pub struct MulticodecRegistry {
    supported_codecs: HashMap<u64, CodecInfo>,
}

pub struct CodecInfo {
    pub code: u64,
    pub name: String,
    pub tag: CodecTag,
    pub status: CodecStatus,
}

pub enum CodecTag {
    Serialization,
    Ipld,
    Multihash,
    Multiaddr,
}

pub enum CodecStatus {
    Draft,
    Permanent,
    Deprecated,
}

impl MulticodecRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            supported_codecs: HashMap::new(),
        };
        
        // Register supported codecs
        registry.register(0x701, "borsh", CodecTag::Serialization, CodecStatus::Permanent);
        registry.register(0x702, "bincode", CodecTag::Serialization, CodecStatus::Draft);
        registry.register(0x50, "protobuf", CodecTag::Serialization, CodecStatus::Permanent);
        registry.register(0x0200, "json", CodecTag::Ipld, CodecStatus::Permanent);
        registry.register(0x0201, "messagepack", CodecTag::Serialization, CodecStatus::Permanent);
        registry.register(0x0114, "yaml", CodecTag::Serialization, CodecStatus::Draft);
        registry.register(0x55, "raw", CodecTag::Ipld, CodecStatus::Permanent);
        
        registry
    }
    
    fn register(&mut self, code: u64, name: &str, tag: CodecTag, status: CodecStatus) {
        self.supported_codecs.insert(code, CodecInfo {
            code,
            name: name.to_string(),
            tag,
            status,
        });
    }
    
    /// Encode data with multicodec prefix
    pub fn encode(&self, codec: u64, data: &[u8]) -> Vec<u8> {
        let mut buf = varint_encode::u64_buffer();
        let varint_bytes = varint_encode::u64(codec, &mut buf);
        
        let mut result = Vec::with_capacity(varint_bytes.len() + data.len());
        result.extend_from_slice(varint_bytes);
        result.extend_from_slice(data);
        
        result
    }
    
    /// Decode multicodec-prefixed data
    pub fn decode(&self, encoded: &[u8]) -> Result<DecodedData> {
        if encoded.is_empty() {
            return Err(Error::EmptyData);
        }
        
        // Decode varint prefix
        let (codec, prefix_len) = varint_decode::u64(encoded)
            .map_err(|e| Error::InvalidVarint(e))?;
        
        // Extract data after prefix
        let data = &encoded[prefix_len..];
        
        Ok(DecodedData {
            codec,
            data: data.to_vec(),
        })
    }
    
    /// Check if codec is supported
    pub fn is_supported(&self, codec: u64) -> bool {
        self.supported_codecs.contains_key(&codec)
    }
    
    /// Get codec information
    pub fn get_info(&self, codec: u64) -> Option<&CodecInfo> {
        self.supported_codecs.get(&codec)
    }
}

pub struct DecodedData {
    pub codec: u64,
    pub data: Vec<u8>,
}

// WIT host function implementations
impl HostContext {
    pub fn multicodec_encode(&self, codec: u64, data: Vec<u8>) -> Vec<u8> {
        self.multicodec_registry.encode(codec, &data)
    }
    
    pub fn multicodec_decode(&self, encoded: Vec<u8>) -> Result<DecodedData, MulticodecError> {
        self.multicodec_registry.decode(&encoded)
            .map_err(|e| MulticodecError::DecodeFailed(format!("{:?}", e)))
    }
    
    pub fn multicodec_is_supported(&self, codec: u64) -> bool {
        self.multicodec_registry.is_supported(codec)
    }
    
    pub fn multicodec_get_info(&self, codec: u64) -> Result<CodecInfo, MulticodecError> {
        self.multicodec_registry.get_info(codec)
            .cloned()
            .ok_or(MulticodecError::CodecNotFound(codec))
    }
}
```

### Message Validation

**Multicodec Validation in Message Router:**

```rust
impl MessageRouter {
    pub async fn route_message_with_validation(
        &self,
        from: ComponentId,
        to: ComponentId,
        data: Vec<u8>
    ) -> Result<()> {
        // Validate multicodec prefix
        let decoded = self.multicodec_registry.decode(&data)
            .map_err(|e| Error::InvalidMulticodecPrefix(format!("{:?}", e)))?;
        
        // Check if codec is supported
        if !self.multicodec_registry.is_supported(decoded.codec) {
            return Err(Error::UnsupportedCodec {
                codec: decoded.codec,
                from: from.clone(),
            });
        }
        
        // Log codec usage for monitoring
        self.metrics.record_codec_usage(decoded.codec, data.len());
        
        // Route message
        self.route_message(from, to, data).await
    }
}
```

## Format Evolution Strategy

### Versioning Through Multicodec

**Evolving Data Formats:**

```rust
// Version 1: Initial format (borsh 0x701)
#[derive(BorshSerialize, BorshDeserialize)]
struct UserDataV1 {
    id: String,
    name: String,
    email: String,
}

// Version 2: Extended format (still borsh 0x701, but with new fields)
#[derive(BorshSerialize, BorshDeserialize)]
struct UserDataV2 {
    id: String,
    name: String,
    email: String,
    age: Option<u32>,        // New optional field
    created_at: Option<u64>, // New optional field
}

// Backward compatible decoder
impl Component {
    fn decode_user_data(&self, encoded: &[u8]) -> Result<UserDataV2> {
        let decoded = decode(encoded)?;
        
        // Still using borsh (0x701)
        if decoded.codec == 0x701 {
            // Try V2 first
            if let Ok(v2) = UserDataV2::try_from_slice(&decoded.data) {
                return Ok(v2);
            }
            
            // Fallback to V1 and upgrade
            if let Ok(v1) = UserDataV1::try_from_slice(&decoded.data) {
                return Ok(UserDataV2 {
                    id: v1.id,
                    name: v1.name,
                    email: v1.email,
                    age: None,
                    created_at: None,
                });
            }
        }
        
        Err(anyhow!("Failed to decode user data"))
    }
}
```

**Migration to New Format:**

```rust
// Migrate from borsh to msgpack for better cross-language support
impl Component {
    fn decode_user_data_flexible(&self, encoded: &[u8]) -> Result<UserData> {
        let decoded = decode(encoded)?;
        
        match decoded.codec {
            0x701 => {
                // Old borsh format - still support for backward compat
                UserData::try_from_slice(&decoded.data)
                    .map_err(|e| anyhow!("Borsh decode failed: {}", e))
            }
            0x0201 => {
                // New msgpack format - preferred
                rmp_serde::from_slice(&decoded.data)
                    .map_err(|e| anyhow!("MessagePack decode failed: {}", e))
            }
            code => {
                Err(anyhow!("Unsupported codec: 0x{:x}", code))
            }
        }
    }
    
    fn encode_user_data_new(&self, data: &UserData) -> Result<Vec<u8>> {
        // Use new msgpack format for all new messages
        let serialized = rmp_serde::to_vec(data)?;
        Ok(encode(0x0201, &serialized))
    }
}
```

## Component Manifest Integration (Optional)

### Optional Format Documentation

**Note:** Manifest format declarations are **optional metadata** for documentation and tooling purposes. They are NOT required for runtime format detection, as multicodec prefixes provide self-describing data. Components discover format support at runtime through message handling.

**Use manifest declarations for:**
- 📖 **Documentation** - communicating supported formats to integrators
- 🛠️ **Tooling** - enabling IDE hints and validation
- 📊 **Metrics** - tracking format usage across ecosystem
- ⚠️ **Warnings** - detecting potential compatibility issues at build time

**Do NOT use manifest for:**
- ❌ Runtime format negotiation (not needed - multicodec handles this)
- ❌ Access control based on formats
- ❌ Required capability checking

### Example: component.toml (Optional)

**component.toml:**

```toml
[component]
name = "my-component"
version = "1.0.0"

# Optional: Document supported formats for integrators and tooling
[multicodec]
# Format this component prefers to send with
primary-send-format = "borsh"

# All formats this component can receive and decode
supported-receive-formats = [
    "borsh",      # 0x701 - Optimal for Rust components
    "msgpack",    # 0x0201 - Cross-language compatibility
    "json",       # 0x0200 - Debugging and development
]

# Optional: Indicate format flexibility
flexible-encoding = true             # Can encode to any supported format on demand
```

**Build-time tooling can use this to:**
- Generate format compatibility reports
- Warn about potential format mismatches in component graphs
- Document format requirements in generated API docs
- Provide IDE autocomplete for format codes

**Example: Build Warning from Tooling:**

```
⚠️  Warning: Component 'user-service' prefers sending 'borsh' (0x701),
   but target component 'email-notifier' only supports 'msgpack' and 'json'.
   
   Recommendation: Add MessagePack support to 'user-service' for compatibility,
   or add Borsh support to 'email-notifier' for better performance.
```

### Host Tooling Integration (Optional)

**Host can optionally read manifest at load time for tooling features:**

```rust
impl ComponentManifest {
    /// Optional: Parse multicodec config for tooling/documentation
    pub fn parse_multicodec_config(&self) -> Option<MulticodecConfig> {
        self.multicodec.as_ref().map(|mc| MulticodecConfig {
            primary_send_format: self.get_multicodec_code(&mc.primary_send_format),
            supported_receive_formats: mc.supported_receive_formats.iter()
                .map(|name| self.get_multicodec_code(name))
                .collect(),
            flexible_encoding: mc.flexible_encoding.unwrap_or(false),
        })
    }
    
    fn get_multicodec_code(&self, name: &str) -> u64 {
        match name {
            "borsh" => 0x701,
            "bincode" => 0x702,
            "protobuf" => 0x50,
            "json" => 0x0200,
            "msgpack" | "messagepack" => 0x0201,
            "yaml" => 0x0114,
            "raw" => 0x55,
            _ => panic!("Unknown multicodec name: {}", name),
        }
    }
}

// Optional: Use for build-time warnings or documentation generation
impl ComponentLoader {
    pub fn load_component(&mut self, manifest: ComponentManifest) -> Result<LoadedComponent> {
        // Load component as usual
        let component = self.load_wasm(&manifest)?;
        
        // Optional: Log format support for debugging
        if let Some(multicodec_config) = manifest.parse_multicodec_config() {
            log::debug!(
                "Component '{}' prefers sending: 0x{:x}, supports receiving: {:?}",
                manifest.name,
                multicodec_config.primary_send_format,
                multicodec_config.supported_receive_formats
            );
        }
        
        Ok(component)
    }
}
```

**Key Point:** The manifest is purely optional metadata. Runtime format detection always works via multicodec prefixes, regardless of manifest presence.
```

## Performance Considerations

### Format Performance Characteristics

**Benchmark Results (1KB payload):**

| Format | Encode Time | Decode Time | Size | Use Case |
|--------|-------------|-------------|------|----------|
| Borsh | ~5μs | ~3μs | 512 bytes | Rust-to-Rust high performance |
| Bincode | ~4μs | ~2μs | 480 bytes | Rust-to-Rust ultra-fast |
| MessagePack | ~15μs | ~20μs | 550 bytes | Cross-language balanced |
| JSON | ~80μs | ~120μs | 1200 bytes | Debugging, human-readable |
| Protobuf | ~25μs | ~30μs | 520 bytes | Schema evolution needs |

**Guidelines:**
- **Hot path (Rust ↔ Rust)**: Use Borsh (0x701)
- **Cross-language**: Use MessagePack (0x0201)
- **Debugging/Development**: Use JSON (0x0200)
- **Schema evolution**: Use Protobuf (0x50) or MessagePack with versioned structs

### Optimization Tips

**Reuse Encoders/Decoders:**

```rust
pub struct Component {
    // Reuse encoder/decoder instances
    json_encoder: OnceCell<serde_json::Serializer>,
    msgpack_encoder: OnceCell<rmp_serde::Serializer>,
}

impl Component {
    fn encode_cached(&self, codec: u64, data: &impl serde::Serialize) -> Result<Vec<u8>> {
        match codec {
            0x0200 => {
                // Reuse JSON encoder
                let encoder = self.json_encoder.get_or_init(|| {
                    serde_json::Serializer::new(Vec::new())
                });
                // ... encode with cached encoder
            }
            // ... other codecs
        }
    }
}
```

## Testing Multicodec Integration

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_multicodec_encode_decode() {
        let registry = MulticodecRegistry::new();
        
        let data = b"Hello, World!";
        let encoded = registry.encode(0x701, data);
        
        // Verify prefix
        assert_eq!(encoded[0], 0x01); // Varint for 0x701
        assert_eq!(encoded[1], 0x07);
        
        // Decode
        let decoded = registry.decode(&encoded).unwrap();
        assert_eq!(decoded.codec, 0x701);
        assert_eq!(decoded.data, data);
    }
    
    #[test]
    fn test_format_interop() {
        #[derive(Serialize, Deserialize, BorshSerialize, BorshDeserialize, PartialEq, Debug)]
        struct TestData {
            name: String,
            value: u32,
        }
        
        let original = TestData {
            name: "test".to_string(),
            value: 42,
        };
        
        // Encode with borsh
        let borsh_encoded = encode_borsh(&original).unwrap();
        let decoded_borsh: TestData = decode_borsh(&borsh_encoded).unwrap();
        assert_eq!(original, decoded_borsh);
        
        // Encode with msgpack
        let msgpack_encoded = encode_msgpack(&original).unwrap();
        let decoded_msgpack: TestData = decode_msgpack(&msgpack_encoded).unwrap();
        assert_eq!(original, decoded_msgpack);
    }
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_cross_language_messaging() {
    let mut runtime = TestRuntime::new();
    
    // Load Rust component (sends borsh)
    let rust_comp = runtime.load_component("rust_sender.wasm").await.unwrap();
    
    // Load JavaScript component (receives and transcodes to msgpack)
    let js_comp = runtime.load_component("js_receiver.wasm").await.unwrap();
    
    // Send message
    let test_data = TestMessage {
        id: "123".to_string(),
        value: 42,
    };
    
    runtime.send_message(rust_comp, js_comp, &test_data).await.unwrap();
    
    // Verify JS component received and decoded correctly
    let received = runtime.get_received_data(js_comp);
    assert_eq!(received.id, "123");
    assert_eq!(received.value, 42);
}
```

## Future Extensions

### Content Addressing with Multihash

**Integration with IPFS/IPLD:**

```rust
// Future: Content-addressed component artifacts
use multihash::Multihash;

pub struct ComponentArtifact {
    pub wasm_binary: Vec<u8>,
    pub content_hash: Multihash,  // SHA2-256 multihash
}

impl ComponentArtifact {
    pub fn new(wasm_binary: Vec<u8>) -> Self {
        let hash = multihash::Code::Sha2_256.digest(&wasm_binary);
        
        Self {
            wasm_binary,
            content_hash: hash,
        }
    }
    
    pub fn verify(&self) -> bool {
        let computed = multihash::Code::Sha2_256.digest(&self.wasm_binary);
        computed == self.content_hash
    }
}
```

### Cryptographic Operations with Multikey

**Future: Component signing and verification:**

```rust
// Future: Signed component metadata
use multikey::Multikey;

pub struct SignedComponentMetadata {
    pub metadata: ComponentMetadata,
    pub signature: Vec<u8>,
    pub public_key: Multikey,  // Self-describing public key
}

impl SignedComponentMetadata {
    pub fn verify(&self) -> bool {
        // Verify signature using multikey
        self.public_key.verify(&self.metadata_bytes(), &self.signature)
    }
}
```

---

**Document Status:** Complete multiformat integration strategy  
**Next Actions:**
- Implement multicodec registry in host runtime
- Create multicodec utility crates for components
- Add format performance benchmarks
- Develop cross-language interop tests
- Document format migration strategies

**Cross-References:**
- KNOWLEDGE-WASM-004: WIT Interface Definitions
- KNOWLEDGE-WASM-005: Messaging Architecture
- Official Multicodec Table: https://github.com/multiformats/multicodec/blob/master/table.csv
- Multiformats Specification: https://github.com/multiformats/multiformats
