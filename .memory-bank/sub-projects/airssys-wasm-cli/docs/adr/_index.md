# Architecture Decision Records Index: airssys-wasm-cli

**Sub-Project:** airssys-wasm-cli  
**Last Updated:** 2025-12-18  
**Total ADRs:** 1

---

## Active ADRs

### ADR-CLI-001: Library-Only Architecture

**File:** `adr-cli-001-library-only-architecture.md`  
**Status:** ✅ Accepted  
**Created:** 2025-12-18  
**Deciders:** Architecture Team, Platform Team

**Summary:**
airssys-wasm-cli is implemented as a 100% library crate with zero binary components. All CLI functionality is exported as Clap-based structures that can be composed by any binary application (primarily airsstack).

**Key Decision:**
- NO `[[bin]]` section in Cargo.toml
- All commands exported as reusable Clap structures
- Maximum reusability and composability
- Perfect integration with airsstack binary

**Rationale:**
- Enables airsstack integration without code duplication
- Maximizes reusability across multiple binaries
- Improves testability (direct function calls)
- Provides flexibility for future distribution models

**Impact:**
- Positive: Simplified architecture, better testability, single source of truth
- Consideration: Requires binary to be useful (airsstack fulfills this role)

**Related Documents:**
- KNOWLEDGE-CLI-002: Composable CLI Pattern (implementation details)
- TASK-CLI-001: Foundation Setup (retroactive documentation)
- TASK-CLI-002: Trust Command (first composable command)

---

## ADR Status Legend

- ✅ **Accepted** - Approved and implemented
- 📋 **Proposed** - Under review
- ⏸️ **Deferred** - Postponed for later consideration
- ❌ **Rejected** - Not accepted
- 🔄 **Superseded** - Replaced by newer ADR
- 📚 **Deprecated** - No longer relevant

---

## ADR Categories

### Architecture & Design (1 ADR)
- ADR-CLI-001: Library-Only Architecture ✅

### Security (0 ADRs)
- (Future: Security-related decisions)

### Performance (0 ADRs)
- (Future: Performance optimization decisions)

### Integration (0 ADRs)
- (Future: Integration pattern decisions)

---

## Planned ADRs (Future)

### Q1 2026 (Phase 2)
- **ADR-CLI-002**: Async Runtime Choice (Tokio vs alternatives)
- **ADR-CLI-003**: Error Handling Strategy (CliError design)
- **ADR-CLI-004**: Configuration Management Approach (TOML vs other formats)

### Q2 2026 (Phase 3)
- **ADR-CLI-005**: Template System for Init Command (handlebars vs tera vs minijinja)
- **ADR-CLI-006**: Build Orchestration Strategy (language detection approach)
- **ADR-CLI-007**: Progress UI Design (terminal vs structured output)

### Q3 2026 (Phase 4)
- **ADR-CLI-008**: Shell Completion Distribution (bundled vs generated)
- **ADR-CLI-009**: Binary Naming Convention (if standalone binary created)
- **ADR-CLI-010**: Release Automation Strategy (GitHub Actions workflow)

---

## Decision-Making Process

### When to Create an ADR

**Create ADR When**:
- Architectural decision affects multiple components
- Decision has long-term impact (>6 months)
- Decision is reversible but costly to reverse
- Multiple viable options exist
- Decision affects external stakeholders
- Team needs alignment on approach

**Don't Create ADR For**:
- Implementation details (document in code comments)
- Temporary workarounds
- Obvious decisions with single option
- Low-impact local changes

### ADR Review Process

1. **Proposal**: Author creates ADR in `Proposed` status
2. **Review**: Architecture team reviews (async or meeting)
3. **Discussion**: Stakeholders provide feedback
4. **Revision**: Author incorporates feedback
5. **Decision**: Architecture team approves or rejects
6. **Implementation**: Status changed to `Accepted` or `Rejected`
7. **Review**: Periodic review (quarterly or on trigger events)

### ADR Update Triggers

**Update ADR When**:
- Implementation reveals new information
- Assumptions prove incorrect
- New options become available
- Performance data contradicts expectations
- User feedback suggests reconsideration

**Update Process**:
1. Document change in "Updates" section
2. Update "Updated" date
3. Add to "History" section
4. Notify stakeholders if significant change

---

## Cross-Project ADR References

### From airssys-wasm

**ADR-WASM-005: Capability-Based Security Model**
- **Relevance**: CLI must respect security model
- **Impact**: Commands must validate capabilities

**ADR-WASM-010: Implementation Strategy**
- **Relevance**: CLI depends on core library implementation
- **Impact**: CLI blocked until core library reaches milestones

---

## ADR Statistics

- **Total ADRs**: 1
- **Accepted**: 1
- **Proposed**: 0
- **Rejected**: 0
- **Superseded**: 0
- **Deprecated**: 0
- **Last Created**: 2025-12-18

---

## Maintenance

### Review Schedule

**Quarterly Reviews**:
- Check if ADRs remain relevant
- Update status if implementations diverged
- Archive superseded ADRs

**Annual Reviews**:
- Comprehensive review of all ADRs
- Identify patterns for new best practices
- Update ADR template if needed

### Quality Standards

All ADRs must include:
- [ ] Clear problem statement
- [ ] Business and technical context
- [ ] Multiple considered options
- [ ] Explicit decision and rationale
- [ ] Implementation plan
- [ ] Success criteria
- [ ] Risk assessment
- [ ] References to related documentation

### Naming Convention

Format: `adr-cli-[number]-[description].md`

Examples:
- `adr-cli-001-library-only-architecture.md`
- `adr-cli-002-async-runtime-choice.md`
- `adr-cli-003-error-handling-strategy.md`

---

## Related Documentation

- `../project_brief.md` - Project overview
- `../tech_context.md` - Technical architecture
- `../system_patterns.md` - Implementation patterns
- `../docs/knowledges/` - Knowledge base
- `../tasks/` - Implementation tasks

---

## External Resources

### ADR Best Practices
- [ADR GitHub Organization](https://adr.github.io/)
- [Documenting Architecture Decisions](https://cognitect.com/blog/2011/11/15/documenting-architecture-decisions)
- [Architecture Decision Records at Spotify](https://engineering.atspotify.com/2020/04/when-should-i-write-an-architecture-decision-record/)

### Rust-Specific
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Microsoft Rust Guidelines](https://github.com/microsoft/rust-guidelines)
