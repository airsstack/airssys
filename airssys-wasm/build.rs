// build.rs - AirsSys WASM Component Build Script
// Validates WIT definitions and generates Rust bindings using wit-bindgen CLI
//
// Build Process:
// 1. Validate WIT with wasm-tools (better error messages)
// 2. Generate Rust bindings with wit-bindgen CLI
// 3. Output to src/generated/ directory
//
// Environment Variables:
// - WASM_TOOLS: Override wasm-tools binary path (default: "wasm-tools")
// - WIT_BINDGEN: Override wit-bindgen binary path (default: "wit-bindgen")
// - AIRSSYS_BUILD_VERBOSE: Enable verbose build output (any value)
//
// Reference: WASM-TASK-003 Phase 3 - Build System Integration

use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    if let Err(e) = run_build() {
        eprintln!("Build failed: {e}");
        std::process::exit(1);
    }
}

fn run_build() -> Result<(), Box<dyn std::error::Error>> {
    // Inform Cargo to re-run build.rs when WIT files or build script changes
    println!("cargo:rerun-if-changed=wit/");
    println!("cargo:rerun-if-changed=build.rs");

    let wit_dir = PathBuf::from("wit");
    let out_dir = PathBuf::from("src").join("generated");

    // Ensure output directory exists
    std::fs::create_dir_all(&out_dir)?;

    // Stage 1: Validate WIT with wasm-tools (better error messages)
    println!("cargo:warning=Validating WIT definitions...");
    validate_wit(&wit_dir)?;

    // Stage 2: Generate Rust bindings with wit-bindgen
    println!("cargo:warning=Generating Rust bindings from WIT...");
    generate_bindings(&wit_dir, &out_dir)?;

    println!("cargo:warning=WIT bindings generated successfully in {}", out_dir.display());
    Ok(())
}

/// Validate WIT definitions using wasm-tools
///
/// Runs `wasm-tools component wit <package_dir>` for each WIT package to validate:
/// - WIT syntax correctness
/// - Package structure validity
/// - Cross-package dependency resolution
/// - Type definition consistency
///
/// Validates packages in dependency order:
/// 1. core (no dependencies)
/// 2. Extension packages (depend on core)
///
/// Returns error if validation fails with clear error messages.
fn validate_wit(wit_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let wasm_tools = env::var("WASM_TOOLS").unwrap_or_else(|_| "wasm-tools".to_string());

    // Validate core package first (all WIT files are in single package now)
    let core_dir = wit_dir.join("core");
    
    println!("cargo:warning=Validating core package...");
    let core_dir_str = core_dir.to_str().ok_or("Invalid UTF-8 in core directory path")?;
    
    let output = Command::new(&wasm_tools)
        .args([
            "component",
            "wit",
            core_dir_str,
        ])
        .output()
        .map_err(|e| format!("Failed to execute wasm-tools. Is it installed? Run: cargo install wasm-tools\nError: {e}"))?;

    if !output.status.success() {
        eprintln!("\n==================== WIT VALIDATION FAILED (core) ====================");
        eprintln!("{}", String::from_utf8_lossy(&output.stderr));
        eprintln!("=======================================================================\n");
        return Err("WIT validation failed for core package. Fix WIT syntax errors and rebuild.".into());
    }

    // Validate extension packages (they depend on core)
    for ext_name in &["filesystem", "network", "process"] {
        let ext_dir = wit_dir.join("ext").join(ext_name);
        
        println!("cargo:warning=Validating {ext_name} extension package...");
        let ext_dir_str = ext_dir.to_str().ok_or_else(|| format!("Invalid UTF-8 in {ext_name} extension directory path"))?;
        
        let output = Command::new(&wasm_tools)
            .args([
                "component",
                "wit",
                ext_dir_str,
            ])
            .output()?;

        if !output.status.success() {
            eprintln!("\n==================== WIT VALIDATION FAILED ({ext_name}) ====================");
            eprintln!("{}", String::from_utf8_lossy(&output.stderr));
            eprintln!("=====================================================================\n");
            return Err(format!("WIT validation failed for {ext_name} extension. Fix WIT syntax errors and rebuild.").into());
        }
    }

    // Print success message
    if env::var("AIRSSYS_BUILD_VERBOSE").is_ok() {
        println!("cargo:warning=All WIT packages validated successfully");
    }
    
    Ok(())
}

/// Generate Rust bindings using wit-bindgen CLI
///
/// Runs `wit-bindgen rust` to generate Rust bindings for the airssys-component world.
///
/// Generated code includes:
/// - Type definitions for all WIT types
/// - Trait definitions for exported interfaces
/// - Import stubs for host services
/// - Module structure matching WIT package organization
///
/// Returns error if generation fails with diagnostic information.
fn generate_bindings(wit_dir: &Path, out_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let wit_bindgen = env::var("WIT_BINDGEN").unwrap_or_else(|_| "wit-bindgen".to_string());

    // World name from wit/core/world.wit
    let world = "airssys-component";
    
    // Point to core package directory where world is defined
    let core_dir = wit_dir.join("core");
    
    let out_dir_str = out_dir.to_str().ok_or("Invalid UTF-8 in output directory path")?;
    let core_dir_str = core_dir.to_str().ok_or("Invalid UTF-8 in core directory path")?;

    let output = Command::new(&wit_bindgen)
        .args([
            "rust",
            "--out-dir", out_dir_str,
            "--world", world,
            "--ownership", "borrowing-duplicate-if-necessary",
            "--format",  // Run rustfmt on generated code
            core_dir_str,
        ])
        .output()
        .map_err(|e| format!("Failed to execute wit-bindgen. Is it installed? Run: cargo install wit-bindgen-cli\nError: {e}"))?;

    if !output.status.success() {
        eprintln!("\n==================== BINDING GENERATION FAILED ====================");
        eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
        eprintln!("===================================================================\n");
        eprintln!("Ensure wit-bindgen is installed:");
        eprintln!("  cargo install wit-bindgen-cli --version 0.47.0");
        eprintln!();
        eprintln!("Required version: 0.47.0");
        eprintln!("World: {world}");
        eprintln!("Package directory: {}", core_dir.display());
        return Err("wit-bindgen failed - see errors above".into());
    }

    // Print generation output if verbose
    if env::var("AIRSSYS_BUILD_VERBOSE").is_ok() {
        println!("cargo:warning=Binding generation output:");
        println!("cargo:warning={}", String::from_utf8_lossy(&output.stdout));
    }
    
    Ok(())
}
