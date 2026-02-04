---
description: Guide to the Ratatui ecosystem. Covers Tachyonfx animations, Mousefood embedded support, Ratzilla WebAssembly, and community libraries.
globs: "**/Cargo.toml,**/*.rs"
---

# Ratatui Ecosystem Guide

The Ratatui ecosystem includes libraries that extend its capabilities beyond standard terminal applications. This guide covers the major ecosystem projects.

## 1. Ecosystem Overview

| Project | Purpose | Platform |
|---------|---------|----------|
| **Tachyonfx** | Animations and effects | Terminal |
| **Mousefood** | Embedded graphics backend | Embedded/no_std |
| **Ratzilla** | WebAssembly rendering | Browser |

## 2. Tachyonfx - Animations

Tachyonfx adds visual effects and smooth animations to Ratatui applications.

### Features

- Composable animation effects
- Smooth transitions
- Effect chaining and layering
- Terminal-optimized rendering

### Installation

```toml
[dependencies]
ratatui = "0.30.0"
tachyonfx = "0.9"
```

### Basic Usage

```rust
use ratatui::prelude::*;
use tachyonfx::{fx, Effect, EffectTimer, Interpolation};
use std::time::Duration;

struct App {
    effect: Effect,
    last_tick: std::time::Instant,
}

impl App {
    fn new() -> Self {
        // Create a fade-in effect
        let effect = fx::fade_in(
            Duration::from_millis(500),
            Interpolation::Linear,
        );

        Self {
            effect,
            last_tick: std::time::Instant::now(),
        }
    }

    fn tick(&mut self) {
        let elapsed = self.last_tick.elapsed();
        self.last_tick = std::time::Instant::now();
        self.effect.process(elapsed);
    }
}

fn render(frame: &mut Frame, app: &App) {
    let paragraph = Paragraph::new("Animated Text")
        .style(Style::default().fg(Color::Cyan));

    // Render with effect
    frame.render_widget(paragraph, frame.area());

    // Apply effect to buffer
    app.effect.apply(frame.buffer_mut(), frame.area());
}
```

### Common Effects

```rust
use tachyonfx::{fx, Interpolation};
use std::time::Duration;

// Fade effects
let fade_in = fx::fade_in(Duration::from_millis(500), Interpolation::Linear);
let fade_out = fx::fade_out(Duration::from_millis(500), Interpolation::Linear);

// Slide effects
let slide_in = fx::slide_in(
    Duration::from_millis(300),
    Interpolation::EaseOut,
    fx::Direction::Left,
);

// Color transitions
let color_cycle = fx::color_cycle(
    Duration::from_secs(2),
    vec![Color::Red, Color::Green, Color::Blue],
);

// Combining effects
let combined = fx::sequence(vec![
    fx::fade_in(Duration::from_millis(200), Interpolation::Linear),
    fx::pause(Duration::from_millis(1000)),
    fx::fade_out(Duration::from_millis(200), Interpolation::Linear),
]);
```

### Effect Composition

```rust
// Parallel effects
let parallel = fx::parallel(vec![
    fx::fade_in(Duration::from_millis(500), Interpolation::Linear),
    fx::slide_in(Duration::from_millis(500), Interpolation::EaseOut, fx::Direction::Up),
]);

// Sequential effects
let sequence = fx::sequence(vec![
    fx::fade_in(Duration::from_millis(300), Interpolation::Linear),
    fx::pause(Duration::from_secs(1)),
    fx::fade_out(Duration::from_millis(300), Interpolation::Linear),
]);

// Loop effects
let looped = fx::repeat(
    fx::color_cycle(Duration::from_secs(1), colors),
    None, // Loop forever
);
```

### Interactive Demo

Try effects in the browser: [Tachyonfx FTL](https://junkdog.github.io/tachyonfx-ftl/)

### Resources

- [Tachyonfx GitHub](https://github.com/junkdog/tachyonfx)
- [API Documentation](https://docs.rs/tachyonfx/)

## 3. Mousefood - Embedded Graphics

Mousefood enables Ratatui on embedded displays using the `embedded-graphics` ecosystem.

### Features

- `no_std` compatible
- Works with any `embedded-graphics` display
- Enables TUI on embedded devices
- Resource-constrained optimization

### Installation

```toml
[dependencies]
ratatui = { version = "0.30.0", default-features = false }
mousefood = "0.1"
```

### Basic Usage

```rust
use ratatui::prelude::*;
use mousefood::MousefoodBackend;
use embedded_graphics::prelude::*;

// With any embedded-graphics display
fn setup_tui<D: DrawTarget>(display: D) -> Terminal<MousefoodBackend<D>> {
    let backend = MousefoodBackend::new(display);
    Terminal::new(backend).unwrap()
}

fn render<D: DrawTarget>(terminal: &mut Terminal<MousefoodBackend<D>>) {
    terminal.draw(|frame| {
        let widget = Paragraph::new("Embedded TUI!")
            .style(Style::default().fg(Color::White));
        frame.render_widget(widget, frame.area());
    }).unwrap();
}
```

### Supported Platforms

- **Embedded displays**: Any `embedded-graphics` compatible display
- **PSP**: PlayStation Portable via `rust-psp`
- **LED matrices**: Various LED panel projects
- **E-ink displays**: E-paper screens

### Example Projects

- **Tuitar**: Guitar learning tool built with Ratatui + Mousefood
- **PSP TUI**: Ratatui running on PlayStation Portable

### Resources

- [Mousefood GitHub](https://github.com/ratatui/mousefood)
- [embedded-graphics](https://github.com/embedded-graphics/embedded-graphics)

## 4. Ratzilla - WebAssembly

Ratzilla brings Ratatui to the browser via WebAssembly.

### Features

- Terminal-themed web applications
- Rust + WebAssembly
- Browser-native deployment
- No terminal required

### Installation

```toml
[dependencies]
ratatui = "0.30.0"
ratzilla = "0.1"
```

### Basic Usage

```rust
use ratatui::prelude::*;
use ratzilla::prelude::*;

fn main() {
    ratzilla::run(|terminal| {
        terminal.draw(|frame| {
            let widget = Paragraph::new("Hello from the browser!")
                .style(Style::default().fg(Color::Green));
            frame.render_widget(widget, frame.area());
        }).unwrap();
    });
}
```

### Building for Web

```bash
# Install wasm-pack
cargo install wasm-pack

# Build for web
wasm-pack build --target web

# Or use trunk for development
cargo install trunk
trunk serve
```

### Project Structure

```
my-ratzilla-app/
├── src/
│   └── lib.rs
├── Cargo.toml
├── index.html
└── style.css
```

### HTML Integration

```html
<!DOCTYPE html>
<html>
<head>
    <style>
        #terminal {
            width: 100%;
            height: 100vh;
            font-family: monospace;
        }
    </style>
</head>
<body>
    <div id="terminal"></div>
    <script type="module">
        import init from './pkg/my_app.js';
        init();
    </script>
</body>
</html>
```

### Live Demo

See Ratzilla in action: [Ratzilla Demo](https://ratatui.github.io/ratzilla/demo/)

### Resources

- [Ratzilla GitHub](https://github.com/ratatui/ratzilla)
- [API Documentation](https://docs.rs/ratzilla/)

## 5. Community Libraries

### Input Handling

| Library | Purpose |
|---------|---------|
| [tui-textarea](https://github.com/rhysd/tui-textarea) | Multi-line text editor |
| [tui-input](https://github.com/sayanarijit/tui-input) | Single-line text input |
| [tui-prompts](https://github.com/joshka/tui-prompts) | Interactive prompts |

### Widgets

| Library | Purpose |
|---------|---------|
| [tui-tree-widget](https://github.com/EdJoPaTo/tui-rs-tree-widget) | Tree view widget |
| [ratatui-image](https://github.com/benjajaja/ratatui-image) | Image rendering |
| [tui-big-text](https://github.com/joshka/tui-big-text) | Large ASCII text |
| [tui-scrollview](https://github.com/joshka/tui-scrollview) | Scrollable views |

### Templates & Generators

| Tool | Purpose |
|------|---------|
| [cargo-generate templates](https://github.com/ratatui/templates) | Project scaffolding |
| [ratatui-template](https://github.com/ratatui/ratatui-template) | Comprehensive starter |

### Example Usage

```rust
// tui-textarea
use tui_textarea::TextArea;

let mut textarea = TextArea::default();
textarea.insert_str("Hello, World!");

frame.render_widget(textarea.widget(), area);

// tui-input
use tui_input::Input;

let input = Input::default().with_value("Initial text");
frame.render_widget(input, area);

// ratatui-image
use ratatui_image::{protocol::StatefulProtocol, StatefulImage};

let image = StatefulImage::new(None);
frame.render_stateful_widget(image, area, &mut image_state);
```

## 6. Awesome Ratatui

The [Awesome Ratatui](https://github.com/ratatui/awesome-ratatui) repository maintains a curated list of:

- Applications built with Ratatui
- Libraries and extensions
- Tutorials and learning resources
- Tools and utilities

### Categories

- **Applications**: Real-world apps built with Ratatui
- **Widgets**: Custom widget implementations
- **Themes**: Color schemes and styling
- **Examples**: Learning and reference code
- **Articles**: Blog posts and tutorials

## 7. Contributing to the Ecosystem

### Starting a New Project

1. Use official templates: `cargo generate ratatui/templates`
2. Follow Ratatui conventions
3. Document your API
4. Add examples
5. Submit to Awesome Ratatui

### Best Practices

- Make widgets generic over `Backend`
- Support both `Widget` and `WidgetRef` traits
- Provide sensible defaults
- Include comprehensive examples
- Follow semantic versioning

## Resources

- [Ratatui Discord](https://discord.gg/pMCEU9hNEj)
- [Ratatui GitHub Discussions](https://github.com/ratatui/ratatui/discussions)
- [Awesome Ratatui](https://github.com/ratatui/awesome-ratatui)
