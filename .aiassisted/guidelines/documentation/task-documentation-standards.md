# Task Documentation Standards

**These standards are MANDATORY for all task documentation within the workspace.**

## 1. Task Documentation Pattern
**When creating or updating tasks, ALWAYS include a Standards Compliance Checklist:**

```markdown
## Standards Compliance Checklist
**Workspace Standards Applied** (Reference: `[path/to/standards.md]`):
- [ ] **[Standard Name 1]** (e.g., Import Organization) - [Status and evidence]
- [ ] **[Standard Name 2]** (e.g., Error Handling) - [Status and evidence]  
- [ ] **[Standard Name 3]** (e.g., Module Architecture) - [Status and evidence]
- [ ] **[Standard Name 4]** (e.g., Dependency Management) - [Status and evidence]
- [ ] **[Policy Name]** (e.g., Zero Warning Policy) - [Status and evidence]

## Compliance Evidence
[Document proof of standards application with code examples]
```

## 2. Reference, Don't Duplicate
**NEVER explain what workspace standards are in project tasks.**
**ALWAYS reference workspace documentation:**
- ✅ "Per [standards-file.md] §[Section]..."
- ❌ "[Standard Rule] is required because..."

## 3. Evidence Documentation
**ALWAYS provide concrete evidence of compliance:**
```rust
// Evidence of [Standard Name] compliance
impl Context {
    pub fn operation(&self) {
        // ✅ Uses workspace standard pattern
        let result = standard_pattern(); 
        // ...
    }
}
```

## 4. Technical Debt Documentation
**When introducing ANY technical debt:**
```rust
// TODO(DEBT): [Category] - [Description]
// Impact: [Performance/Maintainability impact]
// Remediation: [Specific fix needed]
// Reference: [Issue Tracker ID if created]
// Workspace Standard: [Which standard is violated, if any]
```

**Debt Categories:**
- `DEBT-ARCH`: Architectural debt
- `DEBT-QUALITY`: Code quality debt  
- `DEBT-DOCS`: Documentation debt
- `DEBT-TEST`: Testing debt
- `DEBT-PERF`: Performance debt

## 5. Documentation Quality Metrics
- Task files reference workspace standards rather than explaining them
- Compliance evidence provided for all standards claims
- Clear separation between workspace rules and project application
- Technical debt properly documented and tracked
