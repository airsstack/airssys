# Knowledge Documentation Template

**Document ID:** KNOWLEDGE-[category]-[name]  
**Created:** [YYYY-MM-DD]  
**Updated:** [YYYY-MM-DD]  
**Category:** [architecture/patterns/performance/integration/security/domain]  
**Maturity:** [draft/stable/deprecated]  

## Overview
Brief description of the knowledge being documented (2-3 sentences).

## Context
### Problem Statement
Describe the specific problem or challenge this knowledge addresses.

### Scope
Define the boundaries and applicability of this knowledge.

### Prerequisites  
List required background knowledge or system state.

## Technical Content
### Core Concepts
Explain the fundamental concepts necessary to understand this knowledge.

### Implementation Details
Provide specific technical details, algorithms, or approaches.

### Code Examples
```rust
// Provide working, compilable code examples
// All code MUST follow workspace standards (§2.1, §3.2, §4.3, §5.1)
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use crate::example::Module;

pub fn example_function() -> Result<(), ExampleError> {
    let timestamp = Utc::now(); // ✅ Follows §3.2
    // Implementation details
    Ok(())
}
```

### Configuration
Document any configuration requirements or options.

## Usage Patterns
### Common Use Cases
List typical scenarios where this knowledge applies.

### Best Practices
Document recommended approaches and patterns.

### Antipatterns
Highlight approaches to avoid and why.

## Performance Considerations
### Performance Characteristics
Describe performance implications and characteristics.

### Optimization Opportunities
Identify potential optimization approaches.

### Benchmarks
Include relevant performance measurements if available.

## Integration Points
### Dependencies
List systems, components, or libraries this integrates with.

### Compatibility
Document version compatibility and requirements.

### Migration Paths
Describe upgrade or migration strategies if applicable.

## Security Considerations
### Security Implications
Document security aspects and considerations.

### Threat Model
Identify potential security threats and mitigations.

### Compliance
Note any compliance requirements or standards.

## Maintenance
### Review Schedule
Recommended frequency for reviewing and updating this knowledge.

### Update Triggers
Conditions that should trigger knowledge updates.

### Owner/Maintainer
Primary contact for questions and updates.

## References
### Related Documentation
- **ADRs:** Links to related Architecture Decision Records
- **Technical Debt:** Links to related debt records
- **External References:** Links to external documentation or resources

### Workspace Standards
- **Standards Applied:** Reference specific workspace standards used
- **Compliance Notes:** How this knowledge maintains workspace compliance

## History
### Version History
- **[YYYY-MM-DD]:** [Version] - [Description of changes]
- **[YYYY-MM-DD]:** [Version] - [Description of changes]

### Review History
- **[YYYY-MM-DD]:** Reviewed by [Reviewer] - [Status/Comments]
- **[YYYY-MM-DD]:** Reviewed by [Reviewer] - [Status/Comments]

---
**Template Version:** 1.0  
**Last Updated:** 2025-09-27