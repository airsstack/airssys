# Build System Troubleshooting Guide

**Version:** 1.0.0  
**For:** airssys-wasm developers

---

## Common Issues and Solutions

### Issue 1: "WIT validation failed"

**Symptoms:**
```
WIT validation failed
expected ';', found '.'
    --> wit/core/types/types.wit:10:20
```

**Cause:** WIT syntax error

**Solution:**
1. Check line number in error message
2. Fix syntax (semicolons, braces, etc.)
3. Validate with: `wasm-tools component wit wit/`
4. Rebuild

### Issue 2: "wit-bindgen: command not found"

**Symptoms:**
```
Failed to execute wit-bindgen
```

**Cause:** wit-bindgen CLI not installed

**Solution:**
```bash
cargo install wit-bindgen-cli --version 0.47.0
```

### Issue 3: "failed to resolve dependency"

**Symptoms:**
```
failed to resolve dependency 'types'
```

**Cause:** Missing or incorrect deps.toml

**Solution:**
1. Check `deps.toml` exists in package directory
2. Verify paths are correct and relative
3. Ensure dependency package exists
4. Quote package names with colons: `"test:types" = { path = "../types" }`

### Issue 4: Generated code doesn't compile

**Symptoms:**
```
error[E0425]: cannot find value `TestResult` in module
```

**Cause:** Type not imported or generated incorrectly

**Solution:**
1. Check WIT `use` statements are correct
2. Verify all types are defined
3. Regenerate bindings: `cargo clean && cargo build`
4. If persistent, report to wit-bindgen issues

### Issue 5: "can't find crate for `core`"

**Symptoms:**
```
error[E0463]: can't find crate for `core`
  |
  = note: the `wasm32-wasip1` target may not be installed
```

**Cause:** Either target not installed OR wit-bindgen macro incompatibility

**Solution:**
```bash
# Install target
rustup target add wasm32-wasip1

# If still fails, use CLI generation (build.rs) not macros
```

### Issue 6: Incremental builds regenerate everything

**Symptoms:** Every build regenerates bindings even when WIT unchanged

**Cause:** Missing `cargo:rerun-if-changed` in build.rs

**Solution:**
Add to build.rs:
```rust
println!("cargo:rerun-if-changed=wit/");
```

### Issue 7: Generated bindings not found

**Symptoms:**
```
error[E0433]: failed to resolve: could not find `generated` in the crate root
```

**Cause:** Generated code not in expected location

**Solution:**
1. Check `src/generated/` exists
2. Verify build.rs ran successfully
3. Add to `src/lib.rs`:
```rust
#[allow(warnings)]
mod generated;
```

### Issue 8: Circular dependency detected

**Symptoms:**
```
error: circular dependency detected: package-a -> package-b -> package-a
```

**Cause:** Package A imports B, B imports A

**Solution:**
1. Extract common types to third package
2. Both packages depend on common package
3. Validate dependency graph is acyclic

---

## Debugging Techniques

### Enable Verbose Build Logging

```bash
export AIRSSYS_BUILD_VERBOSE=1
cargo build
```

### Check Generated Code

```bash
# Generate bindings manually to inspect
wit-bindgen rust --out-dir /tmp/inspect wit/

# Review generated files
ls -la /tmp/inspect
cat /tmp/inspect/lib.rs
```

### Validate WIT Independently

```bash
# Validate entire directory
wasm-tools component wit wit/

# Validate specific package
wasm-tools component wit wit/core/types/
```

### Clean Build

```bash
# Nuclear option - clean everything
cargo clean
rm -rf src/generated
cargo build
```

---

## Performance Issues

### Slow Binding Generation

**Symptom:** build.rs takes >5s

**Solution:**
1. Check if regenerating unnecessarily
2. Verify `cargo:rerun-if-changed` configured
3. Consider caching in CI

### Slow Compilation After Binding Generation

**Symptom:** `cargo build` slow after bindings change

**Cause:** Large generated code forces recompilation

**Solution:**
1. Normal - generated code can be large
2. Use `--release` for production builds
3. Incremental compilation helps (enabled by default)

---

## Platform-Specific Issues

### Windows: Path Issues

**Symptom:** Paths not resolving in deps.toml

**Solution:**
- Use forward slashes even on Windows: `path = "../types"`
- Ensure no backslashes in paths

### macOS: Permission Denied

**Symptom:** Can't create `src/generated/`

**Solution:**
```bash
chmod -R u+w src/
```

### Linux: Tool Not Found in CI

**Symptom:** wit-bindgen not found in GitHub Actions

**Solution:**
Add to workflow:
```yaml
- name: Install wit-bindgen
  run: cargo install wit-bindgen-cli --version 0.47.0
```

---

## Getting Help

### Before Asking

1. Check this troubleshooting guide
2. Review build strategy and integration docs
3. Validate WIT syntax with wasm-tools
4. Try clean build
5. Check tool versions match requirements

### Information to Provide

When reporting issues, include:
- Error message (complete output)
- wit-bindgen version
- wasm-tools version
- Rust version
- Operating system
- Relevant WIT file content
- build.rs content

### Resources

- **wit-bindgen Issues:** https://github.com/bytecodealliance/wit-bindgen/issues
- **Component Model Docs:** https://component-model.bytecodealliance.org/
- **Internal Docs:** `docs/build/` and `docs/src/wit/research/`

---

**Document Status:** ✅ Complete  
**Coverage:** Common issues, debugging, platform-specific problems
