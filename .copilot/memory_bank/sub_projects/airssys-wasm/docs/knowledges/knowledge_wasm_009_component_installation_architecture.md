# KNOWLEDGE-WASM-009: Component Installation Architecture

**Status:** Complete  
**Created:** 2025-10-18  
**Last Updated:** 2025-10-18  
**Related ADRs:** None yet  
**Related Tasks:** None yet  
**Dependencies:** KNOWLEDGE-WASM-001, KNOWLEDGE-WASM-003, KNOWLEDGE-WASM-004

---

## Table of Contents

1. [Overview](#overview)
2. [Installation Philosophy](#installation-philosophy)
3. [Installation Sources](#installation-sources)
4. [Component Manifest Format](#component-manifest-format)
5. [Cryptographic Security Model](#cryptographic-security-model)
6. [Installation Workflows](#installation-workflows)
7. [Update and Uninstall Operations](#update-and-uninstall-operations)
8. [Security Verification](#security-verification)
9. [Build Process Integration](#build-process-integration)
10. [Multi-Component Orchestration](#multi-component-orchestration)
11. [Implementation Architecture](#implementation-architecture)
12. [Future Enhancements](#future-enhancements)

---

## Overview

### Purpose

This document defines the complete component installation architecture for airssys-wasm, covering how plugin developers deploy their components to running host engines. The design is inspired by blockchain smart contract deployment patterns (Solana, NEAR, Ethereum) but adapted for general-purpose component systems.

### Scope

**In Scope:**
- Component installation from multiple sources (Git, local file, remote URL)
- TOML-based manifest format specification
- Cryptographic signing and ownership model
- Update and uninstall authorization
- Build process integration
- Security verification workflows

**Out of Scope (Future Enhancements):**
- Public component registry
- Multi-signature support
- Hardware security module (HSM) integration
- Component dependency resolution
- Automated rollback mechanisms

### Design Philosophy

**Key Principles:**
- **Git Platform Agnostic**: Works with any Git provider (GitHub, GitLab, Bitbucket, self-hosted)
- **Build from Source**: Clone → Build → Deploy workflow ensures reproducibility
- **Cryptographic Ownership**: Only private key holder can install/update/uninstall components
- **Developer-Friendly**: Simple CLI workflow with clear feedback
- **Security First**: Signature verification, hash validation, capability enforcement

---

## Installation Philosophy

### Blockchain-Inspired Ownership Model

Similar to smart contract deployment:
- **Solana**: Developer builds contract → deploys to network → receives Program ID
- **NEAR**: Developer builds contract → deploys to account → contract callable
- **Ethereum**: Developer compiles contract → deploys via transaction → receives address
- **airssys-wasm**: Developer signs component → deploys to host → receives Component ID

**Key Difference:**
- Blockchain: Public deployment with economic costs (gas, rent)
- airssys-wasm: Private/public deployment without blockchain overhead

### Two-Audience Model

**Plugin Developer:**
- Develops WASM component
- Signs component with private key
- Deploys to host runtime

**Host System Administrator:**
- Runs host runtime engine
- Reviews capability requests
- Approves/modifies component permissions
- Monitors component operations

---

## Installation Sources

### Source 1: Git Repository (Recommended for Production)

**Workflow:**
```
Git URL → Clone → Checkout Ref → Build WASM → Sign → Deploy
```

**Characteristics:**
- ✅ **Reproducible**: Git commit = exact source code
- ✅ **Auditable**: Git history provides full audit trail
- ✅ **Traceable**: Know exactly what code is running
- ✅ **Platform Agnostic**: Works with any Git provider
- ⚠️ **Build Time**: Requires local build (slower)
- ⚠️ **Dependencies**: Requires build toolchain installed

**Example:**
```bash
airssys-wasm install \
  --from-git https://github.com/user/my-plugin.git \
  --ref v1.0.0 \
  --host http://localhost:8080
```

**Git Reference Options:**
- `--ref v1.0.0` - Specific tag (recommended for production)
- `--ref main` - Branch (useful for development/staging)
- `--ref abc123def` - Specific commit hash (maximum precision)

### Source 2: Local File (Fast Development Iteration)

**Workflow:**
```
Local WASM Binary + Manifest → Validate → Sign → Deploy
```

**Characteristics:**
- ✅ **Instant**: No clone/build time
- ✅ **Development-Friendly**: Fast iteration during development
- ✅ **Air-Gap Compatible**: Works without internet
- ⚠️ **No Audit Trail**: Must manually track source
- ⚠️ **Manual Build**: Developer responsible for building

**Example:**
```bash
airssys-wasm install \
  --from-file ./target/wasm32-unknown-unknown/release/my_plugin.wasm \
  --manifest ./Component.toml \
  --host http://localhost:8080
```

### Source 3: Remote URL (Pre-Built Distribution)

**Workflow:**
```
URL → Download WASM + Manifest → Verify Checksum → Validate → Deploy
```

**Characteristics:**
- ✅ **Fast**: No build time
- ✅ **CDN Support**: Leverage content delivery networks
- ✅ **Pre-Built**: No build toolchain required
- ⚠️ **Trust Required**: Must trust binary source
- ✅ **Checksum Verification**: Optional integrity verification

**Example:**
```bash
airssys-wasm install \
  --from-url https://releases.company.com/plugins/processor-v1.0.0.wasm \
  --manifest https://releases.company.com/plugins/Component.toml \
  --verify-checksum sha256:abc123... \
  --host http://localhost:8080
```

### Source Comparison Matrix

| Aspect | Git Repository | Local File | Remote URL |
|--------|---------------|------------|------------|
| **Speed** | ⚠️ Slow (build time) | ✅ Instant | ✅ Fast (download) |
| **Reproducibility** | ✅ High (Git commit) | ⚠️ Manual tracking | ⚠️ Binary trust |
| **Audit Trail** | ✅ Git history | ❌ No trail | ⚠️ URL logging |
| **Build Tools** | ⚠️ Required | ❌ Not required | ❌ Not required |
| **Use Case** | Production, CI/CD | Development | Pre-built distribution |
| **Platform Agnostic** | ✅ Yes | ✅ Yes | ✅ Yes |

---

## Component Manifest Format

### File: Component.toml

**Design Decision:** TOML format chosen over JSON for:
- Human readability and writability
- Comment support for documentation
- Rust ecosystem standard (Cargo.toml pattern)
- Less syntactic noise
- Better for configuration files

### Complete Manifest Structure

```toml
# Component.toml - airssys-wasm Component Manifest

# ============================================================================
# Package Information
# ============================================================================
[package]
name = "data-processor"
version = "1.0.0"
description = "High-performance data processor with streaming support"
authors = ["developer@example.com"]
repository = "https://github.com/user/data-processor"
license = "MIT"
readme = "README.md"
homepage = "https://example.com/data-processor"
documentation = "https://docs.example.com/data-processor"
keywords = ["data", "processor", "etl", "streaming"]
categories = ["data-processing", "utilities"]

# ============================================================================
# Build Configuration
# ============================================================================
[build]
# Build system type (rust, go, c, javascript)
type = "rust"

# Target platform for WASM
target = "wasm32-unknown-unknown"

# Path to compiled WASM binary after build (relative to project root)
artifact_path = "target/wasm32-unknown-unknown/release/data_processor.wasm"

# Custom build command (optional, defaults to standard build for type)
build_command = "cargo build --target wasm32-unknown-unknown --release"

# Build profile (release, debug)
profile = "release"

# Additional build flags (optional)
rustflags = ["-C", "opt-level=z", "-C", "strip=symbols"]

# Minimum Rust version required (optional)
rust_version = "1.75.0"

# ============================================================================
# Component Capabilities (Permission Requests)
# ============================================================================
[capabilities]

# File system access capabilities
[capabilities.file_system]
# Paths component can read from (glob patterns supported)
read = [
    "/data/**",           # All files under /data
    "/config/**",         # All config files
    "/input/*.json"       # Specific JSON files in input
]

# Paths component can write to
write = [
    "/output/**",         # All files under /output
    "/logs/**",           # Log files
    "/temp/*.tmp"         # Temporary files
]

# Paths component can execute (if applicable)
execute = []

# Network capabilities
[capabilities.network]
# Outbound connections (host:port format)
outbound = [
    "api.example.com:443",
    "cdn.example.com:443",
    "database.internal:5432"
]

# Inbound connections (ports)
inbound = [8080]

# Allow DNS resolution
dns_lookup = true

# Storage capabilities
[capabilities.storage]
# Maximum storage quota in megabytes
quota_mb = 100

# Use persistent storage (survives component restarts)
persistent = true

# Messaging capabilities (inter-component communication)
[capabilities.messaging]
# Components this can send messages to (list or ["*"] for all)
can_send_to = ["notifier", "logger", "analytics"]

# Components this can receive messages from (["*"] for all)
can_receive_from = ["*"]

# Allow event subscriptions
can_subscribe_events = true

# Custom capabilities (extensible for domain-specific needs)
[[capabilities.custom]]
name = "ml-inference"
value = { model_path = "/models/transformer.onnx", max_batch_size = 32 }

[[capabilities.custom]]
name = "database-access"
value = { connection_string = "postgres://localhost/mydb", read_only = true }

# ============================================================================
# Security Configuration (Cryptographic Ownership)
# ============================================================================
[security]
# Author's Ed25519 public key (64 hex chars = 32 bytes)
# Only holder of corresponding private key can install/update/uninstall
author_public_key = "a1b2c3d4e5f6789abcdef0123456789abcdef0123456789abcdef0123456789a"

# Digital signature of component (128 hex chars = 64 bytes Ed25519 signature)
# Signature covers: wasm_hash + component_name + version + timestamp
signature = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"

# Signature payload (what was signed)
[security.signature_payload]
# SHA256 hash of WASM binary
wasm_hash = "sha256:abc123def456789abc123def456789abc123def456789abc123def456789abc12"

# Component name (binds signature to this component)
component_name = "data-processor"

# Component version (binds signature to this version)
component_version = "1.0.0"

# Timestamp when signed (RFC3339 format)
signed_at = "2025-10-18T10:30:00Z"

# ============================================================================
# Deployment Configuration
# ============================================================================
[deployment]
# Deployment strategy (blue-green, canary, rolling, immediate)
strategy = "blue-green"

# Health check endpoint (for deployment validation)
health_check_endpoint = "/health"

# Minimum instances to run
min_instances = 1

# Maximum instances to scale to
max_instances = 10

# Auto-restart on failure
auto_restart = true

# Deployment timeout in seconds
timeout_seconds = 300

# Grace period for shutdown (seconds)
shutdown_grace_period_seconds = 30

# ============================================================================
# Resource Limits
# ============================================================================
[resources]
# Maximum memory usage in megabytes
max_memory_mb = 512

# Maximum CPU usage percentage (0-100)
max_cpu_percent = 50

# Maximum execution time per invocation (seconds)
max_execution_time_seconds = 30

# Maximum concurrent invocations
max_concurrent_invocations = 100

# ============================================================================
# Component Metadata (Optional)
# ============================================================================
[metadata]
# Component category for discovery
category = "data-processing"

# Tags for searchability
tags = ["etl", "streaming", "high-performance", "real-time"]

# Maturity level (experimental, beta, stable, deprecated)
maturity = "stable"

# Support contact
support_email = "support@example.com"
support_url = "https://support.example.com/data-processor"

# Changelog URL
changelog = "https://github.com/user/data-processor/blob/main/CHANGELOG.md"

# ============================================================================
# Development Configuration (Optional)
# ============================================================================
[development]
# Enable hot reload during development
hot_reload = true

# Enable debug logging
debug_logging = true

# Use mock capabilities (bypass security for testing)
mock_capabilities = false

# Development-only endpoints
dev_endpoints = ["/debug", "/metrics"]
```

### Manifest Validation Rules

```rust
pub struct ManifestValidator;

impl ManifestValidator {
    pub fn validate(manifest: &ComponentManifest) -> Result<(), ValidationError> {
        // Package validation
        Self::validate_package(&manifest.package)?;
        
        // Build validation
        Self::validate_build(&manifest.build)?;
        
        // Capabilities validation
        Self::validate_capabilities(&manifest.capabilities)?;
        
        // Security validation
        Self::validate_security(&manifest.security)?;
        
        Ok(())
    }
    
    fn validate_package(package: &PackageInfo) -> Result<(), ValidationError> {
        // Name: lowercase, alphanumeric with hyphens
        if !package.name.chars().all(|c| c.is_alphanumeric() || c == '-') {
            return Err(ValidationError::InvalidPackageName);
        }
        
        // Version: must be valid semver
        semver::Version::parse(&package.version)?;
        
        // Authors: non-empty
        if package.authors.is_empty() {
            return Err(ValidationError::NoAuthors);
        }
        
        Ok(())
    }
    
    fn validate_build(build: &BuildConfig) -> Result<(), ValidationError> {
        // Artifact path must be valid
        if build.artifact_path.to_str().is_none() {
            return Err(ValidationError::InvalidArtifactPath);
        }
        
        // Target must be WASM
        if build.target != "wasm32-unknown-unknown" 
            && build.target != "wasm32-wasi" {
            return Err(ValidationError::InvalidTarget);
        }
        
        Ok(())
    }
    
    fn validate_capabilities(caps: &CapabilitiesConfig) -> Result<(), ValidationError> {
        // File system paths: must be absolute or valid glob
        for path in &caps.file_system.read {
            Self::validate_path_pattern(path)?;
        }
        for path in &caps.file_system.write {
            Self::validate_path_pattern(path)?;
        }
        
        // Network: validate host:port format
        for endpoint in &caps.network.outbound {
            Self::validate_network_endpoint(endpoint)?;
        }
        
        // Storage quota: reasonable limits
        if caps.storage.quota_mb > 10_000 {
            return Err(ValidationError::StorageQuotaTooLarge);
        }
        
        Ok(())
    }
    
    fn validate_security(security: &SecurityConfig) -> Result<(), ValidationError> {
        // Public key: must be 64 hex chars (32 bytes)
        if security.author_public_key.len() != 64 {
            return Err(ValidationError::InvalidPublicKeyLength);
        }
        hex::decode(&security.author_public_key)
            .map_err(|_| ValidationError::InvalidPublicKeyFormat)?;
        
        // Signature: must be 128 hex chars (64 bytes)
        if security.signature.len() != 128 {
            return Err(ValidationError::InvalidSignatureLength);
        }
        hex::decode(&security.signature)
            .map_err(|_| ValidationError::InvalidSignatureFormat)?;
        
        // Payload: hash must be sha256:...
        if !security.signature_payload.wasm_hash.starts_with("sha256:") {
            return Err(ValidationError::InvalidHashFormat);
        }
        
        Ok(())
    }
}
```

---

## Cryptographic Security Model

### Ed25519 Digital Signatures

**Why Ed25519:**
- Industry standard (used by Solana, NEAR, SSH, Signal)
- Fast signature generation and verification
- 32-byte public keys, 64-byte signatures
- Excellent security properties
- Deterministic signatures (no random number generation)

### Ownership Model

```
Private Key → Author Identity
Public Key → Component Ownership
Signature → Proof of Authorship
```

**Security Guarantee:**
Only the holder of the private key that corresponds to the public key in the manifest can:
- Install the component initially
- Update the component to new versions
- Uninstall the component

### Signature Generation Process

```rust
use ed25519_dalek::{Keypair, Signer, PublicKey, Signature};
use sha2::{Sha256, Digest};

pub struct ComponentSigner {
    keypair: Keypair,
}

impl ComponentSigner {
    /// Generate new keypair
    pub fn generate_keypair() -> Keypair {
        let mut csprng = rand::rngs::OsRng;
        Keypair::generate(&mut csprng)
    }
    
    /// Load keypair from file
    pub fn load_keypair(path: &Path) -> Result<Keypair, KeyError> {
        let content = std::fs::read_to_string(path)?;
        let key_data: KeyData = serde_json::from_str(&content)?;
        
        let secret_bytes = hex::decode(&key_data.secret_key)?;
        let public_bytes = hex::decode(&key_data.public_key)?;
        
        let secret_key = ed25519_dalek::SecretKey::from_bytes(&secret_bytes)?;
        let public_key = PublicKey::from_bytes(&public_bytes)?;
        
        Ok(Keypair { secret: secret_key, public: public_key })
    }
    
    /// Sign component
    pub fn sign_component(
        &self,
        wasm_bytes: &[u8],
        manifest: &mut ComponentManifest,
    ) -> Result<(), SignError> {
        // 1. Calculate WASM hash
        let wasm_hash = Self::hash_wasm(wasm_bytes);
        let wasm_hash_str = format!("sha256:{}", hex::encode(&wasm_hash));
        
        // 2. Create signature payload
        let payload = SignaturePayload {
            wasm_hash: wasm_hash_str.clone(),
            component_name: manifest.package.name.clone(),
            component_version: manifest.package.version.clone(),
            signed_at: chrono::Utc::now().to_rfc3339(),
        };
        
        // 3. Serialize payload deterministically
        let payload_bytes = Self::serialize_payload(&payload);
        
        // 4. Sign with private key
        let signature: Signature = self.keypair.sign(&payload_bytes);
        
        // 5. Update manifest with security config
        manifest.security = SecurityConfig {
            author_public_key: hex::encode(self.keypair.public.as_bytes()),
            signature: hex::encode(signature.to_bytes()),
            signature_payload: payload,
        };
        
        Ok(())
    }
    
    /// Calculate SHA256 hash of WASM binary
    fn hash_wasm(wasm_bytes: &[u8]) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(wasm_bytes);
        hasher.finalize().into()
    }
    
    /// Deterministic payload serialization (CRITICAL for verification)
    fn serialize_payload(payload: &SignaturePayload) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(payload.wasm_hash.as_bytes());
        bytes.extend_from_slice(b"|");
        bytes.extend_from_slice(payload.component_name.as_bytes());
        bytes.extend_from_slice(b"|");
        bytes.extend_from_slice(payload.component_version.as_bytes());
        bytes.extend_from_slice(b"|");
        bytes.extend_from_slice(payload.signed_at.as_bytes());
        bytes
    }
}

/// Keypair file format
#[derive(Serialize, Deserialize)]
struct KeyData {
    public_key: String,   // hex-encoded
    secret_key: String,   // hex-encoded
}
```

### Signature Verification Process

```rust
use ed25519_dalek::{PublicKey, Signature, Verifier};

pub struct ComponentVerifier;

impl ComponentVerifier {
    /// Verify component signature
    pub fn verify_signature(
        wasm_bytes: &[u8],
        manifest: &ComponentManifest,
    ) -> Result<(), VerifyError> {
        let security = &manifest.security;
        
        // 1. Verify WASM hash matches
        let actual_hash = Self::hash_wasm(wasm_bytes);
        let actual_hash_str = format!("sha256:{}", hex::encode(&actual_hash));
        if actual_hash_str != security.signature_payload.wasm_hash {
            return Err(VerifyError::WasmHashMismatch {
                expected: security.signature_payload.wasm_hash.clone(),
                actual: actual_hash_str,
            });
        }
        
        // 2. Verify component name matches
        if security.signature_payload.component_name != manifest.package.name {
            return Err(VerifyError::ComponentNameMismatch);
        }
        
        // 3. Verify version matches
        if security.signature_payload.component_version != manifest.package.version {
            return Err(VerifyError::VersionMismatch);
        }
        
        // 4. Verify timestamp is reasonable
        Self::verify_timestamp(&security.signature_payload.signed_at)?;
        
        // 5. Reconstruct payload (same serialization as signing)
        let payload_bytes = Self::serialize_payload(&security.signature_payload);
        
        // 6. Decode and verify public key
        let public_key_bytes = hex::decode(&security.author_public_key)
            .map_err(|_| VerifyError::InvalidPublicKey)?;
        let public_key = PublicKey::from_bytes(&public_key_bytes)
            .map_err(|_| VerifyError::InvalidPublicKey)?;
        
        // 7. Decode signature
        let signature_bytes = hex::decode(&security.signature)
            .map_err(|_| VerifyError::InvalidSignature)?;
        let signature = Signature::from_bytes(&signature_bytes)
            .map_err(|_| VerifyError::InvalidSignature)?;
        
        // 8. Cryptographic verification
        public_key.verify(&payload_bytes, &signature)
            .map_err(|_| VerifyError::SignatureVerificationFailed)?;
        
        Ok(())
    }
    
    fn verify_timestamp(signed_at: &str) -> Result<(), VerifyError> {
        let signed_time = chrono::DateTime::parse_from_rfc3339(signed_at)
            .map_err(|_| VerifyError::InvalidTimestamp)?;
        let now = chrono::Utc::now();
        
        // Prevent future-dated signatures (>1 year tolerance for clock skew)
        if signed_time > now + chrono::Duration::days(365) {
            return Err(VerifyError::TimestampTooFarInFuture);
        }
        
        // Warn on very old signatures (>5 years)
        if signed_time < now - chrono::Duration::days(365 * 5) {
            tracing::warn!("Component signature is very old (>5 years): {}", signed_at);
        }
        
        Ok(())
    }
    
    fn hash_wasm(wasm_bytes: &[u8]) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(wasm_bytes);
        hasher.finalize().into()
    }
    
    fn serialize_payload(payload: &SignaturePayload) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(payload.wasm_hash.as_bytes());
        bytes.extend_from_slice(b"|");
        bytes.extend_from_slice(payload.component_name.as_bytes());
        bytes.extend_from_slice(b"|");
        bytes.extend_from_slice(payload.component_version.as_bytes());
        bytes.extend_from_slice(b"|");
        bytes.extend_from_slice(payload.signed_at.as_bytes());
        bytes
    }
}
```

---

## Installation Workflows

### Workflow 1: Git Repository Installation

```
┌──────────────────────────────────────────────────────────────┐
│              Git Repository Installation Flow                │
└──────────────────────────────────────────────────────────────┘

1. User Command:
   airssys-wasm install --from-git <url> --ref v1.0.0

2. Clone Repository:
   ├─ Clone to temp directory
   ├─ Checkout tag/branch/commit
   └─ Read Component.toml

3. Build Component:
   ├─ Detect build type (Rust, Go, C++, JS)
   ├─ Execute build command
   ├─ Locate artifact (WASM binary)
   └─ Verify build success

4. Validate Signature:
   ├─ Calculate WASM hash
   ├─ Verify hash matches manifest
   ├─ Reconstruct signature payload
   └─ Verify Ed25519 signature with public key

5. Deploy to Host:
   ├─ Connect to host runtime
   ├─ Upload WASM binary + manifest
   ├─ Host verifies capabilities
   ├─ Host instantiates component
   └─ Return component ID

6. Cleanup:
   └─ Remove temp directory

Success: Component installed with ID comp_abc123
```

### Workflow 2: Local File Installation

```
┌──────────────────────────────────────────────────────────────┐
│              Local File Installation Flow                    │
└──────────────────────────────────────────────────────────────┘

1. User Command:
   airssys-wasm install --from-file ./my_plugin.wasm --manifest Component.toml

2. Load Files:
   ├─ Read WASM binary from local path
   └─ Parse Component.toml

3. Validate Signature:
   ├─ Calculate WASM hash
   ├─ Verify hash matches manifest
   └─ Verify Ed25519 signature

4. Deploy to Host:
   ├─ Connect to host runtime
   ├─ Upload WASM + manifest
   └─ Return component ID

Success: Component installed with ID comp_def456
```

### Workflow 3: Remote URL Installation

```
┌──────────────────────────────────────────────────────────────┐
│              Remote URL Installation Flow                    │
└──────────────────────────────────────────────────────────────┘

1. User Command:
   airssys-wasm install --from-url <wasm_url> --manifest <manifest_url>

2. Download Files:
   ├─ HTTP GET WASM binary
   ├─ HTTP GET Component.toml
   └─ Optional: Verify user-provided checksum

3. Validate Signature:
   ├─ Calculate WASM hash
   ├─ Verify hash matches manifest
   └─ Verify Ed25519 signature

4. Deploy to Host:
   ├─ Connect to host runtime
   ├─ Upload WASM + manifest
   └─ Return component ID

Success: Component installed with ID comp_ghi789
```

---

## Update and Uninstall Operations

### Component Update Workflow

```rust
pub struct ComponentUpdater {
    host_client: HostRuntimeClient,
}

impl ComponentUpdater {
    /// Update existing component
    pub async fn update_component(
        &self,
        component_id: ComponentId,
        new_source: InstallSource,
    ) -> Result<UpdateResult, UpdateError> {
        // 1. Get existing component info
        let existing = self.host_client.get_component(component_id).await?;
        
        // 2. Load new component (from Git/file/URL)
        let (new_wasm, new_manifest) = self.load_component(new_source).await?;
        
        // 3. Verify new component signature
        ComponentVerifier::verify_signature(&new_wasm, &new_manifest)?;
        
        // 4. CRITICAL: Verify same author (public keys must match)
        if new_manifest.security.author_public_key 
            != existing.manifest.security.author_public_key {
            return Err(UpdateError::AuthorMismatch {
                existing: existing.manifest.security.author_public_key,
                new: new_manifest.security.author_public_key,
            });
        }
        
        // 5. Validate version is newer
        let existing_version = semver::Version::parse(&existing.manifest.package.version)?;
        let new_version = semver::Version::parse(&new_manifest.package.version)?;
        if new_version <= existing_version {
            return Err(UpdateError::VersionNotNewer {
                existing: existing_version,
                new: new_version,
            });
        }
        
        // 6. Deploy update with strategy
        let strategy = new_manifest.deployment.strategy.clone();
        let result = self.host_client
            .update_component(component_id, new_wasm, new_manifest, strategy)
            .await?;
        
        Ok(result)
    }
}
```

**Update Command:**
```bash
# Update from Git (new version)
airssys-wasm update comp_abc123 \
  --from-git https://github.com/user/my-plugin.git \
  --ref v1.1.0

# Update from local build
airssys-wasm update comp_abc123 \
  --from-file ./target/release/my_plugin.wasm \
  --manifest Component.toml
```

### Component Uninstall Workflow

```rust
pub struct ComponentUninstaller {
    host_client: HostRuntimeClient,
}

impl ComponentUninstaller {
    /// Uninstall component (requires proof of ownership)
    pub async fn uninstall_component(
        &self,
        component_id: ComponentId,
        keypair: &Keypair,
    ) -> Result<(), UninstallError> {
        // 1. Get component info
        let component = self.host_client.get_component(component_id).await?;
        
        // 2. Verify keypair matches component author
        let public_key_hex = hex::encode(keypair.public.as_bytes());
        if public_key_hex != component.manifest.security.author_public_key {
            return Err(UninstallError::KeypairMismatch);
        }
        
        // 3. Generate uninstall proof
        let proof = Self::generate_uninstall_proof(component_id, keypair)?;
        
        // 4. Send uninstall request with proof
        self.host_client.uninstall_component(component_id, proof).await?;
        
        Ok(())
    }
    
    fn generate_uninstall_proof(
        component_id: ComponentId,
        keypair: &Keypair,
    ) -> Result<UninstallProof, UninstallError> {
        // Create payload: "uninstall:<component_id>:<timestamp>"
        let timestamp = chrono::Utc::now().to_rfc3339();
        let payload = format!("uninstall:{}:{}", component_id, timestamp);
        
        // Sign payload
        let signature = keypair.sign(payload.as_bytes());
        
        Ok(UninstallProof {
            timestamp,
            signature: hex::encode(signature.to_bytes()),
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UninstallProof {
    pub timestamp: String,
    pub signature: String,
}
```

**Uninstall Command:**
```bash
# Uninstall component (requires private key)
airssys-wasm uninstall comp_abc123 \
  --keypair ~/.airssys/keypair.json
```

---

## Security Verification

### Multi-Layer Verification

```
┌─────────────────────────────────────────────────────────────┐
│           Security Verification Layers                      │
├─────────────────────────────────────────────────────────────┤
│ Layer 1: Manifest Validation                               │
│   ├─ TOML syntax correct                                   │
│   ├─ Required fields present                               │
│   ├─ Field formats valid                                   │
│   └─ Semver version valid                                  │
├─────────────────────────────────────────────────────────────┤
│ Layer 2: WASM Binary Validation                            │
│   ├─ Valid WebAssembly format                              │
│   ├─ Component Model compliance                            │
│   ├─ Size within limits                                    │
│   └─ No malformed sections                                 │
├─────────────────────────────────────────────────────────────┤
│ Layer 3: Cryptographic Verification                        │
│   ├─ Public key format valid (64 hex chars)                │
│   ├─ Signature format valid (128 hex chars)                │
│   ├─ WASM hash matches signed hash                         │
│   ├─ Component name matches signed name                    │
│   ├─ Version matches signed version                        │
│   ├─ Timestamp reasonable                                  │
│   └─ Ed25519 signature cryptographically valid             │
├─────────────────────────────────────────────────────────────┤
│ Layer 4: Capability Validation                             │
│   ├─ Requested capabilities reasonable                     │
│   ├─ Path patterns valid                                   │
│   ├─ Network endpoints valid                               │
│   └─ Resource quotas within limits                         │
├─────────────────────────────────────────────────────────────┤
│ Layer 5: Host Security Policy                              │
│   ├─ Host reviews capability requests                      │
│   ├─ Host approves/modifies capabilities                   │
│   ├─ Host enforces runtime policies                        │
│   └─ Host monitors component behavior                      │
└─────────────────────────────────────────────────────────────┘
```

### Security Error Handling

```rust
#[derive(Debug, thiserror::Error)]
pub enum SecurityError {
    #[error("WASM hash mismatch: expected {expected}, got {actual}")]
    WasmHashMismatch { expected: String, actual: String },
    
    #[error("Component name mismatch: manifest says {manifest}, signature says {signature}")]
    ComponentNameMismatch { manifest: String, signature: String },
    
    #[error("Version mismatch: manifest says {manifest}, signature says {signature}")]
    VersionMismatch { manifest: String, signature: String },
    
    #[error("Invalid public key format")]
    InvalidPublicKey,
    
    #[error("Invalid signature format")]
    InvalidSignature,
    
    #[error("Signature verification failed - component not signed by claimed author")]
    SignatureVerificationFailed,
    
    #[error("Timestamp too far in future: {0}")]
    TimestampTooFarInFuture(String),
    
    #[error("Author mismatch on update: existing {existing}, new {new}")]
    AuthorMismatch { existing: String, new: String },
    
    #[error("Keypair does not match component author")]
    KeypairMismatch,
}
```

---

## Build Process Integration

### Language-Specific Build Implementations

```rust
pub trait ComponentBuilder: Send + Sync {
    fn can_build(&self, build_config: &BuildConfig) -> bool;
    async fn build(&self, project_dir: &Path, build_config: &BuildConfig) 
        -> Result<Vec<u8>, BuildError>;
}

/// Rust component builder
pub struct RustBuilder;

#[async_trait::async_trait]
impl ComponentBuilder for RustBuilder {
    fn can_build(&self, config: &BuildConfig) -> bool {
        config.build_type == "rust"
    }
    
    async fn build(&self, project_dir: &Path, config: &BuildConfig) 
        -> Result<Vec<u8>, BuildError> {
        // Verify Rust toolchain installed
        Self::verify_toolchain()?;
        
        // Build command (default or custom)
        let build_cmd = config.build_command.clone().unwrap_or_else(|| {
            format!("cargo build --target {} --{}", 
                config.target, config.profile)
        });
        
        // Execute build
        let output = tokio::process::Command::new("sh")
            .arg("-c")
            .arg(&build_cmd)
            .current_dir(project_dir)
            .output()
            .await?;
        
        if !output.status.success() {
            return Err(BuildError::BuildFailed {
                stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            });
        }
        
        // Read artifact
        let artifact_path = project_dir.join(&config.artifact_path);
        let wasm_bytes = tokio::fs::read(&artifact_path).await?;
        
        Ok(wasm_bytes)
    }
    
    fn verify_toolchain() -> Result<(), BuildError> {
        // Check cargo installed
        let cargo_check = std::process::Command::new("cargo")
            .arg("--version")
            .output();
        
        if cargo_check.is_err() {
            return Err(BuildError::ToolchainNotFound {
                tool: "cargo".to_string(),
                install_hint: "Install Rust: https://rustup.rs".to_string(),
            });
        }
        
        // Check wasm32-unknown-unknown target installed
        let target_check = std::process::Command::new("rustup")
            .args(&["target", "list", "--installed"])
            .output()?;
        
        let targets = String::from_utf8_lossy(&target_check.stdout);
        if !targets.contains("wasm32-unknown-unknown") {
            return Err(BuildError::ToolchainNotFound {
                tool: "wasm32-unknown-unknown".to_string(),
                install_hint: "Run: rustup target add wasm32-unknown-unknown".to_string(),
            });
        }
        
        Ok(())
    }
}

/// Multi-language build coordinator
pub struct BuildEngine {
    builders: Vec<Box<dyn ComponentBuilder>>,
}

impl BuildEngine {
    pub fn new() -> Self {
        Self {
            builders: vec![
                Box::new(RustBuilder),
                // Future: GoBuilder, CppBuilder, JsBuilder
            ],
        }
    }
    
    pub async fn build_component(
        &self,
        project_dir: &Path,
        manifest: &ComponentManifest,
    ) -> Result<Vec<u8>, BuildError> {
        // Find builder for this build type
        let builder = self.builders.iter()
            .find(|b| b.can_build(&manifest.build))
            .ok_or_else(|| BuildError::NoBuilderFound {
                build_type: manifest.build.build_type.clone(),
            })?;
        
        // Execute build
        builder.build(project_dir, &manifest.build).await
    }
}
```

---

## Multi-Component Orchestration

### Componentfile Format

```yaml
# componentfile.yml - Multi-component orchestration

version: "1.0"

# Component definitions
components:
  # Component 1: Data processor
  data-processor:
    # Installation source
    source:
      type: git
      url: https://github.com/company/data-processor
      ref: v2.1.0
    
    # Capability overrides (optional)
    capabilities:
      file_system:
        read: ["/data/**"]
        write: ["/processed/**"]
      storage:
        quota_mb: 500
  
  # Component 2: API gateway
  api-gateway:
    source:
      type: git
      url: https://github.com/company/api-gateway
      ref: main
    
    capabilities:
      network:
        outbound: ["*:443"]
        inbound: [8080]
      storage:
        quota_mb: 100
  
  # Component 3: Private notifier
  notifier:
    source:
      type: git
      url: git@github.com:company/private-notifier.git
      ref: v1.5.0
      auth:
        type: ssh-key
        key_path: ~/.ssh/id_ed25519
    
    capabilities:
      network:
        outbound: ["smtp.gmail.com:587"]

# Global deployment settings
deployment:
  # Target host runtime
  host: https://runtime.company.com
  
  # Default deployment strategy
  strategy: blue-green
  
  # Parallel deployment
  parallel: true
  
  # Continue on individual component failure
  continue_on_error: false

# Component dependencies (optional)
dependencies:
  # api-gateway depends on data-processor
  api-gateway:
    requires: [data-processor]
  
  # notifier depends on both
  notifier:
    requires: [data-processor, api-gateway]
```

### Multi-Component Installation

```bash
# Install all components from componentfile
airssys-wasm install --file componentfile.yml

# Output:
# Reading componentfile.yml...
# Found 3 components to install
# 
# [1/3] Installing data-processor@v2.1.0...
#   ├─ Cloning from git...
#   ├─ Building component...
#   ├─ Verifying signature...
#   └─ Deploying... ✓ (comp_abc123)
# 
# [2/3] Installing api-gateway@main...
#   ├─ Cloning from git...
#   ├─ Building component...
#   ├─ Verifying signature...
#   └─ Deploying... ✓ (comp_def456)
# 
# [3/3] Installing notifier@v1.5.0...
#   ├─ Cloning from git (with SSH key)...
#   ├─ Building component...
#   ├─ Verifying signature...
#   └─ Deploying... ✓ (comp_ghi789)
# 
# ✓ All 3 components installed successfully!
```

---

## Implementation Architecture

### Core Installation Components

```rust
/// Main component installer
pub struct ComponentInstaller {
    git_client: GitClient,
    http_client: HttpClient,
    build_engine: BuildEngine,
    host_client: HostRuntimeClient,
    signer: ComponentSigner,
    verifier: ComponentVerifier,
}

impl ComponentInstaller {
    /// Install component from any source
    pub async fn install(
        &self,
        source: InstallSource,
        options: InstallOptions,
    ) -> Result<ComponentId, InstallError> {
        // 1. Load component (WASM + manifest) based on source
        let (wasm_bytes, mut manifest) = match source {
            InstallSource::Git { url, git_ref, auth } => {
                self.install_from_git(&url, git_ref, auth).await?
            }
            InstallSource::LocalFile { wasm_path, manifest_path } => {
                self.install_from_local(&wasm_path, &manifest_path).await?
            }
            InstallSource::RemoteUrl { wasm_url, manifest_url, verify_checksum } => {
                self.install_from_url(&wasm_url, &manifest_url, verify_checksum).await?
            }
        };
        
        // 2. Validate manifest
        ManifestValidator::validate(&manifest)?;
        
        // 3. Validate WASM binary
        WasmValidator::validate(&wasm_bytes)?;
        
        // 4. Verify cryptographic signature
        ComponentVerifier::verify_signature(&wasm_bytes, &manifest)?;
        
        // 5. Merge capability overrides (if provided)
        if let Some(override_caps) = options.capability_overrides {
            manifest.capabilities = override_caps;
        }
        
        // 6. Deploy to host runtime
        let component_id = self.host_client
            .deploy_component(DeploymentRequest {
                wasm_bytes,
                manifest,
                deployment_strategy: options.deployment_strategy,
            })
            .await?;
        
        Ok(component_id)
    }
    
    /// Install from Git repository
    async fn install_from_git(
        &self,
        url: &str,
        git_ref: GitRef,
        auth: Option<GitAuth>,
    ) -> Result<(Vec<u8>, ComponentManifest), InstallError> {
        // Clone repository
        let temp_dir = self.git_client.clone_repository(url, auth).await?;
        
        // Checkout ref
        self.git_client.checkout(&temp_dir, git_ref).await?;
        
        // Read manifest
        let manifest_path = temp_dir.join("Component.toml");
        let manifest = ComponentManifest::from_toml_file(&manifest_path)?;
        
        // Build component
        let wasm_bytes = self.build_engine
            .build_component(&temp_dir, &manifest)
            .await?;
        
        // Cleanup
        tokio::fs::remove_dir_all(&temp_dir).await?;
        
        Ok((wasm_bytes, manifest))
    }
    
    /// Install from local files
    async fn install_from_local(
        &self,
        wasm_path: &Path,
        manifest_path: &Path,
    ) -> Result<(Vec<u8>, ComponentManifest), InstallError> {
        let wasm_bytes = tokio::fs::read(wasm_path).await?;
        let manifest = ComponentManifest::from_toml_file(manifest_path)?;
        Ok((wasm_bytes, manifest))
    }
    
    /// Install from remote URLs
    async fn install_from_url(
        &self,
        wasm_url: &str,
        manifest_url: &str,
        verify_checksum: Option<String>,
    ) -> Result<(Vec<u8>, ComponentManifest), InstallError> {
        // Download WASM binary
        let wasm_bytes = self.http_client.download_bytes(wasm_url).await?;
        
        // Verify checksum if provided
        if let Some(expected) = verify_checksum {
            let actual = Self::calculate_checksum(&wasm_bytes);
            if actual != expected {
                return Err(InstallError::ChecksumMismatch { expected, actual });
            }
        }
        
        // Download and parse manifest
        let manifest_content = self.http_client.download_string(manifest_url).await?;
        let manifest = ComponentManifest::from_toml_str(&manifest_content)?;
        
        Ok((wasm_bytes, manifest))
    }
    
    fn calculate_checksum(data: &[u8]) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(data);
        format!("sha256:{}", hex::encode(hasher.finalize()))
    }
}
```

---

## Future Enhancements

### Phase 2: Enhanced Security (Post-MVP)

**1. Multi-Signature Support**
- Require N-of-M signatures for critical components
- Team ownership with multiple authorized keys
- Security councils for high-value components

**2. Key Rotation Mechanism**
- Rotate keys without losing component ownership
- Gradual migration from old to new keys
- Revocation of compromised keys

**3. Hardware Security Module (HSM) Integration**
- YubiKey support for signing
- Ledger hardware wallet integration
- Cloud HSM for enterprise deployments

**4. Revocation Lists**
- Centralized/decentralized revocation checking
- Automatic blocking of compromised components
- Real-time security updates

### Phase 3: Advanced Features (Future)

**1. Component Dependency Resolution**
- Automatically install component dependencies
- Version conflict resolution
- Dependency graph visualization

**2. Build Caching**
- Cache built artifacts by Git commit
- Faster repeated installations
- Shared build cache across team

**3. Rollback Automation**
- Automatic rollback on deployment failure
- Health check integration
- Traffic-based rollback triggers

**4. Transparency Logs**
- Public audit log of all component operations
- Certificate Transparency model
- Community security monitoring

---

## References

### Related Knowledge Documents
- KNOWLEDGE-WASM-001: Component Framework Architecture
- KNOWLEDGE-WASM-003: Core Architecture Design
- KNOWLEDGE-WASM-004: WIT Management Architecture
- KNOWLEDGE-WASM-010: CLI Tool Specification (airssys-wasm-cli)

### External Standards
- Ed25519: RFC 8032 (Edwards-Curve Digital Signature Algorithm)
- TOML: Tom's Obvious, Minimal Language (v1.0.0)
- Semver: Semantic Versioning 2.0.0
- WebAssembly Component Model: W3C Community Group

### Blockchain References
- Solana Program Deployment: https://docs.solana.com/cli/deploy-a-program
- NEAR Contract Deployment: https://docs.near.org/develop/deploy
- Ethereum Contract Deployment: https://docs.ethers.org/v5/api/contract/contract-factory/

---

**Document Status:** Complete - Ready for implementation (Phase 1: Basic installation with cryptographic signing)

**Next Steps:**
1. Implement CLI tool (airssys-wasm-cli) based on this architecture
2. Create reference implementation of Git-based installer
3. Develop comprehensive test suite for security verification
4. Document deployment best practices and security guidelines
