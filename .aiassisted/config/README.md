# Configuration System

This directory contains the default configuration file that will be installed to `~/.aiassisted/config.toml` during the first-time setup.

## Structure

```
~/.aiassisted/
├── config.toml              # User configuration (created from config.toml.default)
├── templates/               # Default templates (installed from .aiassisted/templates/)
│   ├── skills/
│   │   ├── opencode/
│   │   └── claude/
│   └── agents/
│       ├── opencode/
│       └── claude/
├── cache/                   # Downloaded content cache (future)
└── state/                   # Global state tracking (future)
    └── installed.toml       # Track installed projects
```

## Configuration File

The `config.toml.default` file in this directory serves as the template for user configuration. During installation:

1. `install.sh` creates `~/.aiassisted/` directory
2. Copies `config.toml.default` → `~/.aiassisted/config.toml`
3. Downloads templates to `~/.aiassisted/templates/`

## Configuration Options

See comments in `config.toml.default` for detailed explanation of each option.

### Key Sections

- **`[general]`** - Runtime and verbosity settings
- **`[install]`** - Installation behavior
- **`[templates]`** - Template resolution preferences
- **`[skills]`** - AI skills setup configuration
- **`[update]`** - Update checking behavior
- **`[github]`** - Source repository settings

## User Customization

Users can modify their configuration at `~/.aiassisted/config.toml`:

```bash
# View current configuration
aiassisted config show

# Edit configuration
aiassisted config edit

# Get specific value
aiassisted config get general.default_runtime

# Reset to defaults
aiassisted config reset
```

## Template Cascading

When `aiassisted setup-skills` is run, templates are resolved with the following priority:

1. **Project templates** (`./.aiassisted/templates/`) - If project has custom templates
2. **Global templates** (`~/.aiassisted/templates/`) - Default templates installed with CLI

This allows:
- Most projects use default templates (zero config)
- Specific projects can customize templates (per-project override)
- Teams can commit custom templates to share standards

## TOML Format

We use TOML (Tom's Obvious, Minimal Language) for configuration because:

- ✅ Human-friendly with comments support
- ✅ Easy to parse in shell scripts (no external dependencies)
- ✅ Standard in Rust (Cargo.toml) and Python (pyproject.toml) ecosystems
- ✅ Native support in Bun runtime
- ✅ Clear type system (strings, numbers, booleans, arrays)

## Implementation Notes

### Shell Runtime
Uses simple grep/sed for TOML parsing (see `toml_get()` in `src/shell/core.sh`)

### Python Runtime
Uses `tomllib` (Python 3.11+) or `tomli` (older versions)

### Bun Runtime
Uses native `Bun.TOML.parse()` API

## Version Tracking

This configuration system is part of the `aiassisted` distribution and tracked in:
- `.aiassisted/.version` - Overall content version
- `.aiassisted/FILES.txt` - Manifest with checksums

The config file itself is versioned through git and selective updates.
