# Ratatui Guidelines

This directory contains guidelines for building Terminal User Interface (TUI) applications with [Ratatui](https://ratatui.rs/), a Rust library for creating rich terminal interfaces.

## Overview

Ratatui is a lightweight Rust library focused on building terminal user interfaces. It provides a rich set of widgets, flexible layout system, and immediate-mode rendering architecture.

## Guidelines Index

| Guide | Description |
|-------|-------------|
| [Getting Started](ratatui-getting-started-guide.md) | Installation, setup, hello world, and project structure |
| [Architecture Patterns](ratatui-architecture-guide.md) | Elm, Component, and Flux architectures for TUI apps |
| [Widgets](ratatui-widgets-guide.md) | Built-in widgets, custom widgets, and stateful widgets |
| [Layout System](ratatui-layout-guide.md) | Constraints, directions, nesting, and responsive design |
| [Rendering](ratatui-rendering-guide.md) | Immediate mode rendering, event loops, and frame handling |
| [Backends](ratatui-backends-guide.md) | Crossterm, Termion, Termwiz backend comparison |
| [Ecosystem](ratatui-ecosystem-guide.md) | Tachyonfx, Mousefood, Ratzilla, and other integrations |

## Quick Start

```bash
# Add ratatui to your project
cargo add ratatui

# Or with specific backend
cargo add ratatui --no-default-features --features termion
```

## Key Concepts

- **Immediate Mode Rendering**: UI is rebuilt every frame based on application state
- **Widgets**: Reusable UI components (Block, Paragraph, List, Table, etc.)
- **Layout**: Constraint-based system using Cassowary solver
- **Backends**: Crossterm (default), Termion, Termwiz for terminal interaction

## Resources

- [Official Documentation](https://ratatui.rs/)
- [API Reference](https://docs.rs/ratatui/)
- [GitHub Repository](https://github.com/ratatui/ratatui)
- [Examples](https://github.com/ratatui/ratatui/tree/main/examples)
- [Awesome Ratatui](https://github.com/ratatui/awesome-ratatui)
