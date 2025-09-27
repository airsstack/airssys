# Technical Debt Index Template

**Sub-Project:** [sub_project_name]  
**Last Updated:** [YYYY-MM-DD]  
**Total Debt Items:** [count]  
**Active Debt Items:** [count]  

## Debt Summary

### By Category
| Category | Active | Resolved | Total | Priority Breakdown |
|----------|--------|----------|-------|-------------------|
| DEBT-ARCH | [count] | [count] | [count] | High: [count], Medium: [count], Low: [count] |
| DEBT-QUALITY | [count] | [count] | [count] | High: [count], Medium: [count], Low: [count] |
| DEBT-DOCS | [count] | [count] | [count] | High: [count], Medium: [count], Low: [count] |
| DEBT-TEST | [count] | [count] | [count] | High: [count], Medium: [count], Low: [count] |
| DEBT-PERF | [count] | [count] | [count] | High: [count], Medium: [count], Low: [count] |

### By Priority
| Priority | Count | Recommended Timeline |
|----------|-------|---------------------|
| Critical | [count] | Immediate (within 1 week) |
| High | [count] | Next Sprint (2-4 weeks) |
| Medium | [count] | Next Release (1-3 months) |
| Low | [count] | Future Consideration (>3 months) |

## Active Debt Items

### Critical Priority
| ID | Category | Title | Created | GitHub Issue | Owner |
|----|----------|-------|---------|--------------|-------|
| [DEBT-001] | [DEBT-ARCH] | [Short title] | [YYYY-MM-DD] | [Link] | [Name] |
| [DEBT-002] | [DEBT-QUALITY] | [Short title] | [YYYY-MM-DD] | [Link] | [Name] |

### High Priority  
| ID | Category | Title | Created | GitHub Issue | Owner |
|----|----------|-------|---------|--------------|-------|
| [DEBT-003] | [DEBT-PERF] | [Short title] | [YYYY-MM-DD] | [Link] | [Name] |
| [DEBT-004] | [DEBT-TEST] | [Short title] | [YYYY-MM-DD] | [Link] | [Name] |

### Medium Priority
| ID | Category | Title | Created | GitHub Issue | Owner |
|----|----------|-------|---------|--------------|-------|
| [DEBT-005] | [DEBT-DOCS] | [Short title] | [YYYY-MM-DD] | [Link] | [Name] |

### Low Priority
| ID | Category | Title | Created | GitHub Issue | Owner |
|----|----------|-------|---------|--------------|-------|
| [DEBT-006] | [DEBT-QUALITY] | [Short title] | [YYYY-MM-DD] | [Link] | [Name] |

## Recently Resolved

| ID | Category | Title | Created | Resolved | Resolution Summary |
|----|----------|-------|---------|----------|-------------------|
| [DEBT-XXX] | [DEBT-ARCH] | [Short title] | [YYYY-MM-DD] | [YYYY-MM-DD] | [Brief resolution description] |

## Debt Trends

### Monthly Resolution Rate
- **Target:** [X] items per month
- **Current Rate:** [Y] items per month  
- **Trend:** [Improving/Stable/Declining]

### Age Analysis
- **Items >90 days:** [count] (Review priority and viability)
- **Items 30-90 days:** [count] (Normal age range)
- **Items <30 days:** [count] (Recently created)

## Remediation Planning

### Sprint Planning Integration
- **Current Sprint:** [count] debt items included
- **Next Sprint:** [count] debt items planned
- **Backlog:** [count] debt items in product backlog

### Resource Allocation
- **Recommended Debt Budget:** 20% of development time
- **Current Allocation:** [X]% of development time
- **Adjustment Needed:** [Increase/Decrease/Maintain]

## Cross-References

### Related ADRs
- [ADR-001]: [Title] - Decisions that created/resolved debt
- [ADR-002]: [Title] - Architectural decisions affecting debt

### Related Knowledge Docs
- [KNOWLEDGE-patterns-error-handling]: Error handling patterns for debt remediation
- [KNOWLEDGE-architecture-module-structure]: Module structure for architectural debt

### Related Tasks
- [task_001]: Active task addressing debt items
- [task_002]: Planned task for debt remediation

## Maintenance

### Review Schedule
- **Weekly:** Priority assessment and new debt triage
- **Monthly:** Trend analysis and remediation planning
- **Quarterly:** Comprehensive debt strategy review

### Update Triggers
- New debt items created
- Debt items resolved or status changes
- Priority reassessments
- GitHub issue status changes

### Index Maintenance
- **Owner:** [Name/Role]
- **Backup:** [Name/Role] 
- **Next Review:** [YYYY-MM-DD]

---
**Template Version:** 1.0  
**Last Updated:** 2025-09-27