//! Multicodec support for message serialization.
//!
//! This module implements multicodec-prefixed data encoding and decoding as specified
//! in ADR-WASM-001 (Inter-Component Communication Design). Multicodec provides
//! self-describing data format identification through variable-length integer prefixes.
//!
//! # Supported Codecs
//!
//! - **Borsh** (0x701): Rust-native binary serialization (preferred for performance)
//! - **CBOR** (0x51): Concise Binary Object Representation (RFC 7049)
//! - **JSON** (0x0200): Human-readable text format (UTF-8 JSON)
//!
//! # Multicodec Format
//!
//! ```text
//! ┌─────────────┬──────────────┐
//! │ Varint Code │   Payload    │
//! │  (1-4 bytes)│ (0+ bytes)   │
//! └─────────────┴──────────────┘
//! ```
//!
//! # Usage Example
//!
//! ```rust
//! use airssys_wasm::core::multicodec::{Codec, encode_multicodec, decode_multicodec};
//!
//! // Encode with Borsh
//! let data = b"Hello, WASM!";
//! let encoded = encode_multicodec(Codec::Borsh, data).unwrap();
//!
//! // Decode automatically detects codec
//! let (codec, decoded) = decode_multicodec(&encoded).unwrap();
//! assert_eq!(codec, Codec::Borsh);
//! assert_eq!(decoded, data);
//! ```
//!
//! # References
//!
//! - **ADR-WASM-001**: Inter-Component Communication Design
//! - **Multicodec Spec**: <https://github.com/multiformats/multicodec>
//! - **KNOWLEDGE-WASM-016**: Actor System Integration Implementation Guide (lines 438-666)

// Layer 1: Standard library imports
use std::fmt;

// Layer 2: Third-party crate imports
use serde::{Deserialize, Serialize};

// Layer 3: Internal module imports
use super::error::{WasmError, WasmResult};

/// Supported serialization codecs as per ADR-WASM-001.
///
/// Each codec is identified by a multicodec varint prefix that appears
/// at the start of encoded data. The varint encoding follows the unsigned
/// varint format from the multicodec specification.
///
/// # Codec Selection Guidelines
///
/// - **Borsh**: Best for Rust-to-Rust communication (compact, fast)
/// - **CBOR**: Best for cross-language interop (well-supported)
/// - **JSON**: Best for debugging and human-readable data
///
/// # Example
///
/// ```rust
/// use airssys_wasm::core::multicodec::Codec;
///
/// let codec = Codec::Borsh;
/// assert_eq!(codec as u32, 0x701);
/// assert_eq!(codec.name(), "borsh");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u32)]
pub enum Codec {
    /// Borsh binary serialization (0x701).
    ///
    /// Rust-native serialization format with excellent performance characteristics.
    /// Preferred for Rust-to-Rust component communication.
    ///
    /// **Performance**: Fastest encoding/decoding
    /// **Size**: Most compact representation
    /// **Interop**: Limited to Borsh-supporting languages
    Borsh = 0x701,

    /// CBOR binary serialization (0x51).
    ///
    /// Concise Binary Object Representation (RFC 7049) with broad language support.
    /// Good choice for cross-language component communication.
    ///
    /// **Performance**: Moderate encoding/decoding speed
    /// **Size**: Compact binary format
    /// **Interop**: Excellent (CBOR supported in most languages)
    CBOR = 0x51,

    /// JSON text serialization (0x0200).
    ///
    /// UTF-8 encoded JSON text format for maximum readability and debugging.
    /// Use for development and debugging scenarios.
    ///
    /// **Performance**: Slower than binary formats
    /// **Size**: Largest representation (text-based)
    /// **Interop**: Universal (JSON supported everywhere)
    JSON = 0x0200,
}

impl Codec {
    /// Create codec from multicodec varint value.
    ///
    /// # Errors
    ///
    /// Returns `WasmError::SerializationError` if the varint does not match
    /// a supported codec.
    ///
    /// # Example
    ///
    /// ```rust
    /// use airssys_wasm::core::multicodec::Codec;
    ///
    /// let codec = Codec::from_varint(0x701).unwrap();
    /// assert_eq!(codec, Codec::Borsh);
    ///
    /// let invalid = Codec::from_varint(0xFFFF);
    /// assert!(invalid.is_err());
    /// ```
    pub fn from_varint(varint: u32) -> WasmResult<Self> {
        match varint {
            0x701 => Ok(Codec::Borsh),
            0x51 => Ok(Codec::CBOR),
            0x0200 => Ok(Codec::JSON),
            v => Err(WasmError::serialization_error(format!(
                "Unsupported codec varint: 0x{v:x}"
            ))),
        }
    }

    /// Get codec name as string.
    ///
    /// # Example
    ///
    /// ```rust
    /// use airssys_wasm::core::multicodec::Codec;
    ///
    /// assert_eq!(Codec::Borsh.name(), "borsh");
    /// assert_eq!(Codec::CBOR.name(), "cbor");
    /// assert_eq!(Codec::JSON.name(), "json");
    /// ```
    pub const fn name(self) -> &'static str {
        match self {
            Codec::Borsh => "borsh",
            Codec::CBOR => "cbor",
            Codec::JSON => "json",
        }
    }

    /// Check if codec is binary format.
    ///
    /// # Example
    ///
    /// ```rust
    /// use airssys_wasm::core::multicodec::Codec;
    ///
    /// assert!(Codec::Borsh.is_binary());
    /// assert!(Codec::CBOR.is_binary());
    /// assert!(!Codec::JSON.is_binary());
    /// ```
    pub const fn is_binary(self) -> bool {
        matches!(self, Codec::Borsh | Codec::CBOR)
    }

    /// Check if codec is text format.
    ///
    /// # Example
    ///
    /// ```rust
    /// use airssys_wasm::core::multicodec::Codec;
    ///
    /// assert!(!Codec::Borsh.is_text());
    /// assert!(!Codec::CBOR.is_text());
    /// assert!(Codec::JSON.is_text());
    /// ```
    pub const fn is_text(self) -> bool {
        matches!(self, Codec::JSON)
    }
}

impl fmt::Display for Codec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Encode data with multicodec prefix.
///
/// Prepends the codec's varint identifier to the payload data. The varint
/// is encoded using unsigned varint format (1-4 bytes).
///
/// # Arguments
///
/// * `codec` - Serialization codec to use
/// * `data` - Raw payload bytes (already serialized)
///
/// # Returns
///
/// Vector containing varint prefix + payload
///
/// # Example
///
/// ```rust
/// use airssys_wasm::core::multicodec::{Codec, encode_multicodec};
///
/// let payload = b"test data";
/// let encoded = encode_multicodec(Codec::Borsh, payload).unwrap();
///
/// // Encoded data starts with Borsh varint (0x701)
/// assert!(encoded.len() > payload.len());
/// ```
pub fn encode_multicodec(codec: Codec, data: &[u8]) -> WasmResult<Vec<u8>> {
    let mut result = Vec::with_capacity(data.len() + 4);

    // Encode varint prefix (1-4 bytes)
    let mut varint = codec as u32;
    while varint >= 0x80 {
        result.push(((varint & 0x7F) as u8) | 0x80);
        varint >>= 7;
    }
    result.push(varint as u8);

    // Append payload
    result.extend_from_slice(data);

    Ok(result)
}

/// Decode multicodec-prefixed data.
///
/// Reads the varint prefix to identify the codec, then returns both the
/// codec identifier and the raw payload bytes. The caller is responsible
/// for deserializing the payload using the appropriate codec.
///
/// # Arguments
///
/// * `data` - Multicodec-prefixed data
///
/// # Returns
///
/// Tuple of (codec, payload_bytes)
///
/// # Errors
///
/// Returns `WasmError::SerializationError` if:
/// - Data is empty
/// - Varint prefix is invalid
/// - Varint prefix doesn't match a supported codec
///
/// # Example
///
/// ```rust
/// use airssys_wasm::core::multicodec::{Codec, encode_multicodec, decode_multicodec};
///
/// let original = b"Hello, WASM!";
/// let encoded = encode_multicodec(Codec::JSON, original).unwrap();
///
/// let (codec, decoded) = decode_multicodec(&encoded).unwrap();
/// assert_eq!(codec, Codec::JSON);
/// assert_eq!(decoded, original);
/// ```
pub fn decode_multicodec(data: &[u8]) -> WasmResult<(Codec, Vec<u8>)> {
    if data.is_empty() {
        return Err(WasmError::serialization_error(
            "Cannot decode empty multicodec data",
        ));
    }

    // Read varint prefix (1-4 bytes)
    let mut cursor = 0;
    let mut varint = 0u32;
    let mut shift = 0;

    loop {
        if cursor >= data.len() {
            return Err(WasmError::serialization_error(
                "Truncated multicodec data (incomplete varint)",
            ));
        }

        let byte = data[cursor];
        cursor += 1;

        varint |= ((byte & 0x7F) as u32) << shift;

        // Check if this is the last byte (high bit clear)
        if byte & 0x80 == 0 {
            break;
        }

        shift += 7;
        if shift > 28 {
            return Err(WasmError::serialization_error(
                "Invalid multicodec varint (too large)",
            ));
        }
    }

    // Identify codec
    let codec = Codec::from_varint(varint)?;

    // Extract payload
    let payload = data[cursor..].to_vec();

    Ok((codec, payload))
}

#[allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::too_many_arguments,
    clippy::type_complexity,
    reason = "test code"
)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_codec_from_varint_valid() {
        assert_eq!(Codec::from_varint(0x701).unwrap(), Codec::Borsh);
        assert_eq!(Codec::from_varint(0x51).unwrap(), Codec::CBOR);
        assert_eq!(Codec::from_varint(0x0200).unwrap(), Codec::JSON);
    }

    #[test]
    fn test_codec_from_varint_invalid() {
        let result = Codec::from_varint(0xFFFF);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Unsupported codec"));
    }

    #[test]
    fn test_codec_name() {
        assert_eq!(Codec::Borsh.name(), "borsh");
        assert_eq!(Codec::CBOR.name(), "cbor");
        assert_eq!(Codec::JSON.name(), "json");
    }

    #[test]
    fn test_codec_is_binary() {
        assert!(Codec::Borsh.is_binary());
        assert!(Codec::CBOR.is_binary());
        assert!(!Codec::JSON.is_binary());
    }

    #[test]
    fn test_codec_is_text() {
        assert!(!Codec::Borsh.is_text());
        assert!(!Codec::CBOR.is_text());
        assert!(Codec::JSON.is_text());
    }

    #[test]
    fn test_codec_display() {
        assert_eq!(format!("{}", Codec::Borsh), "borsh");
        assert_eq!(format!("{}", Codec::CBOR), "cbor");
        assert_eq!(format!("{}", Codec::JSON), "json");
    }

    #[test]
    fn test_encode_borsh() {
        let data = b"test payload";
        let encoded = encode_multicodec(Codec::Borsh, data).unwrap();

        // Borsh varint is 0x701 = 1793 decimal
        // Varint encoding: 0x81 0x0E (two bytes: 129, 14)
        assert_eq!(encoded[0], 0x81);
        assert_eq!(encoded[1], 0x0E);
        assert_eq!(&encoded[2..], data);
    }

    #[test]
    fn test_encode_cbor() {
        let data = b"cbor data";
        let encoded = encode_multicodec(Codec::CBOR, data).unwrap();

        // CBOR varint is 0x51 = 81 decimal
        // Varint encoding: 0x51 (one byte: 81)
        assert_eq!(encoded[0], 0x51);
        assert_eq!(&encoded[1..], data);
    }

    #[test]
    fn test_encode_json() {
        let data = b"{\"key\": \"value\"}";
        let encoded = encode_multicodec(Codec::JSON, data).unwrap();

        // JSON varint is 0x0200 = 512 decimal
        // Varint encoding: 0x80 0x04 (two bytes: 128, 4)
        assert_eq!(encoded[0], 0x80);
        assert_eq!(encoded[1], 0x04);
        assert_eq!(&encoded[2..], data);
    }

    #[test]
    fn test_decode_borsh() {
        let original = b"borsh payload";
        let encoded = encode_multicodec(Codec::Borsh, original).unwrap();

        let (codec, decoded) = decode_multicodec(&encoded).unwrap();
        assert_eq!(codec, Codec::Borsh);
        assert_eq!(decoded, original);
    }

    #[test]
    fn test_decode_cbor() {
        let original = b"cbor payload";
        let encoded = encode_multicodec(Codec::CBOR, original).unwrap();

        let (codec, decoded) = decode_multicodec(&encoded).unwrap();
        assert_eq!(codec, Codec::CBOR);
        assert_eq!(decoded, original);
    }

    #[test]
    fn test_decode_json() {
        let original = br#"{"message":"hello"}"#;
        let encoded = encode_multicodec(Codec::JSON, original).unwrap();

        let (codec, decoded) = decode_multicodec(&encoded).unwrap();
        assert_eq!(codec, Codec::JSON);
        assert_eq!(decoded, original);
    }

    #[test]
    fn test_round_trip_all_codecs() {
        let codecs = [Codec::Borsh, Codec::CBOR, Codec::JSON];
        let test_data = b"round trip test data";

        for codec in codecs {
            let encoded = encode_multicodec(codec, test_data).unwrap();
            let (decoded_codec, decoded_data) = decode_multicodec(&encoded).unwrap();

            assert_eq!(decoded_codec, codec, "Codec mismatch for {codec}");
            assert_eq!(decoded_data, test_data, "Data mismatch for {codec}");
        }
    }

    #[test]
    fn test_decode_empty_data() {
        let result = decode_multicodec(&[]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Cannot decode empty"));
    }

    #[test]
    fn test_decode_truncated_varint() {
        // Varint with high bit set but no continuation
        let truncated = vec![0x80];
        let result = decode_multicodec(&truncated);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Truncated"));
    }

    #[test]
    fn test_decode_invalid_varint_too_large() {
        // Varint that's too large (5+ bytes)
        let invalid = vec![0x80, 0x80, 0x80, 0x80, 0x80];
        let result = decode_multicodec(&invalid);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("too large"));
    }

    #[test]
    fn test_encode_empty_payload() {
        let empty: &[u8] = &[];
        let encoded = encode_multicodec(Codec::Borsh, empty).unwrap();

        // Should contain only varint prefix
        assert_eq!(encoded.len(), 2); // Borsh varint is 2 bytes
        assert_eq!(encoded[0], 0x81);
        assert_eq!(encoded[1], 0x0E);
    }

    #[test]
    fn test_encode_large_payload() {
        let large = vec![0xAB; 10_000];
        let encoded = encode_multicodec(Codec::CBOR, &large).unwrap();

        let (codec, decoded) = decode_multicodec(&encoded).unwrap();
        assert_eq!(codec, Codec::CBOR);
        assert_eq!(decoded, large);
    }
}
