---
description: Guide to Ratatui rendering system. Covers immediate mode rendering, event loops, frames, buffers, and terminal management.
globs: "**/*.rs"
---

# Ratatui Rendering Guide

Ratatui uses **immediate mode rendering**, where the UI is recreated every frame based on application state. This guide covers the rendering system, event handling, and terminal management.

## 1. Immediate Mode Rendering

### Core Concept

Unlike retained mode GUIs where widgets persist between frames, Ratatui rebuilds the entire UI each render cycle:

```rust
loop {
    // UI is completely rebuilt each frame
    terminal.draw(|frame| {
        // All widgets are created and rendered fresh
        let widget = create_widget_from_state(&app);
        frame.render_widget(widget, frame.area());
    })?;

    // Handle events
    handle_events(&mut app)?;
}
```

### Advantages

- **Simplicity**: UI directly reflects application state
- **No Synchronization**: No need to keep widget state in sync
- **Flexibility**: Easy conditional rendering
- **Predictability**: Same state always produces same UI

### Trade-offs

- Widgets are recreated each frame
- Complex widget construction may impact performance
- State must be stored separately from widgets

## 2. The Render Loop

### Basic Structure

```rust
use ratatui::DefaultTerminal;
use crossterm::event::{self, Event};

fn run(terminal: &mut DefaultTerminal, app: &mut App) -> std::io::Result<()> {
    loop {
        // 1. Draw the UI
        terminal.draw(|frame| render(frame, app))?;

        // 2. Check for events
        if event::poll(std::time::Duration::from_millis(100))? {
            // 3. Handle events
            if let Event::Key(key) = event::read()? {
                if handle_key(app, key) {
                    break;
                }
            }
        }

        // 4. Check exit condition
        if app.should_quit {
            break;
        }
    }
    Ok(())
}
```

### Event-Driven Rendering

Only redraw when necessary:

```rust
fn run(terminal: &mut DefaultTerminal, app: &mut App) -> std::io::Result<()> {
    let mut needs_redraw = true;

    loop {
        if needs_redraw {
            terminal.draw(|frame| render(frame, app))?;
            needs_redraw = false;
        }

        if event::poll(std::time::Duration::from_millis(100))? {
            let event = event::read()?;

            // Mark redraw needed if state changes
            if handle_event(app, event)? {
                needs_redraw = true;
            }
        }

        if app.should_quit {
            break;
        }
    }
    Ok(())
}
```

## 3. Terminal and Frame

### Terminal Initialization

Modern approach (v0.30.0+):

```rust
fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    // Simple initialization
    ratatui::run(app)?;

    Ok(())
}

fn app(terminal: &mut DefaultTerminal) -> std::io::Result<()> {
    // Your application code
    Ok(())
}
```

Manual initialization:

```rust
use ratatui::prelude::*;

fn main() -> std::io::Result<()> {
    // Initialize
    let mut terminal = ratatui::init();

    // Run app
    let result = run(&mut terminal);

    // Restore terminal
    ratatui::restore();

    result
}
```

### The Frame Type

`Frame` provides methods to render widgets:

```rust
fn render(frame: &mut Frame, app: &App) {
    // Get available area
    let area = frame.area();

    // Render widgets
    frame.render_widget(widget, area);

    // Render stateful widgets
    frame.render_stateful_widget(list, area, &mut app.list_state);

    // Set cursor position (for text input)
    frame.set_cursor_position((x, y));
}
```

### Frame Methods

| Method | Purpose |
|--------|---------|
| `area()` | Get the full terminal area |
| `render_widget()` | Render a widget |
| `render_stateful_widget()` | Render a stateful widget |
| `set_cursor_position()` | Position the cursor |
| `buffer_mut()` | Direct buffer access |

## 4. The Buffer

Widgets render to a `Buffer`:

```rust
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

impl Widget for MyWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Write a single cell
        buf[(area.x, area.y)]
            .set_char('X')
            .set_fg(Color::Red);

        // Write a string
        buf.set_string(area.x, area.y, "Hello", Style::default());

        // Set a span
        buf.set_span(area.x, area.y, &span, span.width() as u16);

        // Fill area with style
        buf.set_style(area, Style::default().bg(Color::Blue));
    }
}
```

### Buffer Methods

```rust
// Get/set individual cells
let cell = &buf[(x, y)];
buf[(x, y)].set_char('A');

// String operations
buf.set_string(x, y, "text", style);
buf.set_stringn(x, y, "text", max_width, style);
buf.set_line(x, y, &line, width);
buf.set_span(x, y, &span, width);

// Style operations
buf.set_style(area, style);

// Get content
let content = buf.content();
```

## 5. Conditional Rendering

### Show/Hide Widgets

```rust
fn render(frame: &mut Frame, app: &App) {
    if app.show_sidebar {
        let layout = Layout::horizontal([
            Constraint::Length(20),
            Constraint::Min(0),
        ]).split(frame.area());

        frame.render_widget(sidebar, layout[0]);
        frame.render_widget(main, layout[1]);
    } else {
        frame.render_widget(main, frame.area());
    }
}
```

### Different Views

```rust
fn render(frame: &mut Frame, app: &App) {
    match app.current_view {
        View::Home => render_home(frame, app),
        View::Settings => render_settings(frame, app),
        View::Help => render_help(frame, app),
    }
}
```

### Popups and Overlays

```rust
fn render(frame: &mut Frame, app: &App) {
    // Render main content
    frame.render_widget(main_content, frame.area());

    // Overlay popup if active
    if app.show_popup {
        let popup_area = centered_rect(60, 20, frame.area());

        // Clear the popup area
        frame.render_widget(Clear, popup_area);

        // Render popup
        frame.render_widget(popup, popup_area);
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::vertical([
        Constraint::Percentage((100 - percent_y) / 2),
        Constraint::Percentage(percent_y),
        Constraint::Percentage((100 - percent_y) / 2),
    ]).split(area);

    Layout::horizontal([
        Constraint::Percentage((100 - percent_x) / 2),
        Constraint::Percentage(percent_x),
        Constraint::Percentage((100 - percent_x) / 2),
    ]).split(popup_layout[1])[1]
}
```

## 6. Event Handling

### Basic Event Loop

```rust
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};

fn handle_events(app: &mut App) -> std::io::Result<bool> {
    if event::poll(std::time::Duration::from_millis(100))? {
        match event::read()? {
            Event::Key(key) if key.kind == KeyEventKind::Press => {
                return handle_key_press(app, key);
            }
            Event::Mouse(mouse) => {
                handle_mouse(app, mouse);
            }
            Event::Resize(width, height) => {
                app.on_resize(width, height);
            }
            _ => {}
        }
    }
    Ok(false)
}

fn handle_key_press(app: &mut App, key: KeyEvent) -> std::io::Result<bool> {
    // Check for Ctrl+C
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
        return Ok(true); // Signal quit
    }

    match key.code {
        KeyCode::Char('q') => Ok(true),
        KeyCode::Up => {
            app.previous();
            Ok(false)
        }
        KeyCode::Down => {
            app.next();
            Ok(false)
        }
        KeyCode::Enter => {
            app.select();
            Ok(false)
        }
        _ => Ok(false),
    }
}
```

### Mouse Events

```rust
use crossterm::event::{MouseEvent, MouseEventKind, MouseButton};

fn handle_mouse(app: &mut App, mouse: MouseEvent) {
    match mouse.kind {
        MouseEventKind::Down(MouseButton::Left) => {
            app.handle_click(mouse.column, mouse.row);
        }
        MouseEventKind::ScrollUp => {
            app.scroll_up();
        }
        MouseEventKind::ScrollDown => {
            app.scroll_down();
        }
        _ => {}
    }
}
```

### Input Modes

```rust
enum InputMode {
    Normal,
    Editing,
}

fn handle_key(app: &mut App, key: KeyEvent) -> bool {
    match app.input_mode {
        InputMode::Normal => handle_normal_mode(app, key),
        InputMode::Editing => handle_editing_mode(app, key),
    }
}

fn handle_editing_mode(app: &mut App, key: KeyEvent) -> bool {
    match key.code {
        KeyCode::Esc => {
            app.input_mode = InputMode::Normal;
        }
        KeyCode::Enter => {
            app.submit_input();
            app.input_mode = InputMode::Normal;
        }
        KeyCode::Char(c) => {
            app.input.push(c);
        }
        KeyCode::Backspace => {
            app.input.pop();
        }
        _ => {}
    }
    false
}
```

## 7. Terminal Modes

### Raw Mode

Enables direct input without line buffering:

```rust
use crossterm::terminal::{enable_raw_mode, disable_raw_mode};

fn main() -> std::io::Result<()> {
    enable_raw_mode()?;
    // ... run application ...
    disable_raw_mode()?;
    Ok(())
}
```

### Alternate Screen

Uses separate screen buffer, preserving original terminal content:

```rust
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::execute;
use std::io::stdout;

fn main() -> std::io::Result<()> {
    execute!(stdout(), EnterAlternateScreen)?;
    // ... run application ...
    execute!(stdout(), LeaveAlternateScreen)?;
    Ok(())
}
```

### Mouse Capture

Enable mouse event capture:

```rust
use crossterm::event::{EnableMouseCapture, DisableMouseCapture};

fn main() -> std::io::Result<()> {
    execute!(stdout(), EnableMouseCapture)?;
    // ... run application ...
    execute!(stdout(), DisableMouseCapture)?;
    Ok(())
}
```

## 8. Performance Tips

### Minimize Allocations

```rust
// Reuse buffers when possible
struct App {
    items: Vec<String>,
    list_items: Vec<ListItem<'static>>, // Cache converted items
}
```

### Efficient Updates

```rust
// Only update changed parts
if app.data_changed {
    app.rebuild_list_items();
    app.data_changed = false;
}
```

### Limit Redraws

```rust
// Don't redraw if nothing changed
let mut last_state_hash = 0;

loop {
    let current_hash = app.state_hash();
    if current_hash != last_state_hash {
        terminal.draw(|f| render(f, &app))?;
        last_state_hash = current_hash;
    }

    handle_events(&mut app)?;
}
```

## 9. Error Handling in Render Loop

### Graceful Terminal Restoration

```rust
fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let mut terminal = ratatui::init();

    // Use panic hook to restore terminal
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic| {
        ratatui::restore();
        original_hook(panic);
    }));

    let result = run(&mut terminal);

    ratatui::restore();
    result
}
```

## Resources

- [Rendering Concepts](https://ratatui.rs/concepts/rendering/)
- [Event Handling](https://ratatui.rs/concepts/event-handling/)
- [Crossterm Events](https://docs.rs/crossterm/latest/crossterm/event/index.html)
