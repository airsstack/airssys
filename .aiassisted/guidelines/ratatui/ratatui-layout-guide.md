---
description: Complete guide to Ratatui layout system. Covers constraints, directions, nesting, flex properties, and responsive design patterns.
globs: "**/*.rs"
---

# Ratatui Layout Guide

The Ratatui layout system provides a flexible, constraint-based approach to organizing UI elements using the Cassowary algorithm.

## 1. Coordinate System

Ratatui uses a standard terminal coordinate system:

- **Origin**: Top-left corner at `(0, 0)`
- **X-axis**: Increases left to right (columns)
- **Y-axis**: Increases top to bottom (rows)
- **Type**: All coordinates use `u16`

```
(0,0) ─────────────────────► X (columns)
  │
  │
  │
  │
  ▼
  Y (rows)
```

## 2. The Rect Type

All layout operations work with `Rect`:

```rust
use ratatui::layout::Rect;

let rect = Rect {
    x: 0,      // Column position
    y: 0,      // Row position
    width: 80, // Width in columns
    height: 24, // Height in rows
};

// Useful methods
let area = frame.area();           // Full terminal area
let inner = block.inner(area);     // Area inside block borders
```

## 3. Basic Layout Usage

### Creating Layouts

```rust
use ratatui::layout::{Layout, Direction, Constraint};

let layout = Layout::default()
    .direction(Direction::Vertical)
    .constraints([
        Constraint::Length(3),
        Constraint::Min(0),
        Constraint::Length(1),
    ])
    .split(frame.area());

// Access resulting rectangles
let header = layout[0];
let content = layout[1];
let footer = layout[2];
```

### Directions

```rust
// Vertical: divides space top to bottom
Layout::default().direction(Direction::Vertical)

// Horizontal: divides space left to right
Layout::default().direction(Direction::Horizontal)
```

## 4. Constraint Types

| Constraint | Description | Use Case |
|------------|-------------|----------|
| `Length(u16)` | Fixed size in cells | Headers, footers, fixed panels |
| `Percentage(u16)` | Percentage of parent | Proportional layouts |
| `Ratio(u16, u16)` | Fractional allocation | Precise proportions |
| `Min(u16)` | Minimum size threshold | Flexible with minimum |
| `Max(u16)` | Maximum size threshold | Flexible with cap |
| `Fill(u16)` | Fill remaining space | Flexible content areas |

### Length Constraint

Fixed absolute size:

```rust
// Fixed 3-row header
Constraint::Length(3)
```

### Percentage Constraint

Relative to parent:

```rust
// 50% of available space
Constraint::Percentage(50)
```

**Note**: Percentages are relative to the parent layout, not the terminal.

### Ratio Constraint

Fractional allocation:

```rust
// One third of space
Constraint::Ratio(1, 3)

// Two thirds of space
Constraint::Ratio(2, 3)
```

### Min/Max Constraints

Flexible with bounds:

```rust
// At least 10 rows, grows if available
Constraint::Min(10)

// At most 20 rows, shrinks if needed
Constraint::Max(20)
```

### Fill Constraint

Takes remaining space with weight:

```rust
// Fill remaining equally
vec![Constraint::Fill(1), Constraint::Fill(1)]

// Fill with 2:1 ratio
vec![Constraint::Fill(2), Constraint::Fill(1)]
```

## 5. Common Layout Patterns

### Header/Content/Footer

```rust
let layout = Layout::default()
    .direction(Direction::Vertical)
    .constraints([
        Constraint::Length(3),  // Header
        Constraint::Min(0),     // Content (fills remaining)
        Constraint::Length(1),  // Footer
    ])
    .split(frame.area());
```

### Sidebar Layout

```rust
let layout = Layout::default()
    .direction(Direction::Horizontal)
    .constraints([
        Constraint::Length(20), // Sidebar
        Constraint::Min(0),     // Main content
    ])
    .split(frame.area());
```

### Three Column Layout

```rust
let layout = Layout::default()
    .direction(Direction::Horizontal)
    .constraints([
        Constraint::Percentage(25),
        Constraint::Percentage(50),
        Constraint::Percentage(25),
    ])
    .split(frame.area());
```

### Equal Split

```rust
// Equal vertical thirds
let layout = Layout::default()
    .direction(Direction::Vertical)
    .constraints([
        Constraint::Ratio(1, 3),
        Constraint::Ratio(1, 3),
        Constraint::Ratio(1, 3),
    ])
    .split(frame.area());

// Or using Fill
let layout = Layout::default()
    .direction(Direction::Vertical)
    .constraints([Constraint::Fill(1); 3])
    .split(frame.area());
```

## 6. Nested Layouts

Compose complex layouts by nesting:

```rust
fn render(frame: &mut Frame) {
    // Outer layout: vertical split
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
        ])
        .split(frame.area());

    // Inner layout: horizontal split of content area
    let inner = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(30),
            Constraint::Percentage(70),
        ])
        .split(outer[1]);

    // Render to each area
    frame.render_widget(header, outer[0]);
    frame.render_widget(sidebar, inner[0]);
    frame.render_widget(main_content, inner[1]);
}
```

### Complex Dashboard Layout

```rust
fn render_dashboard(frame: &mut Frame) {
    let main = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Min(0),     // Body
            Constraint::Length(1),  // Status
        ])
        .split(frame.area());

    // Body: sidebar + content
    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(25),
            Constraint::Min(0),
        ])
        .split(main[1]);

    // Content: split into panels
    let content = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(60),
            Constraint::Percentage(40),
        ])
        .split(body[1]);

    // Right panel: horizontal split
    let bottom_panels = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Fill(1), Constraint::Fill(1)])
        .split(content[1]);
}
```

## 7. Margin and Spacing

### Layout Margins

```rust
use ratatui::layout::Margin;

let layout = Layout::default()
    .direction(Direction::Vertical)
    .margin(1)  // All sides
    .constraints([Constraint::Min(0)])
    .split(frame.area());

// Or specific margins
let layout = Layout::default()
    .horizontal_margin(2)
    .vertical_margin(1)
    .constraints([Constraint::Min(0)])
    .split(frame.area());
```

### Spacing Between Elements

```rust
let layout = Layout::default()
    .direction(Direction::Vertical)
    .spacing(1)  // Gap between elements
    .constraints([
        Constraint::Length(3),
        Constraint::Length(3),
        Constraint::Length(3),
    ])
    .split(frame.area());
```

## 8. Flex Properties

Control how extra space is distributed:

```rust
use ratatui::layout::Flex;

let layout = Layout::default()
    .direction(Direction::Horizontal)
    .flex(Flex::Center)  // Center elements
    .constraints([
        Constraint::Length(10),
        Constraint::Length(10),
    ])
    .split(frame.area());
```

### Flex Options

| Flex | Behavior |
|------|----------|
| `Flex::Start` | Pack at start (default) |
| `Flex::End` | Pack at end |
| `Flex::Center` | Center in available space |
| `Flex::SpaceBetween` | Distribute space between |
| `Flex::SpaceAround` | Distribute space around |

## 9. Centering Content

### Center a Widget

```rust
fn center(area: Rect, width: u16, height: u16) -> Rect {
    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Fill(1),
            Constraint::Length(width),
            Constraint::Fill(1),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Fill(1),
            Constraint::Length(height),
            Constraint::Fill(1),
        ])
        .split(horizontal[1])[1]
}

// Usage
let centered = center(frame.area(), 40, 10);
frame.render_widget(popup, centered);
```

### Using Flex for Centering

```rust
let centered = Layout::default()
    .direction(Direction::Vertical)
    .flex(Flex::Center)
    .constraints([Constraint::Length(10)])
    .split(frame.area())[0];
```

## 10. Responsive Design

### Adapt to Terminal Size

```rust
fn render(frame: &mut Frame) {
    let area = frame.area();

    if area.width < 60 {
        render_compact(frame);
    } else if area.width < 100 {
        render_medium(frame);
    } else {
        render_full(frame);
    }
}
```

### Conditional Layouts

```rust
fn get_layout(area: Rect) -> Vec<Rect> {
    let constraints = if area.width > 80 {
        vec![
            Constraint::Length(20),
            Constraint::Min(0),
            Constraint::Length(20),
        ]
    } else {
        vec![Constraint::Min(0)]
    };

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(constraints)
        .split(area)
        .to_vec()
}
```

### Hide Elements When Small

```rust
fn render(frame: &mut Frame) {
    let area = frame.area();

    // Always show main content
    let main_area = if area.width > 80 {
        // Show sidebar
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(25),
                Constraint::Min(0),
            ])
            .split(area);

        frame.render_widget(sidebar, layout[0]);
        layout[1]
    } else {
        area
    };

    frame.render_widget(main_content, main_area);
}
```

## 11. Layout Caching

For performance, cache layout calculations:

```rust
struct App {
    cached_layout: Option<Vec<Rect>>,
    last_size: (u16, u16),
}

impl App {
    fn get_layout(&mut self, area: Rect) -> &[Rect] {
        let size = (area.width, area.height);

        if self.last_size != size || self.cached_layout.is_none() {
            self.cached_layout = Some(
                Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(3),
                        Constraint::Min(0),
                    ])
                    .split(area)
                    .to_vec()
            );
            self.last_size = size;
        }

        self.cached_layout.as_ref().unwrap()
    }
}
```

## 12. Layout Algorithm Notes

- Ratatui uses the **Cassowary constraint solver**
- When constraints cannot be perfectly satisfied, the solver finds the closest approximation
- Results may be **non-deterministic** when constraints conflict
- Over-constrained layouts may produce unexpected results
- Always test layouts at various terminal sizes

## Resources

- [Layout API Docs](https://docs.rs/ratatui/latest/ratatui/layout/index.html)
- [Layout Recipes](https://ratatui.rs/recipes/layout/)
- [Constraint Reference](https://docs.rs/ratatui/latest/ratatui/layout/enum.Constraint.html)
