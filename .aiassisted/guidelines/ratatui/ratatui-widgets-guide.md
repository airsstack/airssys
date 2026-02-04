---
description: Comprehensive guide to Ratatui widgets. Covers built-in widgets, custom widget creation, stateful widgets, and widget composition patterns.
globs: "**/*.rs"
---

# Ratatui Widgets Guide

Widgets are the fundamental building blocks for Ratatui terminal user interfaces. They handle layout, styling, and rendering of UI elements.

## 1. Built-in Widgets Overview

Ratatui provides 13 core widgets:

| Widget | Purpose | Stateful |
|--------|---------|----------|
| **Block** | Bordered container with title | No |
| **Paragraph** | Styled, wrappable text | No |
| **List** | Selectable item collection | Yes |
| **Table** | Grid with row/column selection | Yes |
| **Tabs** | Tabbed navigation | No |
| **BarChart** | Bar visualizations | No |
| **Chart** | Line/scatter plots | No |
| **Sparkline** | Compact data visualization | No |
| **Gauge** | Progress indicator (blocks) | No |
| **LineGauge** | Progress indicator (line) | No |
| **Calendar** | Month view display | No |
| **Canvas** | Arbitrary shapes | No |
| **Scrollbar** | Scroll position indicator | Yes |
| **Clear** | Remove rendered content | No |

## 2. Basic Widget Usage

### Rendering Widgets

```rust
use ratatui::{Frame, widgets::{Block, Borders, Paragraph}};

fn render(frame: &mut Frame) {
    let block = Block::default()
        .title("My Block")
        .borders(Borders::ALL);

    let paragraph = Paragraph::new("Hello, Ratatui!")
        .block(block);

    frame.render_widget(paragraph, frame.area());
}
```

### Block Widget

The most common container widget:

```rust
use ratatui::widgets::{Block, Borders, BorderType};
use ratatui::style::{Color, Style};

let block = Block::default()
    .title("Title")
    .title_alignment(Alignment::Center)
    .borders(Borders::ALL)
    .border_type(BorderType::Rounded)
    .border_style(Style::default().fg(Color::Cyan));
```

### Paragraph Widget

For displaying styled text:

```rust
use ratatui::widgets::{Paragraph, Wrap};
use ratatui::text::{Line, Span};
use ratatui::style::{Color, Modifier, Style};

let text = vec![
    Line::from(vec![
        Span::raw("Hello "),
        Span::styled("World", Style::default().fg(Color::Green)),
    ]),
    Line::from("Second line"),
];

let paragraph = Paragraph::new(text)
    .block(Block::default().borders(Borders::ALL))
    .style(Style::default().fg(Color::White))
    .alignment(Alignment::Center)
    .wrap(Wrap { trim: true });
```

### List Widget (Stateful)

```rust
use ratatui::widgets::{List, ListItem, ListState};

let items: Vec<ListItem> = vec![
    ListItem::new("Item 1"),
    ListItem::new("Item 2"),
    ListItem::new("Item 3"),
];

let list = List::new(items)
    .block(Block::default().borders(Borders::ALL).title("List"))
    .highlight_style(Style::default().add_modifier(Modifier::BOLD))
    .highlight_symbol("> ");

// State tracks selection
let mut state = ListState::default();
state.select(Some(0));

frame.render_stateful_widget(list, area, &mut state);
```

### Table Widget (Stateful)

```rust
use ratatui::widgets::{Table, Row, Cell, TableState};

let rows = vec![
    Row::new(vec![Cell::from("Row1"), Cell::from("Data1")]),
    Row::new(vec![Cell::from("Row2"), Cell::from("Data2")]),
];

let widths = [Constraint::Length(10), Constraint::Length(20)];

let table = Table::new(rows, widths)
    .block(Block::default().borders(Borders::ALL).title("Table"))
    .header(Row::new(vec!["Name", "Value"]).style(Style::default().bold()))
    .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

let mut state = TableState::default();
state.select(Some(0));

frame.render_stateful_widget(table, area, &mut state);
```

### Tabs Widget

```rust
use ratatui::widgets::Tabs;

let titles = vec!["Tab1", "Tab2", "Tab3"];
let tabs = Tabs::new(titles)
    .block(Block::default().borders(Borders::ALL))
    .select(0)
    .style(Style::default().fg(Color::White))
    .highlight_style(Style::default().fg(Color::Yellow));

frame.render_widget(tabs, area);
```

### Gauge Widget

```rust
use ratatui::widgets::Gauge;

let gauge = Gauge::default()
    .block(Block::default().borders(Borders::ALL).title("Progress"))
    .gauge_style(Style::default().fg(Color::Green))
    .percent(75)
    .label("75%");

frame.render_widget(gauge, area);
```

### Chart Widget

```rust
use ratatui::widgets::{Chart, Dataset, Axis, GraphType};
use ratatui::symbols::Marker;

let data = vec![(0.0, 1.0), (1.0, 3.0), (2.0, 2.0), (3.0, 4.0)];

let dataset = Dataset::default()
    .name("Data")
    .marker(Marker::Braille)
    .graph_type(GraphType::Line)
    .style(Style::default().fg(Color::Cyan))
    .data(&data);

let chart = Chart::new(vec![dataset])
    .block(Block::default().borders(Borders::ALL).title("Chart"))
    .x_axis(Axis::default().title("X").bounds([0.0, 4.0]))
    .y_axis(Axis::default().title("Y").bounds([0.0, 5.0]));

frame.render_widget(chart, area);
```

### Sparkline Widget

```rust
use ratatui::widgets::Sparkline;

let data = vec![0, 2, 3, 4, 1, 4, 10, 5, 3, 2];

let sparkline = Sparkline::default()
    .block(Block::default().title("Sparkline"))
    .data(&data)
    .style(Style::default().fg(Color::Yellow));

frame.render_widget(sparkline, area);
```

### Calendar Widget

Requires `widget-calendar` feature:

```rust
use ratatui::widgets::calendar::{Monthly, CalendarEventStore};
use time::Date;

let events = CalendarEventStore::default();
let calendar = Monthly::new(
    Date::from_calendar_date(2024, time::Month::January, 1).unwrap(),
    events,
)
.show_month_header(Style::default().bold());

frame.render_widget(calendar, area);
```

## 3. Text Primitives

Basic text types also implement Widget:

```rust
// String and &str as widgets
frame.render_widget("Hello", area);
frame.render_widget(String::from("World"), area);

// Span - styled text segment
let span = Span::styled("Styled", Style::default().fg(Color::Red));

// Line - horizontal text composition
let line = Line::from(vec![
    Span::raw("Normal "),
    Span::styled("Bold", Style::default().bold()),
]);

// Text - multiple lines
let text = Text::from(vec![
    Line::from("Line 1"),
    Line::from("Line 2"),
]);
```

## 4. Widget Traits

### Widget Trait (Consuming)

Basic widgets implement `Widget`:

```rust
pub trait Widget {
    fn render(self, area: Rect, buf: &mut Buffer);
}
```

### StatefulWidget Trait

For widgets that maintain state:

```rust
pub trait StatefulWidget {
    type State;
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State);
}
```

### WidgetRef Trait (Unstable)

For rendering by reference (requires `unstable-widget-ref` feature):

```rust
pub trait WidgetRef {
    fn render_ref(&self, area: Rect, buf: &mut Buffer);
}
```

## 5. Creating Custom Widgets

### Basic Custom Widget

```rust
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::widgets::Widget;

pub struct StatusBar {
    message: String,
    style: Style,
}

impl StatusBar {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            style: Style::default(),
        }
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }
}

impl Widget for StatusBar {
    fn render(self, area: Rect, buf: &mut Buffer) {
        buf.set_string(area.x, area.y, &self.message, self.style);
    }
}
```

### Compositional Widget

Build widgets from other widgets:

```rust
pub struct InfoPanel {
    title: String,
    content: String,
}

impl Widget for InfoPanel {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .title(self.title)
            .borders(Borders::ALL);

        let inner = block.inner(area);
        block.render(area, buf);

        Paragraph::new(self.content).render(inner, buf);
    }
}
```

### Stateful Custom Widget

```rust
pub struct ScrollableList {
    items: Vec<String>,
}

pub struct ScrollableListState {
    offset: usize,
    selected: Option<usize>,
}

impl Default for ScrollableListState {
    fn default() -> Self {
        Self {
            offset: 0,
            selected: None,
        }
    }
}

impl ScrollableListState {
    pub fn select(&mut self, index: Option<usize>) {
        self.selected = index;
    }

    pub fn selected(&self) -> Option<usize> {
        self.selected
    }

    pub fn scroll_down(&mut self) {
        self.offset = self.offset.saturating_add(1);
    }

    pub fn scroll_up(&mut self) {
        self.offset = self.offset.saturating_sub(1);
    }
}

impl StatefulWidget for ScrollableList {
    type State = ScrollableListState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let visible_items = self.items
            .iter()
            .skip(state.offset)
            .take(area.height as usize);

        for (i, item) in visible_items.enumerate() {
            let style = if state.selected == Some(state.offset + i) {
                Style::default().add_modifier(Modifier::REVERSED)
            } else {
                Style::default()
            };

            buf.set_string(
                area.x,
                area.y + i as u16,
                item,
                style,
            );
        }
    }
}
```

## 6. Widget Styling

### Style Struct

```rust
use ratatui::style::{Color, Modifier, Style};

let style = Style::default()
    .fg(Color::White)           // Foreground color
    .bg(Color::Black)           // Background color
    .add_modifier(Modifier::BOLD)
    .add_modifier(Modifier::ITALIC);
```

### Available Colors

```rust
// Named colors
Color::Black, Color::Red, Color::Green, Color::Yellow,
Color::Blue, Color::Magenta, Color::Cyan, Color::White,
Color::Gray, Color::DarkGray

// Bright variants
Color::LightRed, Color::LightGreen, Color::LightYellow,
Color::LightBlue, Color::LightMagenta, Color::LightCyan

// RGB and indexed
Color::Rgb(255, 128, 0)
Color::Indexed(42)

// Reset
Color::Reset
```

### Available Modifiers

```rust
Modifier::BOLD
Modifier::DIM
Modifier::ITALIC
Modifier::UNDERLINED
Modifier::SLOW_BLINK
Modifier::RAPID_BLINK
Modifier::REVERSED
Modifier::HIDDEN
Modifier::CROSSED_OUT
```

## 7. Best Practices

### Widget Construction Pattern

Use builder pattern for configuration:

```rust
impl MyWidget {
    pub fn new(required: String) -> Self {
        Self {
            required,
            optional: None,
            style: Style::default(),
        }
    }

    pub fn optional(mut self, value: String) -> Self {
        self.optional = Some(value);
        self
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }
}
```

### Avoid Widget Reconstruction

For complex widgets, consider caching:

```rust
struct App {
    // Cache expensive widget data
    cached_items: Vec<ListItem<'static>>,
    items_dirty: bool,
}

impl App {
    fn update_items(&mut self, new_data: Vec<String>) {
        self.cached_items = new_data
            .into_iter()
            .map(ListItem::new)
            .collect();
        self.items_dirty = false;
    }
}
```

### Responsive Widgets

Adapt to available space:

```rust
impl Widget for ResponsiveWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width < 20 {
            // Compact view
            self.render_compact(area, buf);
        } else {
            // Full view
            self.render_full(area, buf);
        }
    }
}
```

## 8. Third-Party Widgets

Explore additional widgets:

- [tui-textarea](https://github.com/rhysd/tui-textarea) - Text area with editing
- [tui-input](https://github.com/sayanarijit/tui-input) - Text input field
- [tui-tree-widget](https://github.com/EdJoPaTo/tui-rs-tree-widget) - Tree view
- [ratatui-image](https://github.com/benjajaja/ratatui-image) - Image rendering

See [Awesome Ratatui](https://github.com/ratatui/awesome-ratatui) for more.

## Resources

- [Widget API Docs](https://docs.rs/ratatui/latest/ratatui/widgets/index.html)
- [Widget Showcase](https://ratatui.rs/showcase/widgets/)
- [Widget Examples](https://github.com/ratatui/ratatui/tree/main/examples)
