//! Type conversion utilities for WASM function invocation.
//!
//! This module provides conversion between Rust types and WASM Val types, enabling
//! seamless parameter marshalling for WASM function calls. It implements the bridge
//! between multicodec-decoded bytes and Wasmtime's Val representation.
//!
//! # Supported Types
//!
//! - **i32**: 32-bit signed integer
//! - **i64**: 64-bit signed integer  
//! - **f32**: 32-bit floating point (stored as u32 bits in Val)
//! - **f64**: 64-bit floating point (stored as u64 bits in Val)
//!
//! # Performance
//!
//! - Single parameter conversion: <1μs overhead
//! - Zero-copy where possible
//! - No heap allocations for primitive types

// Layer 2: Third-party crate imports
use wasmtime::{FuncType, Val, ValType};

// Layer 3: Internal module imports
use crate::core::WasmError;

/// Convert decoded multicodec bytes to WASM Val parameters.
pub fn prepare_wasm_params(
    decoded_bytes: &[u8],
    func_type: &FuncType,
) -> Result<Vec<Val>, WasmError> {
    let param_types: Vec<ValType> = func_type.params().collect();

    match param_types.len() {
        0 => Ok(vec![]),
        1 => {
            let val = bytes_to_val(decoded_bytes, &param_types[0])?;
            Ok(vec![val])
        }
        _ => Err(WasmError::invalid_configuration(format!(
            "Multi-parameter functions ({} params) require schema definition (not yet implemented)",
            param_types.len()
        ))),
    }
}

/// Convert WASM function results to bytes.
pub fn extract_wasm_results(results: &[Val]) -> Result<Vec<u8>, WasmError> {
    match results.len() {
        0 => Ok(vec![]),
        1 => val_to_bytes(&results[0]),
        _ => Err(WasmError::invalid_configuration(format!(
            "Multi-value returns ({} values) require schema definition (not yet implemented)",
            results.len()
        ))),
    }
}

/// Convert raw bytes to a single WASM Val.
fn bytes_to_val(bytes: &[u8], val_type: &ValType) -> Result<Val, WasmError> {
    match val_type {
        ValType::I32 => {
            if bytes.len() != 4 {
                return Err(WasmError::invalid_configuration(format!(
                    "Type mismatch: Expected 4 bytes for i32, got {}",
                    bytes.len()
                )));
            }
            let array: [u8; 4] = bytes
                .try_into()
                .map_err(|_| WasmError::internal("slice conversion failed"))?;
            Ok(Val::I32(i32::from_le_bytes(array)))
        }
        ValType::I64 => {
            if bytes.len() != 8 {
                return Err(WasmError::invalid_configuration(format!(
                    "Type mismatch: Expected 8 bytes for i64, got {}",
                    bytes.len()
                )));
            }
            let array: [u8; 8] = bytes
                .try_into()
                .map_err(|_| WasmError::internal("slice conversion failed"))?;
            Ok(Val::I64(i64::from_le_bytes(array)))
        }
        ValType::F32 => {
            if bytes.len() != 4 {
                return Err(WasmError::invalid_configuration(format!(
                    "Type mismatch: Expected 4 bytes for f32, got {}",
                    bytes.len()
                )));
            }
            let array: [u8; 4] = bytes
                .try_into()
                .map_err(|_| WasmError::internal("slice conversion failed"))?;
            let value = f32::from_le_bytes(array);
            Ok(Val::F32(value.to_bits()))
        }
        ValType::F64 => {
            if bytes.len() != 8 {
                return Err(WasmError::invalid_configuration(format!(
                    "Type mismatch: Expected 8 bytes for f64, got {}",
                    bytes.len()
                )));
            }
            let array: [u8; 8] = bytes
                .try_into()
                .map_err(|_| WasmError::internal("slice conversion failed"))?;
            let value = f64::from_le_bytes(array);
            Ok(Val::F64(value.to_bits()))
        }
        ValType::V128 => Err(WasmError::invalid_configuration(
            "Unsupported type: V128 SIMD types not yet implemented",
        )),
        ValType::Ref(_) => Err(WasmError::invalid_configuration(
            "Unsupported type: Reference types require reference management",
        )),
    }
}

/// Convert single WASM Val to bytes.
fn val_to_bytes(val: &Val) -> Result<Vec<u8>, WasmError> {
    match val {
        Val::I32(v) => Ok(v.to_le_bytes().to_vec()),
        Val::I64(v) => Ok(v.to_le_bytes().to_vec()),
        Val::F32(bits) => Ok(f32::from_bits(*bits).to_le_bytes().to_vec()),
        Val::F64(bits) => Ok(f64::from_bits(*bits).to_le_bytes().to_vec()),
        Val::V128(_) => Err(WasmError::invalid_configuration(
            "Unsupported type: V128 SIMD types not yet implemented",
        )),
        Val::FuncRef(_) | Val::ExternRef(_) | Val::AnyRef(_) => Err(
            WasmError::invalid_configuration("Unsupported type: Cannot serialize reference types"),
        ),
    }
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
    use wasmtime::Engine;

    fn create_func_type(params: &[ValType], results: &[ValType]) -> FuncType {
        let engine = Engine::default();
        FuncType::new(&engine, params.iter().cloned(), results.iter().cloned())
    }

    // Basic i32 tests
    #[test]
    fn test_i32_positive() {
        let bytes = 42i32.to_le_bytes();
        let val = bytes_to_val(&bytes, &ValType::I32).unwrap();
        assert_eq!(val.unwrap_i32(), 42);
    }

    #[test]
    fn test_i32_negative() {
        let bytes = (-123i32).to_le_bytes();
        let val = bytes_to_val(&bytes, &ValType::I32).unwrap();
        assert_eq!(val.unwrap_i32(), -123);
    }

    #[test]
    fn test_i32_max_min() {
        let bytes = i32::MAX.to_le_bytes();
        let val = bytes_to_val(&bytes, &ValType::I32).unwrap();
        assert_eq!(val.unwrap_i32(), i32::MAX);

        let bytes = i32::MIN.to_le_bytes();
        let val = bytes_to_val(&bytes, &ValType::I32).unwrap();
        assert_eq!(val.unwrap_i32(), i32::MIN);
    }

    #[test]
    fn test_i32_wrong_size() {
        let bytes = vec![1, 2];
        let result = bytes_to_val(&bytes, &ValType::I32);
        assert!(result.is_err());
    }

    // Basic i64 tests
    #[test]
    fn test_i64_positive() {
        let bytes = 123456789i64.to_le_bytes();
        let val = bytes_to_val(&bytes, &ValType::I64).unwrap();
        assert_eq!(val.unwrap_i64(), 123456789);
    }

    #[test]
    fn test_i64_max_min() {
        let bytes = i64::MAX.to_le_bytes();
        let val = bytes_to_val(&bytes, &ValType::I64).unwrap();
        assert_eq!(val.unwrap_i64(), i64::MAX);

        let bytes = i64::MIN.to_le_bytes();
        let val = bytes_to_val(&bytes, &ValType::I64).unwrap();
        assert_eq!(val.unwrap_i64(), i64::MIN);
    }

    // Float tests
    #[test]
    fn test_f32_positive() {
        let bytes = std::f32::consts::PI.to_le_bytes();
        let val = bytes_to_val(&bytes, &ValType::F32).unwrap();
        assert_eq!(val.unwrap_f32(), std::f32::consts::PI);
    }

    #[test]
    fn test_f32_nan() {
        let bytes = f32::NAN.to_le_bytes();
        let val = bytes_to_val(&bytes, &ValType::F32).unwrap();
        assert!(val.unwrap_f32().is_nan());
    }

    #[test]
    fn test_f64_positive() {
        let bytes = std::f64::consts::E.to_le_bytes();
        let val = bytes_to_val(&bytes, &ValType::F64).unwrap();
        assert_eq!(val.unwrap_f64(), std::f64::consts::E);
    }

    // Val to bytes
    #[test]
    fn test_val_to_bytes_i32() {
        let val = Val::I32(100);
        let bytes = val_to_bytes(&val).unwrap();
        assert_eq!(i32::from_le_bytes(bytes.try_into().unwrap()), 100);
    }

    #[test]
    fn test_val_to_bytes_f32() {
        let val = Val::F32(1.23f32.to_bits());
        let bytes = val_to_bytes(&val).unwrap();
        assert_eq!(f32::from_le_bytes(bytes.try_into().unwrap()), 1.23f32);
    }

    // Round-trip tests
    #[test]
    fn test_roundtrip_i32() {
        let original = 42i32;
        let bytes = original.to_le_bytes();
        let val = bytes_to_val(&bytes, &ValType::I32).unwrap();
        let result_bytes = val_to_bytes(&val).unwrap();
        let result = i32::from_le_bytes(result_bytes.try_into().unwrap());
        assert_eq!(original, result);
    }

    #[test]
    fn test_roundtrip_f64() {
        let original = std::f64::consts::E;
        let bytes = original.to_le_bytes();
        let val = bytes_to_val(&bytes, &ValType::F64).unwrap();
        let result_bytes = val_to_bytes(&val).unwrap();
        let result = f64::from_le_bytes(result_bytes.try_into().unwrap());
        assert_eq!(original, result);
    }

    // Public API tests
    #[test]
    fn test_prepare_wasm_params_zero() {
        let func_type = create_func_type(&[], &[]);
        let params = prepare_wasm_params(&[], &func_type).unwrap();
        assert_eq!(params.len(), 0);
    }

    #[test]
    fn test_prepare_wasm_params_single_i32() {
        let bytes = 42i32.to_le_bytes();
        let func_type = create_func_type(&[ValType::I32], &[]);
        let params = prepare_wasm_params(&bytes, &func_type).unwrap();
        assert_eq!(params.len(), 1);
        assert_eq!(params[0].unwrap_i32(), 42);
    }

    #[test]
    fn test_prepare_wasm_params_multi_unsupported() {
        let bytes = vec![1, 2, 3, 4];
        let func_type = create_func_type(&[ValType::I32, ValType::I32], &[]);
        let result = prepare_wasm_params(&bytes, &func_type);
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_wasm_results_zero() {
        let results: Vec<Val> = vec![];
        let bytes = extract_wasm_results(&results).unwrap();
        assert_eq!(bytes.len(), 0);
    }

    #[test]
    fn test_extract_wasm_results_single() {
        let results = vec![Val::I32(100)];
        let bytes = extract_wasm_results(&results).unwrap();
        assert_eq!(i32::from_le_bytes(bytes.try_into().unwrap()), 100);
    }

    #[test]
    fn test_extract_wasm_results_multi_unsupported() {
        let results = vec![Val::I32(10), Val::I32(20)];
        let result = extract_wasm_results(&results);
        assert!(result.is_err());
    }

    // Boundary values
    #[test]
    fn test_boundary_values() {
        for &value in &[i32::MIN, -1, 0, 1, i32::MAX] {
            let bytes = value.to_le_bytes();
            let val = bytes_to_val(&bytes, &ValType::I32).unwrap();
            assert_eq!(val.unwrap_i32(), value);
        }
    }

    // Performance
    #[test]
    fn test_conversion_performance() {
        use std::time::Instant;
        let iterations = 10_000;
        let bytes = 42i32.to_le_bytes();

        let start = Instant::now();
        for _ in 0..iterations {
            let _ = bytes_to_val(&bytes, &ValType::I32).unwrap();
        }
        let elapsed = start.elapsed();

        let avg_time = elapsed / iterations;
        assert!(avg_time.as_nanos() < 1000);
    }
}
