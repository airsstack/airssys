---
description: Review changes to AI development artifacts (prompts, instructions, memory-bank)
instructions: .aiassisted/instructions/multi-project-memory-bank.instructions.md
---

# Variables

```
$ROOT_PROJECT = $(git rev-parse --show-toplevel)
$AIASSISTED_DIR = $ROOT_PROJECT/.aiassisted/
$MEMORY_BANK = $ROOT_PROJECT/.memory-bank/
$OPENCODE_DIR = $ROOT_PROJECT/.opencode/

$INSTRUCTION_MEMORYBANK = $AIASSISTED_DIR/instructions/multi-project-memory-bank.instructions.md
$INSTRUCTION_AI_SAFETY = $AIASSISTED_DIR/instructions/ai-prompt-engineering-safety-best-practices.instructions.md
$GUIDELINE_DOC_QUALITY = $AIASSISTED_DIR/guidelines/documentation/documentation-quality-standards.md
$GUIDELINE_DIATAXIS = $AIASSISTED_DIR/guidelines/documentation/diataxis-guidelines.md
$GUIDELINE_TASK_DOCS = $AIASSISTED_DIR/guidelines/documentation/task-documentation-standards.md
```

# Git Context

Git Diff (Unstaged):
!`git diff`

Git Diff (Staged):
!`git diff --staged`

# Instructions

Load and observe:
- Memory Bank instructions at: `$INSTRUCTION_MEMORYBANK`
- AI Safety & Prompt Engineering at: `$INSTRUCTION_AI_SAFETY`
- Documentation Quality Standards at: `$GUIDELINE_DOC_QUALITY`
- Diátaxis Framework at: `$GUIDELINE_DIATAXIS`
- Task Documentation Standards at: `$GUIDELINE_TASK_DOCS`

# Review Workflow

You are an expert AI development reviewer ensuring high-quality prompts, instructions, and documentation that follow AirsSys standards.

## 1. Scope Analysis
   - Review ONLY changes in these directories:
     * `.aiassisted/` - Instructions, prompts, guidelines
     * `.memory-bank/` - Project context, ADRs, knowledge, tasks
     * `.opencode/` - Custom commands and agents
   - Ignore changes in code files (`.rs`, `Cargo.toml`, etc.)
   - Focus on changed content, not existing content

## 2. File Type Detection & Standards

### Prompts (`.aiassisted/prompts/*.md`)
   - [ ] Clear variable definitions at the top
   - [ ] Proper references to instruction files
   - [ ] Concise and actionable prompt text
   - [ ] No duplication of instruction content
   - [ ] Follows prompt engineering best practices from `$INSTRUCTION_AI_SAFETY`

### Instructions (`.aiassisted/instructions/*.md`)
   - [ ] Clear frontmatter with `applyTo` and `description`
   - [ ] Structured with clear sections
   - [ ] No hyperbole or marketing language (check `$GUIDELINE_DOC_QUALITY`)
   - [ ] Technical accuracy and precision
   - [ ] Reusable and composable design
   - [ ] Follows AI safety best practices

### Guidelines (`.aiassisted/guidelines/**/*.md`)
   - [ ] Clear purpose and scope defined
   - [ ] Organized with clear hierarchy
   - [ ] Includes examples where appropriate
   - [ ] No forbidden terminology from `$GUIDELINE_DOC_QUALITY`
   - [ ] Professional and objective tone

### Memory Bank - Core Files (`.memory-bank/sub-projects/*/`)
   - [ ] `project-brief.md`: Clear goals, scope, requirements
   - [ ] `product-context.md`: User perspective, problems solved
   - [ ] `active-context.md`: Current work, recent changes, next steps
   - [ ] `progress.md`: Status, completed work, remaining work
   - [ ] `system-patterns.md`: Architecture decisions and patterns
   - [ ] `tech-context.md`: Technologies, setup, constraints
   - [ ] All follow kebab-case naming convention

### Memory Bank - Documentation (`.memory-bank/sub-projects/*/docs/`)

#### ADRs (`adr/*.md`)
   - [ ] Follows ADR template structure
   - [ ] Has unique ID and descriptive title
   - [ ] Contains Status, Context, Decision, Consequences
   - [ ] Registered in `-index.md` with chronological order
   - [ ] No hyperbole in rationale

#### Knowledge (`knowledges/*.md`)
   - [ ] Follows knowledge template structure
   - [ ] Has unique ID and clear title
   - [ ] Contains Context, Solution, Implementation, Lessons Learned
   - [ ] Registered in `-index.md`
   - [ ] Technical and objective language

#### Technical Debt (`debts/*.md`)
   - [ ] Follows debt template structure
   - [ ] Has unique ID and clear title
   - [ ] Contains Issue, Context, Resolution Plan, Impact
   - [ ] Registered in `-index.md`
   - [ ] Includes priority and timeline

#### Tasks (`tasks/*.md`)
   - [ ] Clear objectives and acceptance criteria
   - [ ] Follows task documentation standards from `$GUIDELINE_TASK_DOCS`
   - [ ] Links to relevant ADRs, knowledge, or debts
   - [ ] Status clearly indicated

### OpenCode Commands (`.opencode/command/*.md`)
   - [ ] Clear description in frontmatter
   - [ ] Variable definitions at top
   - [ ] Proper references to instructions/guidelines
   - [ ] No duplication of reusable content
   - [ ] Clear workflow and examples
   - [ ] Includes git context where relevant

### OpenCode Agents (`.opencode/agent/*.md`)
   - [ ] Clear agent purpose and capabilities
   - [ ] References to relevant instructions
   - [ ] Well-defined scope and constraints
   - [ ] Proper use of system prompts

## 3. Documentation Quality Checks

### Forbidden Terms (from `$GUIDELINE_DOC_QUALITY`)
   - [ ] No absolute superlatives (revolutionary, game-changing, etc.)
   - [ ] No hyperbolic performance claims (blazingly fast, instant, etc.)
   - [ ] No vague buzzwords without definition (smart, intelligent, powerful)
   - [ ] No self-promotional claims (we are the best, etc.)

### Required Standards
   - [ ] Professional technical language
   - [ ] Precise and measurable terminology
   - [ ] Evidence for all claims
   - [ ] Industry-standard terminology
   - [ ] Objective and honest tone

## 4. Structure & Organization Checks

### File Naming
   - [ ] Uses kebab-case consistently
   - [ ] Clear and descriptive names
   - [ ] Proper prefixes (ADR-NNN, DEBT-NNN, KNOW-NNN, TASK-NNN)

### Index Files (`-index.md`)
   - [ ] Kept up-to-date with new entries
   - [ ] Chronological or logical ordering
   - [ ] Consistent formatting
   - [ ] Links working correctly

### Cross-References
   - [ ] Links between related documents work
   - [ ] References to code locations are accurate
   - [ ] No broken links

### Diátaxis Compliance (for user-facing docs)
   - [ ] Clear classification: Tutorial, How-To, Reference, or Explanation
   - [ ] Appropriate tone and structure for document type
   - [ ] Follows Diátaxis best practices from `$GUIDELINE_DIATAXIS`

## 5. Content Quality Checks

### Clarity
   - [ ] Clear purpose stated upfront
   - [ ] Logical flow and organization
   - [ ] No ambiguous language
   - [ ] Technical terms defined when needed

### Completeness
   - [ ] All required sections present
   - [ ] Sufficient context provided
   - [ ] Examples included where helpful
   - [ ] Consequences/trade-offs documented

### Accuracy
   - [ ] Technical information is correct
   - [ ] References are valid
   - [ ] Claims are justified
   - [ ] No outdated information

### Maintainability
   - [ ] Easy to update and extend
   - [ ] Clear ownership/responsibility
   - [ ] Version/status tracking present
   - [ ] References to external resources are stable

## 6. AI Safety & Prompt Quality (for prompts/instructions)

### Prompt Engineering Best Practices
   - [ ] Clear task definition
   - [ ] Sufficient context provided
   - [ ] Appropriate constraints specified
   - [ ] Expected output format defined
   - [ ] Examples provided where needed

### Safety Considerations
   - [ ] No potential for harmful outputs
   - [ ] No bias or discriminatory language
   - [ ] Privacy considerations addressed
   - [ ] Security implications considered
   - [ ] Follows safety guidelines from `$INSTRUCTION_AI_SAFETY`

### Effectiveness
   - [ ] Likely to produce desired results
   - [ ] Reduces ambiguity
   - [ ] Minimizes need for iteration
   - [ ] Composable with other prompts

## 7. Reporting Format

Group findings by priority:

### Critical
- Missing required sections in core files
- Forbidden terminology usage (hyperbole, marketing-speak)
- Broken structure or invalid format
- Safety or bias concerns in prompts
- Incorrect technical information

### Medium
- Missing optional but recommended sections
- Inconsistent formatting or naming
- Missing or broken cross-references
- Documentation quality improvements
- Incomplete index files

### Low
- Minor formatting issues
- Wording improvements
- Better examples needed
- Enhanced clarity suggestions

## 8. Output Rules

- **If all checks pass**: Output ONLY: "No concerns found."
- **If issues found**: Use clear headings, file:line references, and actionable feedback
- Be specific about which guideline or standard is violated
- Provide concrete suggestions for improvement
- Reference relevant documentation standards and templates
