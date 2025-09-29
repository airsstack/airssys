# Documentation Guidelines

## Documentation Strategy
This document provides guidelines for when and how to create technical documentation within the AirsSys workspace using the standardized template system.

## MANDATORY Documentation Quality Standards (§7.2)
**ALL documentation MUST adhere to professional software engineering standards:**

### Critical Requirements
- ✅ **No assumptions**: Document only implemented or officially planned features
- ✅ **No fictional content**: All examples must be real or explicitly marked as planned
- ✅ **Source all claims**: Reference memory bank, code, or specifications
- ✅ **Professional tone**: No excessive emoticons, hyperbole, or self-promotional language
- ✅ **Status accuracy**: Clearly indicate implementation status at all times

### Forbidden Patterns
- ❌ Unsourced performance claims ("blazingly fast", "lightning speed")
- ❌ Marketing language ("best-in-class", "revolutionary", "cutting-edge")
- ❌ Excessive emoji usage in technical documentation
- ❌ Speculative features without official planning documentation
- ❌ Assumptions about implementation without memory bank verification

## Documentation Types and Triggers

### Technical Debt Records
**When to Create:**
- Any `TODO(DEBT)` comment is added to code
- Workspace standards violations are identified but not immediately fixed
- Architectural shortcuts are taken due to time or resource constraints
- Code quality compromises are made for expedient delivery
- Performance optimizations are deferred
- Testing coverage gaps are identified but not immediately addressed

**Template:** `templates/docs/technical-debt-template.md`

**Example Triggers:**
```rust
// TODO(DEBT): DEBT-QUALITY - Error handling simplified for MVP
// Impact: Reduced error context, harder debugging
// Remediation: Implement structured error types with full context
// Reference: GitHub issue #123
// Workspace Standard: Violates comprehensive error handling patterns
```

### Knowledge Documentation
**When to Create:**
- Complex algorithms or data structures are implemented
- Integration patterns with external systems are established
- Performance-critical code sections are optimized
- Reusable architectural patterns are developed
- Security implementations or protocols are added
- Domain-specific business logic is implemented

**Template:** `templates/docs/knowledge-template.md`

**Categories:**
- **architecture/**: System design patterns, component interactions
- **patterns/**: Reusable code patterns, design patterns
- **performance/**: Optimization techniques, performance analysis
- **integration/**: External system integration approaches
- **security/**: Security implementations, threat mitigations
- **domain/**: Business logic, domain-specific knowledge

### Architecture Decision Records (ADRs)
**When to Create:**
- Technology selections (programming language, frameworks, libraries)
- Architectural patterns or styles are chosen
- Data storage or persistence strategies are defined
- Security models or authentication approaches are selected
- Performance targets or optimization strategies are established
- Integration approaches with external systems are decided
- API design patterns or communication protocols are chosen

**Template:** `templates/docs/adr-template.md`

## Quality Standards

### Code Examples
- **Compilation Requirement:** ALL code examples MUST compile and run correctly
- **Standards Compliance:** Code must follow workspace standards (§2.1, §3.2, §4.3, §5.1)
- **Import Organization:** Use 3-layer import structure
- **Time Handling:** Use `chrono::DateTime<Utc>` for all time operations
- **Error Handling:** Include proper error handling in examples

### Content Standards
- **Workspace Standards Reference:** Always reference workspace standards rather than duplicating content
- **Cross-References:** Maintain links between related documentation types
- **Accuracy:** Ensure technical accuracy and currentness
- **Completeness:** Cover all necessary aspects as outlined in templates

### Maintenance Requirements
- **Regular Review:** Document review schedules in each document
- **Update Triggers:** Define conditions requiring updates
- **Ownership:** Assign clear ownership/maintainership
- **Version Control:** Track document versions and changes

## Workflow Integration

### Development Process
1. **Before Implementation:** Check if ADR is needed for architectural decisions
2. **During Implementation:** Create knowledge docs for complex implementations
3. **Code Review:** Verify documentation completeness and accuracy
4. **Technical Debt:** Document any debt created during implementation
5. **Post-Implementation:** Update related documentation and create new docs as needed

### Task Integration
- **Task Planning:** Include documentation requirements in task breakdown
- **Task Completion:** Verify all required documentation is created/updated
- **Standards Compliance:** Document compliance evidence in task files
- **Cross-References:** Link tasks to related documentation

### Code Review Integration
- **Documentation Completeness:** Verify all triggers are addressed
- **Template Compliance:** Ensure correct template usage
- **Cross-Reference Validation:** Check links and references
- **Standards Compliance:** Verify workspace standards are followed

## Index Maintenance

### Required Index Files
Each documentation type maintains an `_index.md` file:
- `docs/debts/_index.md`: Technical debt registry
- `docs/knowledges/_index.md`: Knowledge document catalog  
- `docs/adr/_index.md`: ADR chronological registry

### Index Templates
- **Technical Debt Index:** `templates/docs/debt-index-template.md`
- **ADR Index:** `templates/docs/adr-index-template.md`
- **Knowledge Index:** Organized by category with status tracking

### Update Requirements
- **Concurrent Updates:** Update indexes when creating/modifying documents
- **Status Tracking:** Maintain accurate status information
- **Cross-References:** Link related documents in indexes
- **Search Optimization:** Include keywords and tags for discoverability

## Document Lifecycle

### Creation Process
1. **Identify Trigger:** Recognize documentation need based on triggers
2. **Select Template:** Choose appropriate template type
3. **Create Document:** Use exact template structure
4. **Update Index:** Add entry to appropriate index file
5. **Cross-Reference:** Link to related existing documentation

### Review Process
- **Initial Review:** Verify template compliance and accuracy
- **Scheduled Review:** Follow review schedule defined in document
- **Update Review:** Review when triggering conditions change
- **Compliance Review:** Ensure workspace standards compliance

### Retirement Process
- **Deprecation:** Mark documents as deprecated when superseded
- **Archive:** Move deprecated documents to archive section
- **Index Updates:** Update indexes to reflect status changes
- **Cross-Reference Updates:** Update links in related documents

## Success Metrics

### Coverage Metrics
- Documentation exists for all complex implementations
- All architectural decisions are recorded in ADRs
- All technical debt is properly documented and tracked
- Knowledge gaps are identified and addressed

### Quality Metrics
- All code examples compile and run correctly
- Documentation is reviewed and updated regularly
- Cross-references are accurate and maintained
- Workspace standards compliance is documented

### Usage Metrics
- Documentation is referenced during development
- Decisions are informed by existing ADRs
- Technical debt is actively managed using debt records
- Knowledge is successfully transferred through documentation

---
**Version:** 1.0  
**Last Updated:** 2025-09-27  
**Next Review:** 2025-12-27