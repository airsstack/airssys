# Tutorials

**Learning-oriented guides that take you through practical exercises step-by-step.**

Tutorials are designed for learning by doing. Each tutorial:
- Has a clear learning objective
- Provides step-by-step instructions
- Delivers visible results early and often
- Focuses on acquisition of skills and knowledge
- Minimizes explanation (links to Explanation docs instead)

## Available Tutorials

### Beginner Tutorials

Start here if you're new to AirsSys-RT:

#### [Getting Started](../implementation/getting-started.md)
Your first steps with the runtime system. Install, configure, and run your first actor.

#### [Your First Actor](../implementation/actor-creation.md)
Create a simple actor from scratch. Learn the Actor trait, message handling, and lifecycle hooks.

#### [Message Handling](../implementation/message-handling.md)
Understand how actors communicate through messages. Implement message types and handlers.

#### [Building a Supervisor Tree](../implementation/supervision-setup.md)
Set up fault-tolerant supervisor hierarchies. Learn supervision strategies and restart policies.

## Learning Path

We recommend following this order:
1. **Getting Started** - Set up your environment
2. **Your First Actor** - Learn Actor fundamentals
3. **Message Handling** - Master actor communication  
4. **Building a Supervisor Tree** - Add fault tolerance

## After Tutorials

Once you've completed these tutorials:
- Explore **[How-To Guides](../guides/actor-development.md)** for task-specific instructions
- Read **[Explanation](../explanation.md)** docs to deepen your understanding
- Reference **[API Documentation](../api.md)** for detailed specifications

## Related Resources

- **Examples**: See `examples/` directory for working code
  - `examples/actor_basic.rs` - Reinforces "Your First Actor"
  - `examples/supervisor_basic.rs` - Reinforces "Building a Supervisor Tree"
  - `examples/README.md` - Complete example catalog with learning progression
- **How-To Guides**: Practical solutions to specific problems
- **Architecture**: System design and component overview

## Diátaxis Framework

This section follows the **Tutorials** category of the [Diátaxis framework](https://diataxis.fr/):
- **Purpose**: Learning experiences through practical steps
- **User Need**: "I want to learn by doing something meaningful"
- **Focus**: Skill acquisition and knowledge building
- **Approach**: Teacher-student relationship with guided exercises
