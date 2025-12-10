# Examples

This section provides practical examples demonstrating AirsSys components in real-world scenarios.

## OSL Examples

### Secure File Operations
Examples demonstrating OSL's secure file I/O with ACL policies and audit logging.

[View OSL Examples →](osl-examples.md)

## RT Examples

### Actor Systems
Examples showing how to build actor-based concurrent systems with fault tolerance.

[View RT Examples →](rt-examples.md)

## Integration Examples

### Combined OSL + RT
Examples integrating both components for complete system programming solutions.

Coming soon.

## Running Examples

All examples are included in the repository:

```bash
# Clone the repository
git clone https://github.com/airsstack/airssys
cd airssys

# Run OSL examples
cargo run --example helper_functions_comprehensive
cargo run --example security_middleware_comprehensive
cargo run --example custom_executor_with_macro --features macros

# Run RT examples
cargo run --example actor_basic
cargo run --example supervisor_basic
cargo run --example osl_integration_example
```

## Example Categories

### OSL Categories
- Basic operations (read, write, delete)
- Security middleware configuration
- Custom executor development
- Middleware pipelines
- Logger configuration

### RT Categories
- Basic actor implementation
- Supervision patterns
- Message passing
- System integration
- Performance optimization

## Learning Path

1. **Start simple**: Run `actor_basic` or `basic_usage`
2. **Add security**: Try `security_middleware_comprehensive`
3. **Add supervision**: Try `supervisor_basic`
4. **Integrate**: Try `osl_integration_example`
5. **Customize**: Try `custom_executor_with_macro`

## Example Code

All examples include:
- Detailed inline comments
- Error handling patterns
- Best practices
- Performance considerations
- Integration patterns
