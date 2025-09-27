# Technical Debt Record Template

**Document ID:** DEBT-[XXX]-[short-description]  
**Created:** [YYYY-MM-DD]  
**Updated:** [YYYY-MM-DD]  
**Status:** [active/resolved/superseded]  
**Category:** [DEBT-ARCH/DEBT-QUALITY/DEBT-DOCS/DEBT-TEST/DEBT-PERF]  

## Summary
Brief description of the technical debt (1-2 sentences).

## Context
### Background
Explain the situation that led to this technical debt.

### Decision Point
Describe the specific decision or compromise that created the debt.

### Constraints
List the constraints (time, resources, knowledge) that influenced the decision.

## Technical Details
### Code Location
- **Files:** List specific files and line numbers
- **Components:** Affected components or modules
- **Dependencies:** Related dependencies or systems

### Current Implementation
Describe the current approach that constitutes technical debt.

### Impact Assessment
- **Performance Impact:** Quantify performance implications
- **Maintainability Impact:** How does this affect code maintainability
- **Security Impact:** Any security implications
- **Scalability Impact:** Effects on system scalability

## Remediation Plan
### Ideal Solution
Describe the proper implementation that would resolve the debt.

### Implementation Steps
1. Step 1: [Description]
2. Step 2: [Description]
3. Step 3: [Description]

### Effort Estimate
- **Development Time:** [hours/days/weeks]
- **Testing Time:** [hours/days]
- **Risk Level:** [low/medium/high]

### Dependencies
List any dependencies that must be resolved before remediation.

## Tracking
### GitHub Issue
- **Issue:** [Link to GitHub issue if created]
- **Labels:** Applied labels for categorization

### Workspace Standards
- **Standards Violated:** Reference specific workspace standards (e.g., §3.2)
- **Compliance Impact:** How this affects overall workspace compliance

### Priority
- **Business Priority:** [low/medium/high/critical]
- **Technical Priority:** [low/medium/high/critical]
- **Recommended Timeline:** Target resolution timeframe

## History
### Changes
- **[YYYY-MM-DD]:** [Description of change]
- **[YYYY-MM-DD]:** [Description of change]

### Related Decisions
- **ADR References:** Links to related Architecture Decision Records
- **Other Debt:** Links to related technical debt records

## Resolution
*[Fill when resolved]*
### Resolution Date
[YYYY-MM-DD]

### Resolution Summary
Brief description of how the debt was resolved.

### Lessons Learned
Key insights gained from resolving this debt.

---
**Template Version:** 1.0  
**Last Updated:** 2025-09-27