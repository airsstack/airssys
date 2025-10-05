# ADR-RT-003: Backpressure Strategy Simplification

**Status**: Accepted  
**Date**: 2025-10-05  
**Context**: RT-TASK-003 Phase 3 implementation review  
**Deciders**: Technical review during implementation  

## Context and Problem Statement

During RT-TASK-003 Phase 3 implementation, we initially implemented four backpressure strategies:
- `Block` - Async wait for space
- `DropOldest` - Drop oldest message in queue
- `DropNewest` - Drop incoming message
- `Error` - Return error immediately

However, `DropOldest` and `DropNewest` had **identical behavior** due to tokio mpsc channel limitations - both dropped the incoming (newest) message when the mailbox was full.

**Problem**: Should we keep a misleading API that promises functionality we can't deliver, or simplify to an honest, YAGNI-compliant design?

## Decision Drivers

- **§6.1 YAGNI Principle**: Don't implement what we can't actually deliver
- **Honest API Design**: API should accurately reflect actual behavior
- **Technical Constraints**: tokio::sync::mpsc::Sender doesn't support:
  - Peeking at queued messages
  - Removing messages from the front without receiving
  - Dropping specific messages
- **Simplicity**: Simpler mental model for API users
- **Future-Proofing**: Can add true drop-oldest behavior later with custom queue if needed

## Considered Options

### Option 1: Keep DropOldest and DropNewest (Original)
**Pros:**
- API looks feature-complete
- Room for future custom implementation

**Cons:**
- **Misleading API** - DropOldest doesn't actually drop oldest
- Violates YAGNI principle (§6.1)
- Documentation required to explain limitation
- Users might choose strategy based on false assumptions

### Option 2: Simplify to Single `Drop` Strategy (CHOSEN)
**Pros:**
- ✅ **Honest API** - does what it says
- ✅ **YAGNI Compliant** (§6.1) - only what we can deliver
- ✅ **Simpler** - 3 strategies instead of 4
- ✅ **Clearer semantics** - no ambiguity
- ✅ **Future-proof** - can add drop-oldest later if truly needed

**Cons:**
- Less granular strategy options
- Breaking API change (acceptable since just completed)

### Option 3: Implement Custom Ring Buffer
**Pros:**
- True drop-oldest behavior possible
- Full feature set

**Cons:**
- Significant additional complexity
- Performance overhead vs tokio mpsc
- Premature optimization (no use case yet)
- Violates YAGNI

## Decision Outcome

**Chosen Option**: **Option 2 - Simplify to Single `Drop` Strategy**

### Final API Design

```rust
pub enum BackpressureStrategy {
    /// Block sender until space becomes available (async wait)
    Block,
    
    /// Drop the incoming message when mailbox is full
    Drop,
    
    /// Return an error to the sender immediately
    Error,
}
```

### Priority Mapping

```rust
pub fn for_priority(priority: MessagePriority) -> Self {
    match priority {
        MessagePriority::Critical => Self::Block,  // Must deliver
        MessagePriority::High => Self::Block,      // Important
        MessagePriority::Normal => Self::Error,    // Sender feedback needed
        MessagePriority::Low => Self::Drop,        // Can lose
    }
}
```

## Rationale

1. **Honest Communication**: API accurately reflects implementation capabilities
2. **YAGNI Compliance**: Build only what we can actually deliver now
3. **Simplicity**: Easier mental model for users
4. **Maintainability**: Less code, clearer semantics
5. **Future Flexibility**: Can add drop-oldest strategy later when:
   - Real use case emerges
   - Custom queue implementation is justified
   - Can be added as `DropOldest` variant without breaking existing code

## Consequences

### Positive
- Clear, honest API that does what it says
- Simpler codebase (92 tests instead of 93)
- Better YAGNI compliance (§6.1)
- No misleading documentation required
- Users make informed decisions

### Negative
- Less granular backpressure control
- Breaking change from initial Phase 3 implementation (mitigated: completed same day)

### Neutral
- Future enhancement possible if use case emerges
- Would require custom bounded queue implementation

## Compliance

- ✅ **§6.1 YAGNI**: Only implement what's actually deliverable
- ✅ **§6.3 Microsoft Rust Guidelines**: Honest, simple abstractions
- ✅ **M-DESIGN-FOR-AI**: Clear, unambiguous API design

## Future Considerations

If a real use case for drop-oldest behavior emerges:

1. **Evaluate Need**: Confirm use case requires true drop-oldest semantics
2. **Custom Queue**: Implement custom bounded queue with VecDeque or similar
3. **Add Strategy**: Re-introduce `DropOldest` variant
4. **Performance**: Benchmark custom queue vs tokio mpsc
5. **Compatibility**: Existing `Drop` users unaffected

## References

- **RT-TASK-003**: Mailbox System Implementation
- **KNOWLEDGE-RT-006**: Mailbox System Implementation Guide
- **§6.1**: YAGNI Principles (workspace/shared_patterns.md)
- **§6.3**: Microsoft Rust Guidelines Integration
- **tokio::sync::mpsc Documentation**: Channel API limitations

## Notes

- Decision made during RT-TASK-003 Phase 3 implementation review
- Refactoring completed same day as initial implementation
- No external users affected (task just completed)
- Tests reduced from 93 to 92 (one less duplicate test)
