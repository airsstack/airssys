# Diátaxis Documentation Framework Guidelines

**All sub-projects MUST organize documentation following the Diátaxis framework - a systematic approach to technical documentation authoring.**

## Framework Overview

Diátaxis identifies four distinct needs and four corresponding forms of documentation, organized around user needs:

```
                    PRACTICAL STEPS          THEORETICAL KNOWLEDGE
                    ===============          =====================
LEARNING-ORIENTED   │  TUTORIALS    │       │  EXPLANATION     │
(Study)             │  (Learning)   │       │  (Understanding) │
                    │               │       │                  │
────────────────────┼───────────────┼───────┼──────────────────┼────────
TASK-ORIENTED       │  HOW-TO       │       │  REFERENCE       │
(Work)              │  GUIDES       │       │  (Information)   │
                    │  (Goals)      │       │                  │
                    ═══════════════          =====================
```

## The Four Documentation Types

### 1. TUTORIALS (Learning-Oriented)
- **Purpose**: Learning experiences that take users through practical steps
- **User Need**: "I want to learn by doing something meaningful"
- **Characteristics**:
  - Practical activity with achievable goals
  - Learning-oriented, not completion-oriented
  - Teacher-student relationship (tutorial guides the learner)
  - Focus on acquisition of skills and knowledge
- **Key Principles**:
  - Show where the learner will be going (set expectations)
  - Deliver visible results early and often
  - Maintain narrative of the expected (reassure learner they're on track)
  - Point out what to notice (close learning loops)
  - Ruthlessly minimize explanation (link to it instead)
  - Focus on concrete steps, not abstractions
  - Ignore options and alternatives (stay focused)
  - Aspire to perfect reliability (every step must work)
- **Language**: "We will...", "First, do x. Now do y.", "Notice that...", "You have built..."
- **Example Structure**:
  ```markdown
  # Tutorial: Building Your First Secure File Operation
  
  In this tutorial, we will create a simple application that reads and writes
  files with security middleware. You will learn how to use the OSL framework's
  security features.
  
  ## What You'll Build
  A command-line tool that securely reads and writes configuration files.
  
  ## Step 1: Set Up Your Project
  First, create a new Rust project:
  ```
  cargo new secure-file-app
  cd secure-file-app
  ```
  
  You should see output confirming the project was created...
  ```

### 2. HOW-TO GUIDES (Task-Oriented)
- **Purpose**: Directions that guide readers through problems to achieve specific goals
- **User Need**: "I need to accomplish this specific task"
- **Characteristics**:
  - Goal-oriented and problem-focused
  - Assumes user knows what they want to achieve
  - Serves the work of already-competent users
  - Addresses real-world complexity and use-cases
- **Key Principles**:
  - Address real-world complexity (adaptable to use-cases)
  - Omit the unnecessary (practical usability over completeness)
  - Provide executable instructions (contract: if situation X, then steps Y)
  - Describe logical sequence (ordered in meaningful way)
  - Seek flow (smooth progress through user's thinking patterns)
  - Pay attention to naming (titles say exactly what guide shows)
- **Language**: "This guide shows you how to...", "If you want x, do y.", "Refer to reference guide for..."
- **Example Structure**:
  ```markdown
  # How to Configure Custom RBAC Policies
  
  This guide shows you how to create and configure custom Role-Based Access
  Control (RBAC) policies for your application.
  
  ## Prerequisites
  - Existing OSL application
  - Understanding of your application's permission requirements
  
  ## Steps
  
  ### 1. Define Your Permissions
  Identify the permissions your application needs:
  ```rust
  let read_perm = Permission::new("file:read", "Read file access");
  let write_perm = Permission::new("file:write", "Write file access");
  ```
  
  ### 2. Create Roles
  If you need role hierarchies, configure them now...
  ```

### 3. REFERENCE (Information-Oriented)
- **Purpose**: Technical descriptions of the machinery and how to operate it
- **User Need**: "I need accurate, authoritative information about this"
- **Characteristics**:
  - Information-oriented, describes the product
  - Austere and uncompromising
  - Wholly authoritative (no doubt or ambiguity)
  - Like a map of the territory
  - Structured according to the machinery itself
- **Key Principles**:
  - Describe and only describe (neutral description)
  - Adopt standard patterns (consistency aids effectiveness)
  - Respect structure of machinery (docs mirror code structure)
  - Provide examples (illustrate without distracting)
  - State facts, list features, provide warnings
- **Language**: "Class X inherits from Y...", "Sub-commands are: a, b, c...", "You must use a. Never d."
- **Example Structure**:
  ```markdown
  # API Reference: SecurityMiddleware
  
  ## Module: airssys_osl::middleware::security
  
  ### Struct: SecurityMiddleware
  
  ```rust
  pub struct SecurityMiddleware { /* fields */ }
  ```
  
  A middleware component that enforces security policies.
  
  #### Methods
  
  ##### `new()`
  ```rust
  pub fn new() -> Self
  ```
  Creates a new SecurityMiddleware instance with default configuration.
  
  **Returns**: SecurityMiddleware instance
  
  **Example**:
  ```rust
  let middleware = SecurityMiddleware::new();
  ```
  
  ##### `with_policy()`
  ```rust
  pub fn with_policy<P: SecurityPolicy>(self, policy: P) -> Self
  ```
  Adds a security policy to the middleware.
  
  **Parameters**:
  - `policy`: A type implementing SecurityPolicy trait
  
  **Returns**: Self for method chaining
  ```

### 4. EXPLANATION (Understanding-Oriented)
- **Purpose**: Discursive treatment that deepens understanding
- **User Need**: "I want to understand the context, reasoning, and implications"
- **Characteristics**:
  - Understanding-oriented, permits reflection
  - Deepens and broadens knowledge
  - Higher and wider perspective than other types
  - Makes sense to read away from the product (bath-reading documentation)
  - Brings clarity, context, and connections
- **Key Principles**:
  - Make connections (weave web of understanding)
  - Provide context (background, history, design decisions)
  - Talk about the subject (bigger picture, alternatives, why)
  - Admit opinion and perspective (discuss tradeoffs)
  - Keep closely bounded (prevent scope creep)
- **Language**: "The reason for x is...", "W is better than z because...", "Some prefer w because..."
- **Example Structure**:
  ```markdown
  # Understanding Security Context Architecture
  
  ## Background
  
  The security context architecture in AirsSys OSL emerged from the need to
  separate concerns between operation definition and security enforcement. This
  separation allows operations to declare their permission requirements without
  being coupled to specific security policy implementations.
  
  ## Design Rationale
  
  Historically, many frameworks tightly couple security checks to business logic,
  leading to scattered authorization code. We chose a middleware-based approach
  for several reasons:
  
  1. **Separation of Concerns**: Operations focus on what they do, not who can do it
  2. **Composability**: Multiple security policies can be combined
  3. **Testability**: Security logic can be tested independently
  
  ## Architectural Tradeoffs
  
  The attribute-based approach offers flexibility but introduces complexity...
  
  Some teams prefer declarative security (annotations/attributes) as it keeps
  security visible in code. Our approach favors runtime flexibility, which is
  better suited for systems with dynamic security requirements.
  
  ## Alternative Approaches
  
  Other security architectures we considered include...
  ```

## Documentation Organization Standards

**Directory Structure Following Diátaxis:**
```
{sub-project}/docs/src/
├── SUMMARY.md
├── introduction.md
├── tutorials/              # TUTORIALS - Learning-oriented
│   ├── getting-started.md
│   ├── first-secure-app.md
│   └── building-middleware.md
├── guides/                 # HOW-TO GUIDES - Task-oriented  
│   ├── configure-rbac.md
│   ├── custom-policies.md
│   └── integration.md
├── reference/              # REFERENCE - Information-oriented
│   ├── api/
│   │   ├── core.md
│   │   ├── middleware.md
│   │   └── operations.md
│   └── cli.md
└── explanation/            # EXPLANATION - Understanding-oriented
    ├── architecture.md
    ├── security-model.md
    └── design-decisions.md
```

## Content Placement Guidelines

**When to use TUTORIALS:**
- New users learning the product
- Introducing core concepts through practice
- Building confidence through success
- Teaching fundamental workflows
- Example: "Tutorial: Your First Secure File Operation"

**When to use HOW-TO GUIDES:**
- Solving specific problems
- Accomplishing particular tasks
- Real-world scenarios and use-cases
- Multiple approaches to goals
- Example: "How to Configure Custom Security Policies"

**When to use REFERENCE:**
- API documentation
- Configuration options
- Command-line interface
- Data structures and types
- Error codes and messages
- Example: "API Reference: SecurityMiddleware"

**When to use EXPLANATION:**
- Architecture and design rationale
- Conceptual overviews
- Historical context and evolution
- Comparison of approaches
- Performance characteristics
- Example: "Understanding the Security Context Architecture"

## Quality Checklist for Each Type

**Tutorial Quality:**
- [ ] Clear learning objective stated upfront
- [ ] Every step produces visible result
- [ ] Minimal explanation (links to explanation docs)
- [ ] Concrete examples only (no abstractions)
- [ ] Tested end-to-end reliability
- [ ] Success achievable by following exactly

**How-To Guide Quality:**
- [ ] Specific goal/problem clearly stated
- [ ] Assumes user competence
- [ ] Focuses on practical steps
- [ ] Adaptable to real-world variations
- [ ] Omits unnecessary completeness
- [ ] Title describes exactly what it shows

**Reference Quality:**
- [ ] Neutral, objective description
- [ ] Consistent structure and patterns
- [ ] Complete and accurate information
- [ ] Mirrors code/product structure
- [ ] No instruction or explanation
- [ ] Examples illustrate without teaching

**Explanation Quality:**
- [ ] Provides context and background
- [ ] Makes connections to related topics
- [ ] Discusses alternatives and tradeoffs
- [ ] Admits opinions where appropriate
- [ ] Bounded scope (doesn't try to explain everything)
- [ ] Can be read away from product

## Integration with Existing Standards

Diátaxis complements existing AirsSys documentation requirements:
- **mdBook**: Technical delivery mechanism for Diátaxis content
- **Quality Standards**: Professional tone applied across all four types
- **Memory Bank**: Technical debt, knowledge docs, ADRs map to EXPLANATION category
- **Rustdoc**: Generated API docs fit REFERENCE category
- **README files**: Mix of TUTORIAL (getting started) and REFERENCE (quick facts)

## Migration Strategy

For existing documentation:
1. Audit current docs and categorize by Diátaxis type
2. Identify gaps (missing tutorials, how-tos, explanations, reference)
3. Reorganize content into Diátaxis structure
4. Fill critical gaps (prioritize tutorials and how-to guides)
5. Ensure each type follows its quality principles

**Success Metrics:**

Documentation following Diátaxis should demonstrate:
- Users can get started quickly (effective tutorials)
- Users can solve real problems (useful how-to guides)
- Users can find accurate information (reliable reference)
- Users understand the system deeply (comprehensive explanation)
- Reduced "where do I find X?" questions
- Increased user confidence and satisfaction

## Further Reading
- Official Diátaxis site: https://diataxis.fr/
- Complete framework theory: https://diataxis.fr/theory/
- Quality guidelines: https://diataxis.fr/quality/
