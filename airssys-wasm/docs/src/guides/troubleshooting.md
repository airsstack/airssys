# Troubleshooting Guide

**Document Type:** How-To Guide (Diátaxis)  
**Audience:** Developers encountering build or runtime issues  
**Last Updated:** 2025-11-29

## Overview

This guide provides solutions to common issues encountered when building and using airssys-wasm.

## Build Issues

### Error: "wasm-tools: command not found"

**Symptom:**
```
error: failed to execute command: "wasm-tools"
No such file or directory (os error 2)
```

**Cause:** `wasm-tools` is not installed or not in PATH.

**Solution:**

1. Install wasm-tools version 1.240.0:
   ```bash
   cargo install wasm-tools --version 1.240.0
   ```

2. Verify installation:
   ```bash
   wasm-tools --version
   # Expected: wasm-tools 1.240.0
   ```

3. If still not found, check your PATH:
   ```bash
   echo $PATH | grep cargo
   # Should show ~/.cargo/bin
   ```

4. Add cargo bin to PATH if missing:
   ```bash
   # Add to ~/.bashrc or ~/.zshrc
   export PATH="$HOME/.cargo/bin:$PATH"
   source ~/.bashrc  # or ~/.zshrc
   ```

---

### Error: "wit-bindgen: command not found"

**Symptom:**
```
error: failed to execute command: "wit-bindgen"
No such file or directory (os error 2)
```

**Cause:** `wit-bindgen` is not installed or not in PATH.

**Solution:**

1. Install wit-bindgen version 0.47.0:
   ```bash
   cargo install wit-bindgen-cli --version 0.47.0
   ```

2. Verify installation:
   ```bash
   wit-bindgen --version
   # Expected: wit-bindgen-cli 0.47.0
   ```

3. If issues persist, try reinstalling:
   ```bash
   cargo uninstall wit-bindgen-cli
   cargo install wit-bindgen-cli --version 0.47.0
   ```

---

### Error: "WIT validation failed"

**Symptom:**
```
error: WIT validation failed
  --> wit/core/types.wit:15:10
   |
15 |     use unknown.{type-name}
   |         ^^^^^^^ unresolved package
```

**Causes:**
- WIT syntax errors
- Invalid type references
- Missing package dependencies
- Circular dependencies

**Solution:**

1. **Check WIT syntax manually:**
   ```bash
   wasm-tools component wit wit/core
   ```

2. **Common syntax issues:**
   - Missing semicolons after type definitions
   - Invalid type names (must be kebab-case)
   - Incorrect `use` statement syntax

3. **Verify type references:**
   - Ensure all `use` statements reference defined types
   - Check that imported packages exist in `deps.toml`

4. **Check for circular dependencies:**
   ```bash
   # Validate each package independently
   wasm-tools component wit wit/core
   wasm-tools component wit wit/ext/filesystem
   wasm-tools component wit wit/ext/network
   wasm-tools component wit wit/ext/process
   ```

**Example Fix:**

Before (incorrect):
```wit
// Missing semicolon
record config {
    value: string
}

// Invalid type reference
use unknown-package.{type-name}
```

After (correct):
```wit
// Semicolon added
record config {
    value: string,
}

// Valid type reference (package exists)
use airssys:core.{component-id}
```

---

### Error: "Binding generation failed"

**Symptom:**
```
error: wit-bindgen failed
wit-bindgen exited with status code 1
```

**Causes:**
- Missing world definition
- Invalid interface exports/imports
- wit-bindgen version mismatch

**Solution:**

1. **Verify world definition exists:**
   ```bash
   # Check that wit/core/world.wit exists
   ls wit/core/world.wit
   ```

2. **Check world syntax:**
   ```wit
   // wit/core/world.wit
   package airssys:core@1.0.0;

   world airssys-component {
       // All exported interfaces must be defined
       export component;
       export lifecycle;
       
       // All imported interfaces must be defined
       import host-services;
   }
   ```

3. **Verify wit-bindgen version:**
   ```bash
   wit-bindgen --version
   # Must be 0.47.0
   ```

4. **Try manual binding generation:**
   ```bash
   wit-bindgen rust wit/core --out-dir src/generated/
   ```

5. **Check generated output:**
   ```bash
   # Generated file should exist
   ls src/generated/airssys_component.rs
   ```

---

### Build Performance Issues

**Symptom:** Build takes longer than expected (> 30 seconds for incremental builds).

**Possible Causes:**
- WIT files being regenerated unnecessarily
- Large number of dependencies
- Slow disk I/O

**Solution:**

1. **Check if WIT regeneration is happening:**
   ```bash
   AIRSSYS_BUILD_VERBOSE=1 cargo build
   # Look for "WIT files changed, regenerating bindings"
   ```

2. **Verify incremental compilation is enabled:**
   ```bash
   # Check Cargo.toml
   [profile.dev]
   incremental = true
   ```

3. **Use faster linker (optional):**
   ```toml
   # .cargo/config.toml
   [target.x86_64-unknown-linux-gnu]
   linker = "clang"
   rustflags = ["-C", "link-arg=-fuse-ld=lld"]
   ```

4. **Clean build if corruption suspected:**
   ```bash
   cargo clean && cargo build
   ```

---

## Runtime Issues

### Error: "Component validation failed"

**Symptom:**
```rust
Error: ComponentValidationFailed("invalid component")
```

**Causes:**
- Component not compiled with WebAssembly Component Model
- Incorrect WASM target
- Missing WIT interfaces

**Solution:**

1. **Verify component was built with correct target:**
   ```bash
   # Component must target wasm32-wasip1
   cargo build --target wasm32-wasip1
   ```

2. **Check component has required exports:**
   ```bash
   wasm-tools component wit component.wasm
   ```

3. **Validate component structure:**
   ```bash
   wasm-tools validate component.wasm
   ```

---

### Error: "Permission denied" at Runtime

**Symptom:**
```rust
Error: CapabilityDenied("file:read", "/path/to/file")
```

**Cause:** Component lacks required capability in Component.toml.

**Solution:**

1. **Check Component.toml permissions:**
   ```toml
   [permissions.filesystem]
   readable_paths = [
       "/path/to/file",     # Exact path
       "/data/**",          # Glob pattern
   ]
   ```

2. **Verify pattern matching:**
   - Patterns are case-sensitive
   - Use `**` for recursive directory matching
   - Use `*` for single-level wildcard

3. **Check security mode:**
   ```rust
   // Development mode allows all (insecure)
   SecurityConfig {
       mode: SecurityMode::Development,
       ..Default::default()
   }
   ```

---

### Error: "Memory limit exceeded"

**Symptom:**
```rust
Error: ResourceLimitExceeded("memory", 2097152, 1048576)
```

**Cause:** Component exceeded configured memory limit.

**Solution:**

1. **Increase memory limit in Component.toml:**
   ```toml
   [resources.memory]
   max_memory_bytes = 4194304  # 4MB (increase from 2MB)
   ```

2. **Valid memory range:** 512KB - 4MB

3. **Check component memory usage:**
   ```rust
   // Monitor memory usage
   let metrics = runtime.get_metrics(component_id)?;
   println!("Memory used: {}", metrics.memory_used);
   ```

---

## Testing Issues

### Tests Fail with "Component load error"

**Symptom:**
```
test result: FAILED. 200 passed; 50 failed
```

**Cause:** Test fixtures not built or outdated.

**Solution:**

1. **Rebuild test fixtures:**
   ```bash
   cd tests/fixtures
   ./build.sh
   ```

2. **Verify fixtures exist:**
   ```bash
   ls tests/fixtures/*.wasm
   ```

3. **Check fixture WIT compatibility:**
   ```bash
   wasm-tools component wit tests/fixtures/hello_world.wasm
   ```

---

### Error: "No such file or directory" in Tests

**Symptom:**
```
Error: Os { code: 2, kind: NotFound, message: "No such file or directory" }
```

**Cause:** Test is looking for files relative to wrong working directory.

**Solution:**

1. **Use absolute paths in tests:**
   ```rust
   let fixture = env!("CARGO_MANIFEST_DIR")
       .to_string() + "/tests/fixtures/hello_world.wasm";
   ```

2. **Or use workspace-relative paths:**
   ```rust
   let workspace = env!("CARGO_WORKSPACE_DIR");
   let fixture = format!("{}/airssys-wasm/tests/fixtures/hello_world.wasm", workspace);
   ```

---

## Documentation Issues

### Error: "cargo doc" fails

**Symptom:**
```
error: unresolved link to `ComponentError`
```

**Cause:** Broken rustdoc links.

**Solution:**

1. **Check link syntax:**
   ```rust
   // Incorrect
   /// See [`ComponentError`] for details.
   
   // Correct
   /// See [`crate::core::error::ComponentError`] for details.
   ```

2. **Generate docs with all features:**
   ```bash
   cargo doc --all-features
   ```

---

## Platform-Specific Issues

### macOS: "xcrun: error: invalid active developer path"

**Symptom:**
```
xcrun: error: invalid active developer path
```

**Cause:** Xcode command-line tools not installed.

**Solution:**
```bash
xcode-select --install
```

---

### Windows: "link.exe not found"

**Symptom:**
```
error: linker `link.exe` not found
```

**Cause:** Visual Studio Build Tools not installed.

**Solution:**

1. Install Visual Studio Build Tools:
   - Download from https://visualstudio.microsoft.com/downloads/
   - Select "Desktop development with C++"

2. Or use GNU toolchain:
   ```bash
   rustup target add x86_64-pc-windows-gnu
   ```

---

### Linux: "cannot find -lssl"

**Symptom:**
```
error: linking with `cc` failed
/usr/bin/ld: cannot find -lssl
```

**Cause:** OpenSSL development libraries not installed.

**Solution:**

Debian/Ubuntu:
```bash
sudo apt-get install libssl-dev pkg-config
```

Fedora/RHEL:
```bash
sudo dnf install openssl-devel
```

Arch Linux:
```bash
sudo pacman -S openssl pkg-config
```

---

## Getting Help

If your issue isn't covered here:

1. **Check existing issues:** [GitHub Issues](https://github.com/airsstack/airssys/issues)
2. **Search discussions:** [GitHub Discussions](https://github.com/airsstack/airssys/discussions)
3. **Create new issue:** Include:
   - Error message (full output)
   - Build environment (OS, Rust version, tool versions)
   - Steps to reproduce
   - Relevant code snippets

## Diagnostic Information

When reporting issues, include this diagnostic output:

```bash
# System information
uname -a
rustc --version
cargo --version

# Tool versions
wasm-tools --version
wit-bindgen --version

# Build output (verbose)
AIRSSYS_BUILD_VERBOSE=1 cargo build 2>&1 | tee build.log

# Test output
cargo test -- --nocapture 2>&1 | tee test.log
```

## Summary

This guide covered:
- ✅ Common build errors and solutions
- ✅ Runtime permission and resource issues
- ✅ Testing and documentation problems
- ✅ Platform-specific troubleshooting
- ✅ How to get help and report issues

For architecture questions, see [Architecture Overview](../architecture/overview.md).  
For component development, see [Component Development Guide](component-development.md).
