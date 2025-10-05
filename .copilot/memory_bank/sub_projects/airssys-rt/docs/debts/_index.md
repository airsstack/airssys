# airssys-rt Technical Debt Index

**Sub-Project:** airssys-rt  
**Last Updated:** 2025-09-27  
**Total Debt Items:** 0  
**Active Debt Items:** 0  

## Anticipated Debt Categories

### Expected DEBT-ARCH  
- Actor state management simplifications during initial implementation
- Message routing optimizations deferred for basic functionality
- Supervisor tree complexity reductions for MVP

### Expected DEBT-PERF
## Pending Debts

### High Priority

- **DEBT-RT-004**: Request-Reply Serialization Performance
  - **Component**: Message Broker (InMemoryMessageBroker)
  - **Issue**: Uses JSON serialization for in-process request-reply (2-10μs overhead)
  - **Solution**: TypedBox pattern for zero-copy type erasure (~100ns overhead)
  - **Impact**: ~100x performance improvement for request-reply pattern
  - **Target**: RT-TASK-010 (Performance Optimization) or Q1 2026
  - **File**: `debt_rt_004_request_reply_serialization.md`

### Medium Priority

- Message serialization strategy selection for cross-boundary communication
- Memory pooling implementations postponed
- Actor scheduling optimizations delayed

### Expected DEBT-QUALITY
- Error handling simplifications in actor boundaries
- Testing coverage gaps during rapid development
- Documentation gaps for complex actor patterns

### Debt Prevention Strategy
- Follow workspace standards (§2.1, §3.2, §4.3, §5.1) from start
- Implement comprehensive testing early
- Regular performance benchmarking
- Continuous integration with debt detection

---
**Note:** Debt tracking will begin when development starts.