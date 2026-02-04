---
description: Getting started guide for Ratatui TUI library. Covers installation, feature flags, project setup, and building your first terminal application.
globs: "**/Cargo.toml,**/*.rs"
---

# Ratatui Getting Started Guide

This guide covers everything you need to start building Terminal User Interface (TUI) applications with Ratatui.

## 1. Installation

### Basic Installation

Add Ratatui to your project using cargo:

```bash
cargo add ratatui
```

Or add directly to `Cargo.toml`:

```toml
[dependencies]
ratatui = "0.30.0"
```

### Default Backend

Ratatui uses **Crossterm** as the default backend. As of Ratatui 0.27.0, backend crates are exported at the root level, eliminating the need for separate backend imports.

### Alternative Backends

Switch to Termion or Termwiz by disabling default features:

```bash
# Termion (Unix-like systems)
cargo add ratatui --no-default-features --features termion

# Termwiz (Facebook's terminal library)
cargo add ratatui --no-default-features --features termwiz
```

## 2. Feature Flags

| Feature | Description | Default |
|---------|-------------|---------|
| `crossterm` | Crossterm backend | Yes |
| `termion` | Termion backend | No |
| `termwiz` | Termwiz backend | No |
| `all-widgets` | Enable all widget features | Yes (v0.30.0+) |
| `widget-calendar` | Calendar widget (requires `time` crate) | Yes (via all-widgets) |
| `serde` | Serialization for styles/colors | No |

### Minimal Build

For faster compile times, disable default features:

```bash
cargo add ratatui --no-default-features --features=crossterm
```

### Serde Support

Enable theme persistence with serde:

```bash
cargo add ratatui --features serde
```

## 3. Project Setup

### Using Templates

The recommended way to start a new project:

```bash
# Install cargo-generate if not already installed
cargo install cargo-generate

# Generate a new project from template
cargo generate ratatui/templates hello-world
```

### Manual Setup

Create a new project and add dependencies:

```bash
cargo new my-tui-app
cd my-tui-app
cargo add ratatui crossterm color-eyre
```

### Recommended Dependencies

```toml
[dependencies]
ratatui = "0.30.0"
crossterm = "0.29.0"
color-eyre = "0.6.3"  # Enhanced error handling
```

## 4. Hello World Application

### Minimal Example

```rust
use ratatui::{DefaultTerminal, Frame};

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    ratatui::run(app)?;
    Ok(())
}

fn app(terminal: &mut DefaultTerminal) -> std::io::Result<()> {
    loop {
        terminal.draw(render)?;
        if crossterm::event::read()?.is_key_press() {
            break Ok(());
        }
    }
}

fn render(frame: &mut Frame) {
    frame.render_widget("hello world", frame.area());
}
```

### Three-Part Pattern

Ratatui applications typically follow this structure:

1. **Main Function**: Initialize error handling, run the app
2. **App Function**: Event loop - draw UI and handle input
3. **Render Function**: Define what to display

### Running the Application

```bash
cargo run
```

Press any key to exit.

## 5. Project Structure

### Recommended Structure for Larger Applications

```
my-tui-app/
├── src/
│   ├── main.rs       # Entry point, composition
│   ├── app.rs        # Application state and logic
│   ├── ui.rs         # Rendering functions
│   ├── event.rs      # Event handling
│   └── widgets/      # Custom widgets
│       └── mod.rs
├── Cargo.toml
└── README.md
```

### Application State

Define a struct to hold application state:

```rust
#[derive(Debug, Default)]
pub struct App {
    pub counter: i32,
    pub running: bool,
}

impl App {
    pub fn new() -> Self {
        Self {
            counter: 0,
            running: true,
        }
    }

    pub fn increment(&mut self) {
        self.counter += 1;
    }

    pub fn decrement(&mut self) {
        self.counter -= 1;
    }

    pub fn quit(&mut self) {
        self.running = false;
    }
}
```

## 6. Event Handling

### Basic Event Loop

```rust
use crossterm::event::{self, Event, KeyCode, KeyEventKind};

fn handle_events(app: &mut App) -> std::io::Result<()> {
    if event::poll(std::time::Duration::from_millis(100))? {
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('q') => app.quit(),
                    KeyCode::Up => app.increment(),
                    KeyCode::Down => app.decrement(),
                    _ => {}
                }
            }
        }
    }
    Ok(())
}
```

### Main Loop with Event Handling

```rust
fn run(terminal: &mut DefaultTerminal, app: &mut App) -> std::io::Result<()> {
    while app.running {
        terminal.draw(|frame| render(frame, app))?;
        handle_events(app)?;
    }
    Ok(())
}
```

## 7. Error Handling

### Using color-eyre

```rust
use color_eyre::eyre::Result;

fn main() -> Result<()> {
    color_eyre::install()?;
    // Application code
    Ok(())
}
```

### Custom Error Types

For larger applications, define custom error types:

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Configuration error: {0}")]
    Config(String),
}
```

## 8. Rust Version Requirements

- **Minimum**: Rust 1.74 or later
- Verify with: `rustc --version`
- Install via [rustup](https://rustup.rs/)

## 9. Next Steps

1. **Learn Widgets**: See [Widgets Guide](ratatui-widgets-guide.md)
2. **Understand Layout**: See [Layout Guide](ratatui-layout-guide.md)
3. **Choose Architecture**: See [Architecture Guide](ratatui-architecture-guide.md)
4. **Explore Examples**: [Ratatui Examples](https://github.com/ratatui/ratatui/tree/main/examples)

## Quick Reference

| Task | Command |
|------|---------|
| Add ratatui | `cargo add ratatui` |
| New project from template | `cargo generate ratatui/templates hello-world` |
| Run application | `cargo run` |
| Build release | `cargo build --release` |

## Resources

- [Tutorials](https://ratatui.rs/tutorials/)
- [Counter App Tutorial](https://ratatui.rs/tutorials/counter-app/)
- [JSON Editor Tutorial](https://ratatui.rs/tutorials/json-editor/)
