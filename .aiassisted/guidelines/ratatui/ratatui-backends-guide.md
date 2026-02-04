---
description: Guide to Ratatui terminal backends. Covers Crossterm, Termion, Termwiz comparison, features, and selection criteria.
globs: "**/Cargo.toml,**/*.rs"
---

# Ratatui Backends Guide

Ratatui supports multiple terminal backends for different use cases. This guide covers the available backends, their features, and how to choose between them.

## 1. Available Backends

| Backend | Default | Platforms | Maintenance |
|---------|---------|-----------|-------------|
| **Crossterm** | Yes | Windows, macOS, Linux | Active |
| **Termion** | No | Unix-like (macOS, Linux) | Maintained |
| **Termwiz** | No | Windows, macOS, Linux | Active (Facebook) |
| **TestBackend** | N/A | Testing only | Built-in |

## 2. Crossterm (Default)

Crossterm is the recommended backend for most applications.

### Features

- Cross-platform (Windows, macOS, Linux)
- Pure Rust implementation
- Active development
- Feature-rich API
- Good performance

### Installation

```bash
# Default installation includes Crossterm
cargo add ratatui
```

### Basic Usage

```rust
use ratatui::prelude::*;
use ratatui::crossterm::event::{self, Event, KeyCode};

fn main() -> std::io::Result<()> {
    let mut terminal = ratatui::init();

    loop {
        terminal.draw(|frame| {
            frame.render_widget("Hello Crossterm!", frame.area());
        })?;

        if let Event::Key(key) = event::read()? {
            if key.code == KeyCode::Char('q') {
                break;
            }
        }
    }

    ratatui::restore();
    Ok(())
}
```

### Version Management

Ratatui supports multiple Crossterm versions via feature flags:

```toml
[dependencies]
# Use specific Crossterm version
ratatui = { version = "0.30.0", features = ["crossterm_0_28"] }
# or
ratatui = { version = "0.30.0", features = ["crossterm_0_29"] }
```

**Why this matters**: Different Crossterm major versions have separate event queues and incompatible types, which can cause compilation errors or race conditions.

## 3. Termion

A lightweight alternative for Unix-like systems.

### Features

- Unix-only (macOS, Linux)
- Smaller dependency footprint
- Simple API
- Lower memory usage

### Installation

```bash
cargo add ratatui --no-default-features --features termion
```

### Basic Usage

```rust
use ratatui::prelude::*;
use ratatui::backend::TermionBackend;
use std::io::{self, stdout};
use termion::raw::IntoRawMode;
use termion::input::TermRead;
use termion::screen::IntoAlternateScreen;

fn main() -> io::Result<()> {
    let stdout = stdout()
        .into_raw_mode()?
        .into_alternate_screen()?;
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    loop {
        terminal.draw(|frame| {
            frame.render_widget("Hello Termion!", frame.area());
        })?;

        // Termion uses different event handling
        for key in io::stdin().keys() {
            if let Ok(termion::event::Key::Char('q')) = key {
                return Ok(());
            }
            break;
        }
    }

    Ok(())
}
```

### Considerations

- No Windows support
- Different event handling API than Crossterm
- May be preferred for resource-constrained Unix environments

## 4. Termwiz

Facebook's terminal manipulation library.

### Features

- Cross-platform
- Rich terminal capabilities
- Part of Wezterm ecosystem
- Advanced features (sixel graphics, etc.)

### Installation

```bash
cargo add ratatui --no-default-features --features termwiz
```

### Basic Usage

```rust
use ratatui::prelude::*;
use ratatui::backend::TermwizBackend;
use termwiz::terminal::Terminal as TermwizTerminal;
use termwiz::caps::Capabilities;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let caps = Capabilities::new_from_env()?;
    let tw_terminal = termwiz::terminal::new_terminal(caps)?;
    let backend = TermwizBackend::new(tw_terminal)?;
    let mut terminal = Terminal::new(backend)?;

    terminal.draw(|frame| {
        frame.render_widget("Hello Termwiz!", frame.area());
    })?;

    Ok(())
}
```

### Considerations

- Larger dependency tree
- Different API patterns
- Good for Wezterm integration
- Advanced terminal features

## 5. TestBackend

For unit testing without a real terminal.

### Features

- No terminal required
- Predictable behavior
- Snapshot testing support
- Fast execution

### Usage

```rust
use ratatui::prelude::*;
use ratatui::backend::TestBackend;

#[test]
fn test_render() {
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal.draw(|frame| {
        frame.render_widget(
            Paragraph::new("Hello Test!"),
            frame.area(),
        );
    }).unwrap();

    // Access the buffer for assertions
    let buffer = terminal.backend().buffer();
    assert_eq!(buffer[(0, 0)].symbol(), "H");
}
```

### Snapshot Testing

```rust
#[test]
fn test_ui_snapshot() {
    let backend = TestBackend::new(40, 10);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal.draw(|frame| {
        render_my_widget(frame, frame.area());
    }).unwrap();

    // Compare with expected output
    let expected = Buffer::with_lines(vec![
        "┌──────────────────────────────────────┐",
        "│ My Widget                            │",
        "│                                      │",
        "└──────────────────────────────────────┘",
    ]);

    terminal.backend().assert_buffer(&expected);
}
```

## 6. Backend Comparison

### Feature Matrix

| Feature | Crossterm | Termion | Termwiz |
|---------|-----------|---------|---------|
| Windows | ✓ | ✗ | ✓ |
| macOS | ✓ | ✓ | ✓ |
| Linux | ✓ | ✓ | ✓ |
| Raw Mode | ✓ | ✓ | ✓ |
| Alternate Screen | ✓ | ✓ | ✓ |
| Mouse Support | ✓ | ✓ | ✓ |
| 256 Colors | ✓ | ✓ | ✓ |
| True Color | ✓ | ✓ | ✓ |
| Async Events | ✓ | ✗ | ✓ |

### Performance Characteristics

| Aspect | Crossterm | Termion | Termwiz |
|--------|-----------|---------|---------|
| Compile Time | Medium | Fast | Slow |
| Binary Size | Medium | Small | Large |
| Runtime Speed | Fast | Fast | Fast |
| Memory Usage | Medium | Low | Medium |

## 7. Choosing a Backend

### Use Crossterm (Default) When

- Building cross-platform applications
- Want the most active maintenance
- Need comprehensive features
- Starting a new project

### Use Termion When

- Targeting Unix-only
- Want minimal dependencies
- Need small binary size
- Building resource-constrained applications

### Use Termwiz When

- Need advanced terminal features
- Integrating with Wezterm
- Want Facebook's ecosystem
- Need specific Termwiz features

## 8. Backend Abstraction

### Writing Backend-Agnostic Code

```rust
use ratatui::prelude::*;

// Generic over any backend
fn render<B: Backend>(terminal: &mut Terminal<B>, app: &App) -> std::io::Result<()> {
    terminal.draw(|frame| {
        let widget = Paragraph::new(format!("Count: {}", app.count));
        frame.render_widget(widget, frame.area());
    })?;
    Ok(())
}
```

### Feature Flags for Multiple Backends

```toml
[features]
default = ["crossterm-backend"]
crossterm-backend = ["ratatui/crossterm"]
termion-backend = ["ratatui/termion"]
termwiz-backend = ["ratatui/termwiz"]
```

```rust
#[cfg(feature = "crossterm-backend")]
fn create_terminal() -> std::io::Result<Terminal<CrosstermBackend<Stdout>>> {
    // Crossterm initialization
}

#[cfg(feature = "termion-backend")]
fn create_terminal() -> std::io::Result<Terminal<TermionBackend<...>>> {
    // Termion initialization
}
```

## 9. Common Backend Operations

All backends support these core capabilities:

```rust
// Draw to screen
terminal.draw(|frame| { /* ... */ })?;

// Query terminal size
let size = terminal.size()?;

// Clear screen
terminal.clear()?;

// Show/hide cursor
terminal.show_cursor()?;
terminal.hide_cursor()?;

// Set cursor position
terminal.set_cursor(x, y)?;
```

## Resources

- [Crossterm Docs](https://docs.rs/crossterm/)
- [Termion Docs](https://docs.rs/termion/)
- [Termwiz Docs](https://docs.rs/termwiz/)
- [Backend Comparison](https://ratatui.rs/concepts/backends/)
