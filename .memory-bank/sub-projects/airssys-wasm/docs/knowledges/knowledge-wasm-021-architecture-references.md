# Architecture Documentation References

**Last Updated:** 2025-12-14  
**Status:** ✅ Current and Complete

This document provides a navigation guide to essential architecture documentation created to support airssys-wasm development.

---

## 🎯 Quick Navigation

### **I want to understand...**

| Question | Document | Section |
|----------|----------|---------|
| **What is a "component" exactly?** | KNOWLEDGE-WASM-018 | Part 1 (1.1-1.5) |
| **How do the three layers work together?** | ADR-WASM-018 + KNOWLEDGE-WASM-018 | Full documents |
| **Which layer owns this feature?** | KNOWLEDGE-WASM-018 | Part 4 (4.1-4.2) Decision Tree |
| **Where should I put my code?** | KNOWLEDGE-WASM-018 | Part 4 (4.1-4.4) Guidelines |
| **Does airssys-wasm create its own actor system?** | KNOWLEDGE-WASM-018 | Part 1 (1.2) + Part 3 (3.1) |
| **What are the boundaries between layers?** | ADR-WASM-018 | Layer Responsibilities Matrix |
| **What anti-patterns should I avoid?** | KNOWLEDGE-WASM-018 | Part 4 (4.3) Anti-Patterns |
| **Who owns mailbox/supervision/routing?** | ADR-WASM-018 | Explicit Non-Ownership Statements |

---

## 📚 Complete Documentation Set

### **ADR-WASM-018: Three-Layer Architecture and Boundary Definitions**
**File:** `.memory-bank/sub-projects/airssys-wasm/docs/adr/adr-wasm-018-three-layer-architecture.md`

**Purpose:** Architecture decision record that codifies three-layer architecture

**Key Sections:**
1. **Context** - Problem statement and decision rationale
2. **Decision** - Three explicit layers with ownership
3. **Architectural Diagram** - Visual representation
4. **Layer Responsibilities Matrix** - Feature ownership table
5. **Integration Patterns** - How layers connect
6. **Explicit Non-Ownership Statements** - What each layer does NOT own
7. **Dependencies** - Dependency relationships
8. **Consequences** - Positive and negative impacts
9. **Implementation Status** - What's done, what's coming
10. **Related Documents** - Cross-references

**Read this when:**
- Making architectural decisions
- Questioning whether airssys-wasm owns something
- Implementing new features
- Reviewing code architecture
- Planning Phase 3+ work

**Size:** ~15KB, ~400 lines

---

### **KNOWLEDGE-WASM-018: Component Definitions and Three-Layer Architecture**
**File:** `.memory-bank/sub-projects/airssys-wasm/docs/knowledges/knowledge-wasm-018-component-definitions-and-architecture-layers.md`

**Purpose:** Comprehensive reference guide for component architecture

**Key Sections:**
1. **Part 1: Component Definition** (1.1-1.5)
   - What is a "component" (precise definition)
   - At creation time vs. runtime
   - Key characteristics table
   - Component lifecycle with state diagram

2. **Part 2: Three-Layer Architecture Detailed** (2.1-2.3)
   - Layer 1: WASM Component Configuration & Tracking
   - Layer 2: WASM Component Lifecycle & Spawning
   - Layer 3: Actor System Runtime (airssys-rt)
   - Code examples for each layer

3. **Part 3: Ownership & Responsibility Matrix** (3.1-3.3)
   - Feature ownership table
   - Dependency flow diagram
   - Data flow at runtime
   - Shared vs. exclusive responsibilities

4. **Part 4: Development Guidelines** (4.1-4.4)
   - Decision tree: "Where should I put this?"
   - Feature implementation checklist
   - Anti-patterns to avoid (4 examples)
   - Phase 3+ integration checklist

5. **Part 5: Common Questions & Answers** (Q1-Q7)
   - Q1: Can I use airssys-wasm's RestartBackoff?
   - Q2: Should ComponentActor import from airssys-rt?
   - Q3: Can I access the mailbox?
   - Q4: Should ComponentSupervisor make decisions?
   - Q5: Inter-component messaging flow
   - Q6: Customize SupervisorNode strategy?
   - Q7: Where does capability checking happen?

6. **Part 6: Phase Evolution**
   - Current phase status
   - Phase 3 plans
   - Phase 4+ roadmap

7. **Appendix: Quick Reference**
   - File locations by layer
   - Key types by layer

**Read this when:**
- Implementing a feature
- Questioning component definition
- Understanding component lifecycle
- Reviewing architecture
- Wanting detailed examples and code
- Phase 3+ planning

**Size:** ~22KB, ~550 lines

---

## 🔗 Document Relationships

```
User Question
    ↓
├─ Quick Navigation Table (above) points to section
│
├─ ADR-WASM-018 (Decision Record)
│   ├─ Read for: DECISION and RATIONALE
│   ├─ Related to all previous ADRs
│   └─ Links to KNOWLEDGE-WASM-018
│
└─ KNOWLEDGE-WASM-018 (Reference Guide)
    ├─ Read for: DETAILED EXPLANATION and EXAMPLES
    ├─ Implements the ADR
    ├─ Provides development guidelines
    └─ Answers common questions
```

---

## 📋 Quick Reference: Three Layers

### Layer 1: WASM Component Configuration & Tracking
**Ownership:** airssys-wasm  
**Responsibility:** Define policies, track component state  
**Key Types:** SupervisorConfig, BackoffStrategy, ComponentSupervisor  
**Files:** `src/actor/supervisor_config.rs`, `component_supervisor.rs`  

### Layer 2: WASM Component Lifecycle & Spawning
**Ownership:** airssys-wasm  
**Responsibility:** Load WASM binaries, manage component actors, track instances  
**Key Types:** ComponentActor, ComponentSpawner, ComponentRegistry, WasmRuntime  
**Files:** `src/actor/component_actor.rs`, `component_spawner.rs`, `component_registry.rs`, `src/runtime/`  

### Layer 3: Actor System Runtime
**Ownership:** airssys-rt  
**Responsibility:** Actor system, mailbox, message broker, supervision  
**Key Types:** ActorSystem, SupervisorNode, MessageBroker, Mailbox, ActorAddress  
**Files:** airssys-rt entire crate  

---

## ✅ Current Status (2025-12-14)

### Documented
- ✅ ADR-WASM-018 - Decision record approved
- ✅ KNOWLEDGE-WASM-018 - Comprehensive reference complete
- ✅ This navigation guide

### Implementation Status
- ✅ Phase 2.1 Complete (ComponentSpawner, type conversion)
- ✅ Phase 2.2 Complete (ComponentRegistry)
- ⏳ Phase 2.3 Ready (Actor Address and Routing)
- ⏳ Phase 3 Planning (Supervisor integration)

---

## 🎓 Reading Guide by Role

### **For Developers Implementing Features**
1. Start: KNOWLEDGE-WASM-018, Part 4 (Development Guidelines)
2. Decision tree: "Where should I put this?"
3. Reference: Anti-patterns section
4. Implement according to guidelines

### **For Architects & Reviewers**
1. Start: ADR-WASM-018 (complete)
2. Deep dive: KNOWLEDGE-WASM-018, Part 2 & 3
3. Reference: Ownership & responsibility matrix
4. Verify: Feature ownership and layer boundaries

### **For New Team Members**
1. Start: KNOWLEDGE-WASM-018, Part 1 (Component Definition)
2. Overview: Part 2 (Three-Layer Architecture)
3. Reference: Part 5 (FAQs)
4. Keep: Part 4 (Guidelines) as reference

### **For Phase 3+ Planning**
1. Start: KNOWLEDGE-WASM-018, Part 6 (Phase Evolution)
2. Reference: Part 3 (Responsibility Matrix)
3. Review: ADR-WASM-018 (Integration Patterns)

---

## 🔍 Finding Specific Information

### **"Where is mailbox handled?"**
→ KNOWLEDGE-WASM-018, Part 3, Data Flow  
→ ADR-WASM-018, Explicit Non-Ownership  
**Answer:** Layer 3 (airssys-rt)

### **"Which layer owns restart decisions?"**
→ KNOWLEDGE-WASM-018, Part 3, Responsibility Matrix  
→ ADR-WASM-018, Layer Responsibilities Matrix  
**Answer:** Layer 3 (airssys-rt SupervisorNode)

### **"Can I modify SupervisorConfig from Layer 1?"**
→ KNOWLEDGE-WASM-018, Part 4.1, Decision Tree  
→ ADR-WASM-018, Dependencies  
**Answer:** Yes, Layer 1 owns configuration

### **"What's the component lifecycle?"**
→ KNOWLEDGE-WASM-018, Part 1.4, Component Lifecycle  
**Answer:** 6 stages with detailed state diagram

### **"How do components communicate?"**
→ KNOWLEDGE-WASM-018, Part 5, Q5  
→ KNOWLEDGE-WASM-018, Part 3, Data Flow  
**Answer:** Via ActorAddress → MessageBroker → Mailbox

### **"Who owns component addressing?"**
→ KNOWLEDGE-WASM-018, Part 3, Ownership Matrix  
→ ADR-WASM-018, Explicit Non-Ownership  
**Answer:** Layer 3 provides ActorAddress, Layer 2 uses and tracks it

---

## 📞 Support

**Have a question not listed?**
1. Check Quick Navigation (above)
2. Search KNOWLEDGE-WASM-018 Part 5 (FAQs)
3. Review KNOWLEDGE-WASM-018 Part 4 (Guidelines)
4. Check ADR-WASM-018 (Decision Details)

**Found an error or inconsistency?**
1. Report in memory bank issue tracker
2. Update both ADR and KNOWLEDGE documents together
3. Keep index synchronized

---

## 📖 Related Documentation

### Previous ADRs
- **ADR-WASM-006:** Component Isolation and Sandboxing
- **ADR-WASM-009:** Component Communication Model
- **ADR-WASM-010:** Implementation Strategy and Build Order
- **ADR-RT-004:** Actor and Child Trait Separation (airssys-rt)

### Previous Knowledge Documents
- **KNOWLEDGE-WASM-001:** Component Framework Architecture
- **KNOWLEDGE-WASM-016:** Actor System Integration Implementation Guide
- **KNOWLEDGE-WASM-017:** Health Check System Architecture

### Project Standards
- **PROJECTS_STANDARD.md:** AirsSys project standards and conventions
- **Workspace standards:** `.aiassisted/instructions/` (multi-project memory bank, Rust guidelines, etc.)

---

## 📝 Document Maintenance

**Last Updated:** 2025-12-14  
**Maintainer:** Architecture Team  
**Version:** 1.0  
**Status:** ✅ CURRENT & COMPLETE

**Update Schedule:**
- ✅ Updated after each phase completion
- ✅ Updated after architectural decisions
- ✅ Updated before major implementation phases
- Check git history for previous versions

---

**Quick Links:**
- 📄 [ADR-WASM-018](docs/adr/adr-wasm-018-three-layer-architecture.md)
- 📚 [KNOWLEDGE-WASM-018](docs/knowledges/knowledge-wasm-018-component-definitions-and-architecture-layers.md)
- 📋 [ADR Index](docs/adr/-index.md)
- 📚 [Knowledge Index](docs/knowledges/-index.md)
