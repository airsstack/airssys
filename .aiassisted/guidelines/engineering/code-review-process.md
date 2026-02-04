# Code Review Process Guidelines

**Document Type:** Engineering Standard
**Status:** Active
**Scope:** All Code Reviews

## Purpose

This document establishes guidelines for conducting effective code reviews. It synthesizes best practices for reviewing code changes, writing constructive feedback, and resolving disagreements professionally.

---

## Core Standard

The foundational principle of code review is:

> **Approve a change once it definitely improves the overall code health of the system, even if it is not perfect.**

This principle balances:
- **Developer velocity**: Reviews should enable forward momentum, not block progress for minor polish
- **Code quality**: The codebase should continuously improve with each change
- **Team dynamics**: Reviews should be collaborative, not adversarial

---

## Decision-Making Principles

### 1. Technical Facts Over Opinions

Data and evidence should guide decisions over personal preference. When disagreements arise, seek objective technical justification rather than relying on subjective opinions.

### 2. Style Guides Are Authoritative

For style matters, defer to the project's style guide. If no style guide exists, maintain consistency with the existing codebase. Style preferences not covered by the guide are a matter of personal preference.

### 3. Design Decisions Require Engineering Judgment

Design decisions are not style issues. They require evaluation against engineering principles:
- Maintainability
- Testability
- Performance implications
- Security considerations
- Extensibility needs

### 4. Accept Valid Alternatives

If the author demonstrates their approach is valid using data or solid engineering principles, honor their preference. There are often multiple correct ways to solve a problem.

---

## What to Look For

### Design

- Do the code interactions make logical sense?
- Does the change belong in this part of the codebase?
- Does it integrate well with the rest of the system?
- Is this the right time to add this functionality?

### Functionality

- Does the code do what the developer intended?
- Is the behavior good for users (both end-users and future developers)?
- Check edge cases, especially for user-facing changes
- Consider concurrency issues (race conditions, deadlocks)
- Look for potential bugs in the logic

### Complexity

Assess whether code is more complex than necessary:

- **Line level**: Is any single line too hard to understand?
- **Function level**: Is the function doing too much?
- **Class/module level**: Is the abstraction appropriate?

Watch for over-engineering:

> **Solve the problem you know needs to be solved now, not the problem you speculate might need to be solved in the future.**

### Testing

- Are there appropriate unit, integration, or end-to-end tests?
- Will the tests actually fail if the code breaks?
- Do tests have clear, meaningful assertions?
- Are edge cases covered?
- Tests should be maintainable code too

### Naming

- Are names descriptive enough to understand purpose?
- Are names concise without being cryptic?
- Do names follow project conventions?

### Comments

- Comments should explain **why**, not **what**
- If code needs a comment explaining what it does, consider rewriting for clarity
- Are comments accurate and up-to-date?
- Avoid redundant comments that repeat the code

### Style and Consistency

- Does the code follow the project's style guide?
- Is it consistent with surrounding code?
- Style consistency matters more than personal preference

### Documentation

When functionality changes, verify updates to:
- README files
- API documentation
- User guides
- Configuration documentation

---

## How to Navigate a Code Review

### Step 1: Assess the Overall Change

Begin by reading the change description:
- Does this change make sense?
- Is it solving the right problem?
- Should this change exist at all?

If the change should not have happened, respond immediately with an explanation. Be respectful and suggest alternative approaches.

### Step 2: Examine Primary Components First

Identify the "main" part of the change - typically the file with the most significant logical modifications. This provides context for understanding smaller changes.

**Communicate major design issues immediately**, even if you have not reviewed the rest. Reasons:
- Developers often start new work based on pending changes
- Catching design problems early prevents cascading rework
- Major redesigns take time; early feedback helps meet deadlines

### Step 3: Review Remaining Files Systematically

After addressing major concerns:
- Review remaining files in logical order
- Consider reviewing tests first to understand intended behavior
- Check test files to verify coverage

---

## Writing Review Comments

### Maintain a Constructive Tone

- Focus on the **code**, not the developer
- Describe the impact, not the author's choices
- Be direct but courteous

**Instead of:**
> "Why did you do it this way?"

**Write:**
> "This approach adds complexity without clear benefits. Consider [alternative]."

### Explain Your Reasoning

Developers benefit from understanding the **why** behind comments:
- Reference best practices or documentation
- Explain how the change improves code quality
- Share context about intent or future maintenance

Not every comment needs extensive justification, but context helps.

### Balance Guidance with Problem Identification

- Do not redesign solutions or write code for developers
- Point out problems; let developers find solutions
- This facilitates learning better than prescriptive answers
- Direct instructions are appropriate when necessary for quality

### Use Severity Labels

Clarify comment importance to prevent misunderstanding:

| Label | Meaning |
|-------|---------|
| **Nit:** | Minor suggestion, optional to address |
| **Optional:** | Consider this, but not required |
| **FYI:** | Educational information, no action needed |
| *(no label)* | Should be addressed before approval |
| **Blocking:** | Must be fixed before merge |

### Handle Unclear Code Appropriately

When code requires explanation:
- The solution is typically rewriting for clarity, not adding review comments
- Future readers will not see code review discussions
- If explanation is needed, it should be in the code (as comments or clearer code)

---

## Handling Pushback

### Evaluate Disagreements Objectively

When developers push back, first assess whether they have a valid point:

> **Developers are often closer to the code than reviewers. They may have better insight about certain aspects.**

If their argument holds merit from a code quality standpoint, acknowledge this and move forward.

### Persist When Appropriate

When you believe improvements are justified:
- Provide thorough explanations showing you understand their position
- Explain the rationale for the requested change
- Be courteous but persistent about improvements that matter

**Remember:** Improving code health happens in small steps. Each review is an opportunity for incremental improvement.

### Managing Concerns About Developer Reactions

Reviewers often worry that insistence on improvements will upset developers. However:
- When feedback is delivered politely, developers typically do not become upset
- Developers often appreciate the support later
- **Tone and approach matter more than the standard itself**

### Avoid "Clean Up Later"

A common conflict: developers request deferring improvements to future work.

**Be cautious:**
- The more time passes after a change is written, the less likely cleanup will happen
- Unless cleanup happens immediately after the current change, it typically never occurs
- If the cleanup cannot happen now, file a tracking issue and link it

### Escalation Path

If conflicts persist:
1. Discuss face-to-face or via video call
2. Seek consensus through technical discussion
3. Escalate to technical leads if needed
4. Do not let changes stall due to unresolved disputes

---

## Approval Guidance

### When to Approve

Approve when the change:
- Improves overall code health
- Functions correctly
- Is well-tested
- Follows project standards
- Does not introduce security vulnerabilities

Approve even if:
- Minor improvements could be made (use "Nit:" comments)
- It is not how you would have written it
- You have educational suggestions (use "FYI:" comments)

### When to Request Changes

Request changes when:
- The change worsens code health
- There are bugs or logic errors
- Tests are missing or inadequate
- Security vulnerabilities exist
- The approach is fundamentally flawed

### When to Reject

Reject outright when:
- The change should not exist (wrong approach to the problem)
- It introduces unwanted features
- It significantly degrades the codebase

Always explain the rejection respectfully and suggest alternatives.

---

## Summary Checklist

### For Reviewers

- [ ] Read the change description and understand the purpose
- [ ] Check if the change should exist at all
- [ ] Review main components first, then remaining files
- [ ] Communicate major design issues immediately
- [ ] Check design, functionality, complexity, tests, naming, comments, style, docs
- [ ] Write constructive comments focused on the code
- [ ] Use severity labels (Nit, Optional, FYI)
- [ ] Explain reasoning behind suggestions
- [ ] Approve once the change improves code health, even if imperfect

### For Authors

- [ ] Keep changes small and focused
- [ ] Write clear change descriptions
- [ ] Respond to all comments, even to acknowledge
- [ ] Do not take feedback personally
- [ ] Ask for clarification if comments are unclear
- [ ] Address or respond to every comment before re-requesting review

---

## References

- [Google Engineering Practices - Code Review Standard](https://google.github.io/eng-practices/review/reviewer/standard.html)
- [Google Engineering Practices - What to Look For](https://google.github.io/eng-practices/review/reviewer/looking-for.html)
- [Google Engineering Practices - Navigating a CL](https://google.github.io/eng-practices/review/reviewer/navigate.html)
- [Google Engineering Practices - Writing Comments](https://google.github.io/eng-practices/review/reviewer/comments.html)
- [Google Engineering Practices - Handling Pushback](https://google.github.io/eng-practices/review/reviewer/pushback.html)
