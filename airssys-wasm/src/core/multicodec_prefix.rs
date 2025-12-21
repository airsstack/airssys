//! Multicodec prefix parsing and validation for inter-component messages.
//!
//! This module provides multicodec prefix detection according to ADR-WASM-001.
//! The host runtime MUST parse and validate multicodec prefixes for all
//! inter-component messages, but does NOT translate between codecs.
//!
//! # ADR-WASM-001 Compliance
//!
//! Per ADR-WASM-001 (Multicodec Compatibility Strategy):
//! - Host validates codec compatibility but does NOT translate between codecs
//! - Messages are routed as opaque bytes
//! - Fail fast with clear errors for incompatible codecs
//!
//! # Supported Codecs
//!
//! | Codec | Prefix (2 bytes BE) | Description |
//! |-------|---------------------|-------------|
//! | Borsh | 0x0701 | Default for airssys components |
//! | Bincode | 0x0702 | Alternative binary format |
//! | MessagePack | 0x0201 | JSON-compatible binary |
//! | Protobuf | 0x0050 | Protocol Buffers |
//!
//! # Prefix Format
//!
//! The multicodec prefix is a 2-byte big-endian unsigned integer at the start
//! of each message. This differs from the varint encoding used in some multicodec
//! implementations but provides simpler and faster parsing (~10ns).
//!
//! ```text
//! ┌──────────────┬──────────────┐
//! │  Byte 0 (HI) │  Byte 1 (LO) │  Payload...
//! └──────────────┴──────────────┘
//! ```
//!
//! # Usage Example
//!
//! ```rust
//! use airssys_wasm::core::multicodec_prefix::MulticodecPrefix;
//!
//! // Create a message with borsh prefix
//! let mut message = MulticodecPrefix::Borsh.prefix_bytes().to_vec();
//! message.extend_from_slice(b"actual payload");
//!
//! // Parse prefix from message
//! let (codec, prefix_len) = MulticodecPrefix::from_prefix(&message).unwrap();
//! assert_eq!(codec, MulticodecPrefix::Borsh);
//! assert_eq!(prefix_len, 2);
//! ```
//!
//! # References
//!
//! - ADR-WASM-001: Multicodec Compatibility Strategy
//! - KNOWLEDGE-WASM-024 Section 6: Multicodec Serialization Requirements

// Layer 1: Standard library imports
// (none)

// Layer 2: Third-party crate imports
use serde::{Deserialize, Serialize};
use thiserror::Error;

// Layer 3: Internal module imports
// (none - this module has no internal dependencies)

/// Supported multicodec types for inter-component messages (ADR-WASM-001).
///
/// Each variant represents a serialization format identified by a 2-byte
/// big-endian prefix. The host runtime validates these prefixes but does
/// NOT translate between codecs.
///
/// # Performance
///
/// - Prefix parsing: ~10ns (read 2 bytes, match enum)
/// - No codec translation overhead
///
/// # Example
///
/// ```rust
/// use airssys_wasm::core::multicodec_prefix::MulticodecPrefix;
///
/// let codec = MulticodecPrefix::Borsh;
/// assert_eq!(codec.prefix_value(), 0x0701);
/// assert_eq!(codec.name(), "borsh");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MulticodecPrefix {
    /// Borsh binary serialization (airssys default)
    ///
    /// Prefix: 0x0701 (big-endian)
    ///
    /// Recommended for Rust-to-Rust component communication.
    /// Deterministic and cross-language compatible.
    Borsh,

    /// Bincode binary serialization
    ///
    /// Prefix: 0x0702 (big-endian)
    ///
    /// Fast Rust-only serialization. Use when performance is critical
    /// and all components are Rust-based.
    Bincode,

    /// MessagePack serialization (JSON-compatible)
    ///
    /// Prefix: 0x0201 (big-endian)
    ///
    /// Good for cross-language interop with JavaScript/Python components.
    MessagePack,

    /// Protocol Buffers serialization
    ///
    /// Prefix: 0x0050 (big-endian)
    ///
    /// Schema-based serialization with excellent cross-language support.
    Protobuf,
}

/// Error types for multicodec prefix operations.
#[derive(Debug, Error)]
pub enum MulticodecPrefixError {
    /// Message too short to contain multicodec prefix
    #[error("Message too short: expected at least 2 bytes, got {actual}")]
    MessageTooShort {
        /// Actual message length
        actual: usize,
    },

    /// Unknown or unsupported multicodec prefix
    #[error("Unknown multicodec prefix: 0x{prefix:04X}")]
    UnknownPrefix {
        /// The unrecognized prefix value
        prefix: u16,
    },
}

impl MulticodecPrefix {
    /// Parse multicodec from message bytes (ADR-WASM-001 compliant).
    ///
    /// Reads the first 2 bytes as a big-endian unsigned integer and
    /// identifies the codec. This is the REQUIRED validation step for
    /// all inter-component messages per ADR-WASM-001.
    ///
    /// # Arguments
    ///
    /// * `data` - Message bytes (must include multicodec prefix)
    ///
    /// # Returns
    ///
    /// * `Ok((MulticodecPrefix, prefix_len))` - Detected codec and prefix length (always 2)
    /// * `Err(MulticodecPrefixError)` - If prefix is invalid or unknown
    ///
    /// # Performance
    ///
    /// ~10ns (read 2 bytes, match enum)
    ///
    /// # Example
    ///
    /// ```rust
    /// use airssys_wasm::core::multicodec_prefix::MulticodecPrefix;
    ///
    /// let message = [0x07, 0x01, 0xDE, 0xAD, 0xBE, 0xEF];
    /// let (codec, len) = MulticodecPrefix::from_prefix(&message).unwrap();
    /// assert_eq!(codec, MulticodecPrefix::Borsh);
    /// assert_eq!(len, 2);
    /// ```
    pub fn from_prefix(data: &[u8]) -> Result<(Self, usize), MulticodecPrefixError> {
        if data.len() < 2 {
            return Err(MulticodecPrefixError::MessageTooShort { actual: data.len() });
        }

        let prefix = u16::from_be_bytes([data[0], data[1]]);

        match prefix {
            0x0701 => Ok((Self::Borsh, 2)),
            0x0702 => Ok((Self::Bincode, 2)),
            0x0201 => Ok((Self::MessagePack, 2)),
            0x0050 => Ok((Self::Protobuf, 2)),
            _ => Err(MulticodecPrefixError::UnknownPrefix { prefix }),
        }
    }

    /// Get the prefix bytes for this codec.
    ///
    /// Returns the 2-byte big-endian prefix that identifies this codec.
    ///
    /// # Example
    ///
    /// ```rust
    /// use airssys_wasm::core::multicodec_prefix::MulticodecPrefix;
    ///
    /// assert_eq!(MulticodecPrefix::Borsh.prefix_bytes(), [0x07, 0x01]);
    /// assert_eq!(MulticodecPrefix::MessagePack.prefix_bytes(), [0x02, 0x01]);
    /// ```
    pub fn prefix_bytes(&self) -> [u8; 2] {
        match self {
            Self::Borsh => [0x07, 0x01],
            Self::Bincode => [0x07, 0x02],
            Self::MessagePack => [0x02, 0x01],
            Self::Protobuf => [0x00, 0x50],
        }
    }

    /// Get the prefix value as u16 for this codec.
    ///
    /// # Example
    ///
    /// ```rust
    /// use airssys_wasm::core::multicodec_prefix::MulticodecPrefix;
    ///
    /// assert_eq!(MulticodecPrefix::Borsh.prefix_value(), 0x0701);
    /// assert_eq!(MulticodecPrefix::Bincode.prefix_value(), 0x0702);
    /// ```
    pub fn prefix_value(&self) -> u16 {
        match self {
            Self::Borsh => 0x0701,
            Self::Bincode => 0x0702,
            Self::MessagePack => 0x0201,
            Self::Protobuf => 0x0050,
        }
    }

    /// Get human-readable name for this codec.
    ///
    /// The name is used for capability matching and error messages.
    ///
    /// # Example
    ///
    /// ```rust
    /// use airssys_wasm::core::multicodec_prefix::MulticodecPrefix;
    ///
    /// assert_eq!(MulticodecPrefix::Borsh.name(), "borsh");
    /// assert_eq!(MulticodecPrefix::MessagePack.name(), "messagepack");
    /// ```
    pub fn name(&self) -> &'static str {
        match self {
            Self::Borsh => "borsh",
            Self::Bincode => "bincode",
            Self::MessagePack => "messagepack",
            Self::Protobuf => "protobuf",
        }
    }

    /// Check if this codec is a binary format.
    ///
    /// All supported codecs are binary formats.
    ///
    /// # Example
    ///
    /// ```rust
    /// use airssys_wasm::core::multicodec_prefix::MulticodecPrefix;
    ///
    /// assert!(MulticodecPrefix::Borsh.is_binary());
    /// assert!(MulticodecPrefix::MessagePack.is_binary());
    /// ```
    pub fn is_binary(&self) -> bool {
        // All ADR-WASM-001 codecs are binary formats
        true
    }

    /// Get payload from message (skips prefix).
    ///
    /// Convenience method to extract the payload bytes after the prefix.
    ///
    /// # Arguments
    ///
    /// * `data` - Full message with prefix
    ///
    /// # Returns
    ///
    /// * `Ok(&[u8])` - Payload bytes (reference to slice after prefix)
    /// * `Err(MulticodecPrefixError)` - If message is too short
    ///
    /// # Example
    ///
    /// ```rust
    /// use airssys_wasm::core::multicodec_prefix::MulticodecPrefix;
    ///
    /// let message = [0x07, 0x01, 0xDE, 0xAD, 0xBE, 0xEF];
    /// let payload = MulticodecPrefix::get_payload(&message).unwrap();
    /// assert_eq!(payload, &[0xDE, 0xAD, 0xBE, 0xEF]);
    /// ```
    pub fn get_payload(data: &[u8]) -> Result<&[u8], MulticodecPrefixError> {
        if data.len() < 2 {
            return Err(MulticodecPrefixError::MessageTooShort { actual: data.len() });
        }
        Ok(&data[2..])
    }

    /// Create message with this codec's prefix.
    ///
    /// Prepends the codec prefix to the payload bytes.
    ///
    /// # Arguments
    ///
    /// * `payload` - Raw payload bytes (already serialized)
    ///
    /// # Returns
    ///
    /// Vector containing prefix + payload
    ///
    /// # Example
    ///
    /// ```rust
    /// use airssys_wasm::core::multicodec_prefix::MulticodecPrefix;
    ///
    /// let payload = b"test data";
    /// let message = MulticodecPrefix::Borsh.create_message(payload);
    /// assert_eq!(&message[0..2], &[0x07, 0x01]);
    /// assert_eq!(&message[2..], payload);
    /// ```
    pub fn create_message(&self, payload: &[u8]) -> Vec<u8> {
        let mut message = Vec::with_capacity(2 + payload.len());
        message.extend_from_slice(&self.prefix_bytes());
        message.extend_from_slice(payload);
        message
    }
}

impl std::fmt::Display for MulticodecPrefix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} (0x{:04X})", self.name(), self.prefix_value())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================================
    // from_prefix Tests
    // ============================================================================

    #[test]
    fn test_from_prefix_borsh() {
        let data = [0x07, 0x01, 0xDE, 0xAD];
        let (codec, len) = MulticodecPrefix::from_prefix(&data).unwrap();
        assert_eq!(codec, MulticodecPrefix::Borsh);
        assert_eq!(len, 2);
    }

    #[test]
    fn test_from_prefix_bincode() {
        let data = [0x07, 0x02, 0xBE, 0xEF];
        let (codec, len) = MulticodecPrefix::from_prefix(&data).unwrap();
        assert_eq!(codec, MulticodecPrefix::Bincode);
        assert_eq!(len, 2);
    }

    #[test]
    fn test_from_prefix_messagepack() {
        let data = [0x02, 0x01, 0xCA, 0xFE];
        let (codec, len) = MulticodecPrefix::from_prefix(&data).unwrap();
        assert_eq!(codec, MulticodecPrefix::MessagePack);
        assert_eq!(len, 2);
    }

    #[test]
    fn test_from_prefix_protobuf() {
        let data = [0x00, 0x50, 0xAB, 0xCD];
        let (codec, len) = MulticodecPrefix::from_prefix(&data).unwrap();
        assert_eq!(codec, MulticodecPrefix::Protobuf);
        assert_eq!(len, 2);
    }

    #[test]
    fn test_from_prefix_too_short_empty() {
        let data: [u8; 0] = [];
        let err = MulticodecPrefix::from_prefix(&data).unwrap_err();
        assert!(matches!(err, MulticodecPrefixError::MessageTooShort { actual: 0 }));
        assert!(err.to_string().contains("0"));
    }

    #[test]
    fn test_from_prefix_too_short_one_byte() {
        let data = [0x07];
        let err = MulticodecPrefix::from_prefix(&data).unwrap_err();
        assert!(matches!(err, MulticodecPrefixError::MessageTooShort { actual: 1 }));
    }

    #[test]
    fn test_from_prefix_unknown() {
        let data = [0xFF, 0xFF, 0x00, 0x00];
        let err = MulticodecPrefix::from_prefix(&data).unwrap_err();
        assert!(matches!(err, MulticodecPrefixError::UnknownPrefix { prefix: 0xFFFF }));
        assert!(err.to_string().contains("FFFF"));
    }

    // ============================================================================
    // prefix_bytes Tests
    // ============================================================================

    #[test]
    fn test_prefix_bytes_all_codecs() {
        assert_eq!(MulticodecPrefix::Borsh.prefix_bytes(), [0x07, 0x01]);
        assert_eq!(MulticodecPrefix::Bincode.prefix_bytes(), [0x07, 0x02]);
        assert_eq!(MulticodecPrefix::MessagePack.prefix_bytes(), [0x02, 0x01]);
        assert_eq!(MulticodecPrefix::Protobuf.prefix_bytes(), [0x00, 0x50]);
    }

    // ============================================================================
    // prefix_value Tests
    // ============================================================================

    #[test]
    fn test_prefix_value_all_codecs() {
        assert_eq!(MulticodecPrefix::Borsh.prefix_value(), 0x0701);
        assert_eq!(MulticodecPrefix::Bincode.prefix_value(), 0x0702);
        assert_eq!(MulticodecPrefix::MessagePack.prefix_value(), 0x0201);
        assert_eq!(MulticodecPrefix::Protobuf.prefix_value(), 0x0050);
    }

    // ============================================================================
    // name Tests
    // ============================================================================

    #[test]
    fn test_name_all_codecs() {
        assert_eq!(MulticodecPrefix::Borsh.name(), "borsh");
        assert_eq!(MulticodecPrefix::Bincode.name(), "bincode");
        assert_eq!(MulticodecPrefix::MessagePack.name(), "messagepack");
        assert_eq!(MulticodecPrefix::Protobuf.name(), "protobuf");
    }

    // ============================================================================
    // Round-trip Tests
    // ============================================================================

    #[test]
    fn test_prefix_round_trip_borsh() {
        let codec = MulticodecPrefix::Borsh;
        let bytes = codec.prefix_bytes();
        let (parsed, _) = MulticodecPrefix::from_prefix(&bytes).unwrap();
        assert_eq!(parsed, codec);
    }

    #[test]
    fn test_prefix_round_trip_bincode() {
        let codec = MulticodecPrefix::Bincode;
        let bytes = codec.prefix_bytes();
        let (parsed, _) = MulticodecPrefix::from_prefix(&bytes).unwrap();
        assert_eq!(parsed, codec);
    }

    #[test]
    fn test_prefix_round_trip_messagepack() {
        let codec = MulticodecPrefix::MessagePack;
        let bytes = codec.prefix_bytes();
        let (parsed, _) = MulticodecPrefix::from_prefix(&bytes).unwrap();
        assert_eq!(parsed, codec);
    }

    #[test]
    fn test_prefix_round_trip_protobuf() {
        let codec = MulticodecPrefix::Protobuf;
        let bytes = codec.prefix_bytes();
        let (parsed, _) = MulticodecPrefix::from_prefix(&bytes).unwrap();
        assert_eq!(parsed, codec);
    }

    // ============================================================================
    // get_payload Tests
    // ============================================================================

    #[test]
    fn test_get_payload_success() {
        let data = [0x07, 0x01, 0xDE, 0xAD, 0xBE, 0xEF];
        let payload = MulticodecPrefix::get_payload(&data).unwrap();
        assert_eq!(payload, &[0xDE, 0xAD, 0xBE, 0xEF]);
    }

    #[test]
    fn test_get_payload_empty_payload() {
        let data = [0x07, 0x01];
        let payload = MulticodecPrefix::get_payload(&data).unwrap();
        let empty: &[u8] = &[];
        assert_eq!(payload, empty);
    }

    #[test]
    fn test_get_payload_too_short() {
        let data = [0x07];
        let err = MulticodecPrefix::get_payload(&data).unwrap_err();
        assert!(matches!(err, MulticodecPrefixError::MessageTooShort { actual: 1 }));
    }

    // ============================================================================
    // create_message Tests
    // ============================================================================

    #[test]
    fn test_create_message_borsh() {
        let payload = b"test data";
        let message = MulticodecPrefix::Borsh.create_message(payload);
        assert_eq!(&message[0..2], &[0x07, 0x01]);
        assert_eq!(&message[2..], payload);
    }

    #[test]
    fn test_create_message_empty_payload() {
        let message = MulticodecPrefix::MessagePack.create_message(&[]);
        assert_eq!(message, vec![0x02, 0x01]);
    }

    #[test]
    fn test_create_message_round_trip() {
        let payload = b"round trip test";
        let message = MulticodecPrefix::Bincode.create_message(payload);
        
        let (codec, len) = MulticodecPrefix::from_prefix(&message).unwrap();
        assert_eq!(codec, MulticodecPrefix::Bincode);
        assert_eq!(&message[len..], payload);
    }

    // ============================================================================
    // Display Tests
    // ============================================================================

    #[test]
    fn test_display_format() {
        let s = format!("{}", MulticodecPrefix::Borsh);
        assert_eq!(s, "borsh (0x0701)");
    }

    // ============================================================================
    // is_binary Tests
    // ============================================================================

    #[test]
    fn test_is_binary_all_codecs() {
        assert!(MulticodecPrefix::Borsh.is_binary());
        assert!(MulticodecPrefix::Bincode.is_binary());
        assert!(MulticodecPrefix::MessagePack.is_binary());
        assert!(MulticodecPrefix::Protobuf.is_binary());
    }

    // ============================================================================
    // Serialization Tests
    // ============================================================================

    #[test]
    fn test_serde_serialization() {
        let codec = MulticodecPrefix::Borsh;
        let json = serde_json::to_string(&codec).unwrap();
        let deserialized: MulticodecPrefix = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, codec);
    }
}
