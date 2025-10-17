# Documentation Terminology Standards

**Document Type:** Workspace Standard  
**Created:** 2025-10-17  
**Status:** Active - Mandatory for All Documentation  
**Scope:** All AirsSys sub-projects and documentation

## Purpose

This document establishes mandatory terminology standards to ensure all AirsSys documentation maintains professional, technical, and honest language without hyperbole or marketing-speak.

## Core Principles

### ✅ Professional Technical Documentation
- Use precise, measurable, technical terminology
- State capabilities factually without exaggeration
- Provide evidence for all performance claims
- Use industry-standard terminology where applicable

### ❌ Avoid Marketing Hyperbole
- No superlatives without quantifiable evidence
- No absolute claims that cannot be proven
- No buzzwords without clear technical meaning
- No excessive emphasis or promotional language

---

## Terminology Standards

### Approved Taglines by Sub-Project

#### airssys-wasm
**Approved:** "WASM Component Framework for Pluggable Systems"

**Rejected Terms:**
- ~~"Universal Hot-Deployable WASM Component Framework"~~
- ~~"Revolutionary WASM Framework"~~
- ~~"CosmWasm for Everything"~~

#### airssys-osl
**Approved:** "OS Layer Framework with Security and Activity Logging"

#### airssys-rt
**Approved:** "Lightweight Actor Model Runtime System"

---

## Forbidden Terms (Never Use)

### Absolute Superlatives
❌ **NEVER USE:**
- Revolutionary
- Game-changing
- Industry-leading
- Best-in-class
- Cutting-edge
- Next-generation (without context)
- World-class
- State-of-the-art
- Groundbreaking
- Paradigm-shifting

### Hyperbolic Performance Claims
❌ **NEVER USE:**
- Blazingly fast
- Lightning fast
- Instant (unless < 100ms)
- Zero cost (unless actually free)
- Infinite scalability
- Unlimited
- Perfect

### Vague Buzzwords
❌ **AVOID WITHOUT DEFINITION:**
- Smart
- Intelligent
- Advanced
- Modern (prefer specific: "2024 specification")
- Powerful
- Robust (prefer specific: "fault-tolerant")
- Enterprise-grade
- Production-ready (unless tested)

### Self-Promotional Claims
❌ **NEVER USE (Self-Claims):**
- Our framework is...
- We are the best/first/only...
- We provide superior...
- We outperform...
- We revolutionize...
- This is better than...
- Our approach is more advanced...
- We excel at...

**Why:** These are subjective self-assessments that undermine credibility.

✅ **USE INSTEAD:**
- Describe capabilities objectively
- Provide comparative data with measurements
- Let features speak for themselves
- Use third-person technical description

### Absolute Universal Claims
❌ **NEVER USE:**
- Universal (prefer: cross-platform, general-purpose, domain-agnostic)
- Everything/Anything
- All/Every (unless literally true)
- Always/Never (rare technical exceptions only)
- Complete/Total (unless 100% coverage proven)

---

## Replacement Guidelines

### Deployment Terminology

#### ❌ AVOID: "Hot-Deployable" / "Hot Deployment"
**Why:** Buzzword without clear meaning; overused in marketing

#### ✅ USE INSTEAD:
- **Runtime deployment** - Components loaded during runtime
- **Dynamic component loading** - Components can be loaded/unloaded dynamically
- **Live updates** - Updates applied while system is running
- **Runtime component management** - Component lifecycle managed at runtime

**Example:**
```markdown
❌ Hot deployment enables zero-downtime updates
✅ Components can be loaded and updated during runtime without system restart
```

---

### Scope Terminology

#### ❌ AVOID: "Universal"
**Why:** Implies works for everything; unprovable claim

#### ✅ USE INSTEAD:
- **Cross-platform** (if supports multiple OS)
- **General-purpose** (if not domain-specific)
- **Domain-agnostic** (if works across multiple domains)
- **Multi-purpose** (if has multiple use cases)
- **Language-agnostic** (if supports multiple languages)

**Example:**
```markdown
❌ Universal component framework for all use cases
✅ Cross-platform component framework for general-purpose applications
```

---

### Availability Terminology

#### ❌ AVOID: "Zero-Downtime"
**Why:** Absolute claim; hard to guarantee; marketing buzzword

#### ✅ USE INSTEAD:
- **Runtime updates without system restart**
- **Updates applied while system continues running**
- **Minimal service interruption**
- **Live component updates**

**Example:**
```markdown
❌ Zero-downtime deployment in milliseconds
✅ Components can be updated during runtime without restarting the host system
```

---

### Innovation Terminology

#### ❌ AVOID: "Revolutionary" / "Game-Changing"
**Why:** Marketing hyperbole; subjective claims

#### ✅ USE INSTEAD:
- **Novel approach** (if actually new)
- **Enables [specific capability]**
- **Provides [specific benefit]**
- **Different from existing solutions by [specific difference]**
- Remove entirely and describe the capability

**Example:**
```markdown
❌ Revolutionary framework that changes everything
✅ Framework that enables runtime component deployment, differing from traditional plugin systems that require restarts
```

---

### Comparison Terminology

#### ❌ AVOID: "CosmWasm for Everything" / "X for All Y"
**Why:** Hyperbolic comparison; overpromising

#### ✅ USE INSTEAD:
- **CosmWasm-inspired approach** (acknowledges inspiration)
- **Similar to CosmWasm but for general computing** (specific comparison)
- **Applies smart contract deployment patterns to [specific domain]**
- Remove comparison and describe capability directly

**Example:**
```markdown
❌ This is CosmWasm for everything - revolutionizing all software!
✅ Inspired by smart contract deployment patterns (like CosmWasm), this framework enables runtime component management for general-purpose systems
```

---

## Approved Technical Terms

### ✅ USE FREELY (Technical Precision)

#### Architecture & Design
- Component-based architecture
- Pluggable system
- Modular design
- Capability-based security
- Sandbox isolation
- Actor model
- Message passing
- Supervision tree

#### Performance (With Measurements)
- Sub-millisecond latency (with specific measurement)
- High-throughput (with specific numbers)
- Low overhead (with specific percentage)
- Efficient (with comparative data)
- Fast startup (with specific time: "< 10ms")

#### Deployment
- Runtime deployment
- Dynamic loading
- Live updates
- Component lifecycle management
- Version management
- Rollback capabilities

#### Platform & Language
- Cross-platform (specify: Linux, macOS, Windows)
- Language-agnostic (list supported languages)
- Multi-language support
- Standard-compliant (specify standard)

#### Security
- Deny-by-default
- Fine-grained permissions
- Capability-based access control
- Memory isolation
- Sandboxed execution
- Security audit logging

---

## Performance Claims Standards

### ✅ REQUIRED for Performance Claims:
1. **Specific measurements** with units
2. **Test conditions** documented
3. **Comparative context** if claiming "faster"
4. **Reproducible** benchmarks

### Examples:

#### ❌ INCORRECT:
```markdown
Blazingly fast performance with instant startup
```

#### ✅ CORRECT:
```markdown
Component instantiation: < 10ms (measured with wasmtime 24.0 on macOS M1)
Target throughput: > 10,000 component calls/second
```

---

## Feature Description Standards

### ✅ State Capability, Not Hype

#### ❌ INCORRECT:
```markdown
Our revolutionary hot deployment system enables zero-downtime updates faster than anything else!
```

#### ✅ CORRECT:
```markdown
The framework supports runtime component deployment, allowing components to be loaded and updated while the host system continues running. This differs from traditional plugin systems that require application restart for component updates.
```

---

## Implementation Status Standards

### ✅ ALWAYS Indicate Status Clearly

#### Status Labels (Use Consistently):
- **Implemented** / **Complete** - Feature exists and tested
- **In Progress** - Currently being developed
- **Planned** - Designed but not started
- **Proposed** - Under consideration
- **Research** - Investigating feasibility

#### ❌ INCORRECT:
```markdown
Our framework provides hot deployment and zero-downtime updates!
```
*(Misleading if not yet implemented)*

#### ✅ CORRECT:
```markdown
**Planned Feature:** The framework will support runtime component deployment (implementation scheduled for Q1 2026).
```

---

## Use Case Description Standards

### ✅ Specific, Not Absolute

#### ❌ INCORRECT:
```markdown
Works for everything! Perfect for all use cases across any domain!
```

#### ✅ CORRECT:
```markdown
**Target Use Cases:**
- AI agent systems requiring component isolation
- Microservice architectures with runtime updates
- IoT device controllers with resource constraints
- Plugin systems requiring security guarantees

**Applicable Domains:** AI/ML, web services, IoT, gaming, enterprise applications
```

---

## Self-Claim Avoidance Standards

### ✅ Objective vs. Subjective Language

#### ❌ INCORRECT (Self-Promotional):
```markdown
Our framework is superior to all existing solutions and provides the best
performance in the industry. We are the only ones who truly understand the
problem. Our approach is revolutionary and better than competitors.
```

#### ✅ CORRECT (Objective):
```markdown
The framework provides the following measured characteristics:
- Component startup: < 10ms (measured with wasmtime 24.0)
- Memory overhead: < 1MB per component
- Cross-platform: Linux, macOS, Windows support

This differs from container-based approaches which typically have:
- Startup time: seconds
- Memory overhead: 10s-100s of MB
- Platform support: Linux-native (other platforms via emulation)
```

### ✅ Feature Description Without Self-Promotion

#### ❌ INCORRECT:
```markdown
We provide the most advanced security system with superior isolation.
Our capability-based approach is better than anything else available.
```

#### ✅ CORRECT:
```markdown
Security features:
- Capability-based access control with deny-by-default policies
- Memory isolation using WebAssembly sandboxing
- Fine-grained permission system for system resources
- Audit logging for all component operations
```

## Comparison Standards

### ✅ Honest, Specific Comparisons

#### ❌ INCORRECT:
```markdown
Better than Docker! Faster than everything! The only solution that works!
Our framework outperforms all competitors!
```

#### ✅ CORRECT:
```markdown
**Comparison with Container-Based Isolation:**

| Aspect | WASM Components | Docker Containers |
|--------|----------------|-------------------|
| Startup Time | < 10ms | Seconds |
| Memory Overhead | < 1MB | 10s-100s of MB |
| Platform Support | Cross-platform | Linux-native (or with emulation) |
| Isolation Level | Process memory | OS-level (cgroups/namespaces) |

**Note:** Containers provide OS-level isolation suitable for full applications. WASM components provide memory-level isolation suitable for code modules. Choose based on use case requirements.
```

---

## Emoji Usage Standards

### ✅ MINIMAL Use for Structure Only

#### ❌ EXCESSIVE:
```markdown
🚀 Revolutionary! ⚡ Blazingly fast! 🔥 Hot deployment! 💯 Perfect solution!
```

#### ✅ ACCEPTABLE:
```markdown
## ✅ Implemented Features
## ⏳ Planned Features
## 🎯 Target Use Cases
```

**Limit:** Maximum 1 emoji per section header, none in body text

---

## Documentation Quality Checklist

### Before Publishing Any Documentation:

- [ ] **No hyperbolic terms** (check forbidden list above)
- [ ] **Performance claims** have measurements and test conditions
- [ ] **Implementation status** clearly indicated (planned vs. complete)
- [ ] **Comparisons** are specific, honest, and balanced
- [ ] **Use cases** are specific, not absolute ("works for everything")
- [ ] **Technical terms** used correctly with definitions where needed
- [ ] **Taglines** match approved versions
- [ ] **Professional tone** throughout (no marketing speak)
- [ ] **Emoji usage** minimal and structural only
- [ ] **Sources cited** for external claims or research

---

## Examples: Before & After

### Example 1: Project Introduction

#### ❌ BEFORE (Hyperbolic):
```markdown
airssys-wasm is a revolutionary Universal Hot-Deployable WASM Component Framework 
that's blazingly fast and works for everything! 🚀 It's the CosmWasm for all 
domains with zero-downtime deployment! This game-changing technology will 
transform how you build software! ⚡
```

#### ✅ AFTER (Professional):
```markdown
airssys-wasm is a WASM Component Framework for Pluggable Systems. It enables 
runtime component deployment, allowing WebAssembly components to be loaded and 
updated while the host system continues running. The framework provides 
capability-based security, language-agnostic development through WIT interfaces, 
and supports cross-platform execution.
```

---

### Example 2: Feature Description

#### ❌ BEFORE (Hyperbolic):
```markdown
## Revolutionary Hot Deployment System 🔥

Our industry-leading hot deployment enables instant, zero-downtime updates 
faster than anything else! Deploy unlimited components universally across 
any platform with perfect isolation!
```

#### ✅ AFTER (Professional):
```markdown
## Runtime Deployment System

The framework supports runtime component deployment with the following capabilities:

- Components can be loaded during runtime without system restart
- Multiple deployment strategies: Blue-Green, Canary, Rolling updates
- Version management with rollback support
- Target deployment time: < 1 second per component
- Memory isolation between components using WebAssembly sandboxing
```

---

### Example 3: Architecture Description

#### ❌ BEFORE (Hyperbolic + Self-Promotional):
```markdown
Our groundbreaking Universal Component Interface works for absolutely everything! 
It's the most advanced framework ever built with revolutionary capabilities 
that will change software development forever! We provide superior isolation
compared to any other solution! 🚀
```

#### ✅ AFTER (Professional + Objective):
```markdown
## Component Interface Architecture

The framework uses WIT (WebAssembly Interface Types) for component interfaces, 
providing:

- Language-agnostic interface definitions
- Type-safe component communication
- Support for WASM-compatible languages (Rust, C++, Go, JavaScript, Python)
- Standard WASI functions plus extensible host functions

Components expose functions through WIT interfaces, enabling composition across 
language boundaries.
```

### Example 4: Feature Comparison

#### ❌ BEFORE (Self-Promotional):
```markdown
Our security model is far superior to traditional plugin systems. We provide
the best isolation in the industry and outperform all competitors. Our approach
is more advanced and better designed than any existing solution.
```

#### ✅ AFTER (Objective with Data):
```markdown
## Security Model Comparison

**Isolation Characteristics:**

| Feature | This Framework | Traditional Plugins | Containers |
|---------|---------------|---------------------|------------|
| Memory Isolation | Yes (WASM sandbox) | No (shared process) | Yes (OS-level) |
| Permission Model | Capability-based | OS permissions | cgroups/namespaces |
| Startup Overhead | < 10ms | < 1ms | Seconds |
| Cross-Platform | Yes | Varies | Linux-native |

**Security Approach:**
- Deny-by-default with explicit capability grants
- WebAssembly memory sandboxing prevents unauthorized access
- Fine-grained permission system for system resources
- Comprehensive audit logging

**Tradeoffs:**
- More overhead than in-process plugins
- Less isolation than OS-level containers
- Suitable for code-level component isolation use cases
```

---

## Self-Claim Detection and Replacement

### ❌ Common Self-Promotional Patterns to Avoid

**Comparative Self-Claims:**
- ❌ "Better than [X]" → ✅ Provide comparison table with measurements
- ❌ "Superior to [X]" → ✅ List specific technical differences
- ❌ "More advanced than [X]" → ✅ Describe technical approach objectively
- ❌ "Faster than [X]" → ✅ Provide benchmark data for both
- ❌ "Best in class" → ✅ Describe capabilities without ranking

**First-Person Self-Promotion:**
- ❌ "Our framework provides..." → ✅ "The framework provides..."
- ❌ "We offer superior..." → ✅ "Features include..."
- ❌ "We excel at..." → ✅ "Capabilities include..."
- ❌ "We are the only..." → ✅ "This approach differs from..."
- ❌ "We outperform..." → ✅ "Performance characteristics: [data]"

**Absolute Claims:**
- ❌ "The best solution" → ✅ "A solution for [specific use case]"
- ❌ "The only framework that..." → ✅ "This framework provides..."
- ❌ "The most advanced..." → ✅ "Features include [specific list]"

### ✅ Replacement Patterns

**Instead of Self-Claims, Use:**
1. **Technical descriptions**: Describe what it does, not how good it is
2. **Measured comparisons**: Provide data for comparison, not judgments
3. **Use case fit**: Describe suitability for specific scenarios
4. **Tradeoff analysis**: Acknowledge both strengths and limitations

**Example Replacements:**

| ❌ Self-Promotional | ✅ Objective Technical |
|---------------------|------------------------|
| "Our superior security model" | "Security model: capability-based with deny-by-default" |
| "We provide the best performance" | "Performance: < 10ms startup (measured with wasmtime 24.0)" |
| "We are the only solution for..." | "This approach differs from [X] by providing [Y]" |
| "Our advanced architecture" | "Architecture: [describe components and patterns]" |
| "We excel at isolation" | "Isolation: WebAssembly memory sandboxing" |

## Enforcement

### Review Process
1. **Self-review**: Author checks against this document before committing
2. **Peer review**: Reviewer verifies terminology compliance
3. **Documentation CI**: Automated checks for forbidden terms (future)

### Forbidden Terms Detection
When reviewing documentation, search for these patterns:
- `/[Uu]niversal/` - Likely needs replacement
- `/[Hh]ot[ -]deploy/` - Replace with "runtime deployment"
- `/[Zz]ero[ -]downtime/` - Replace with specific capability
- `/[Rr]evolutionary|[Gg]ame[ -]changing/` - Remove or replace
- `/[Bb]lazingly|[Ll]ightning/` - Remove performance hyperbole
- `/CosmWasm for (everything|all)/` - Replace with specific comparison
- `/\b(our|we|us)\s+(provide|offer|excel|superior|better|best|outperform)/i` - Remove self-promotional language
- `/\b(the best|the only|the most|superior to|better than)/i` - Remove absolute comparative claims

---

## Updates to This Document

This is a living document. Updates should be proposed via:
1. Memory bank update process
2. Discussion of specific terminology issues
3. Addition of new examples or clarifications

**Last Updated:** 2025-10-17  
**Version:** 1.0  
**Next Review:** When new terminology issues are identified
