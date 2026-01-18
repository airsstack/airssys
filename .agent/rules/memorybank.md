---
description: Core rules and principles for the Memory Bank system
---

# Memory Bank Rules

You are part of the **Memory Bank** system. Your actions are governed by strict verification protocols.

## ⚠️ CRITICAL: MANDATORY VERIFICATION

**EVERY REPORT MUST BE VERIFIED.**

You must NEVER accept a report from a generic agent (Planner, Implementer, Auditor) without running it through the **Verifier**.

### The Verification Loop
1. **Receive Report**: Agent finishes work.
2. **Trigger Verifier**: invoke `@memorybank-verifier` (or the `memorybank-verifier` workflow).
3. **Analyze Result**:
    - ✅ **VERIFIED**: Proceed.
    - ⚠️ **PARTIAL**: Ask user/manager for decision.
    - ❌ **REJECTED**: DO NOT PROCEED. Fix issues or re-run.

## Roles

| Role | Responsibility |
|------|----------------|
| **Planner** | Creates implementation plans. Must be seemingly perfect before implementation. |
| **Implementer** | Writes code. Must pass all tests and guidelines. |
| **Auditor** | Double-checks completed work. |
| **Verifier** | The gatekeeper. Checks the checkers. |

## Core Instruction Reference
You MUST refer to and follow: `@[.aiassisted/instructions/multi-project-memory-bank.instructions.md]`
