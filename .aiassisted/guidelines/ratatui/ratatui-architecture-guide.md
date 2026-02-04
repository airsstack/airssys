---
description: Application architecture patterns for Ratatui TUI applications. Covers Elm Architecture (TEA), Component Architecture, and Flux patterns.
globs: "**/*.rs"
---

# Ratatui Architecture Guide

This guide covers the primary architectural patterns for structuring Ratatui applications. Choose the pattern that best fits your application's complexity and team preferences.

## 1. Overview of Patterns

| Pattern | Best For | Complexity | State Management |
|---------|----------|------------|------------------|
| **Elm Architecture** | Simple to medium apps | Low-Medium | Centralized |
| **Component Architecture** | Complex UIs | Medium-High | Distributed |
| **Flux Architecture** | Large apps with complex state | High | Unidirectional |

## 2. The Elm Architecture (TEA)

The Elm Architecture is a functional programming pattern organized around three core concepts: **Model**, **Update**, and **View**.

### Core Components

#### Model (State Container)

```rust
#[derive(Debug, Default)]
pub struct Model {
    pub counter: i32,
    pub running_state: RunningState,
    pub input_mode: InputMode,
}

#[derive(Debug, Default, PartialEq)]
pub enum RunningState {
    #[default]
    Running,
    Done,
}

#[derive(Debug, Default)]
pub enum InputMode {
    #[default]
    Normal,
    Editing,
}
```

#### Messages (Events)

```rust
pub enum Message {
    Increment,
    Decrement,
    Reset,
    ToggleMode,
    Quit,
}
```

#### Update (State Transformer)

```rust
fn update(model: &mut Model, msg: Message) -> Option<Message> {
    match msg {
        Message::Increment => {
            model.counter += 1;
            // Cascade to Reset if counter exceeds threshold
            if model.counter > 50 {
                return Some(Message::Reset);
            }
        }
        Message::Decrement => {
            model.counter = model.counter.saturating_sub(1);
        }
        Message::Reset => {
            model.counter = 0;
        }
        Message::ToggleMode => {
            model.input_mode = match model.input_mode {
                InputMode::Normal => InputMode::Editing,
                InputMode::Editing => InputMode::Normal,
            };
        }
        Message::Quit => {
            model.running_state = RunningState::Done;
        }
    }
    None
}
```

#### View (UI Renderer)

```rust
fn view(model: &Model, frame: &mut Frame) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(frame.area());

    let counter_text = format!("Counter: {}", model.counter);
    let paragraph = Paragraph::new(counter_text)
        .block(Block::default().borders(Borders::ALL).title("Counter"));

    frame.render_widget(paragraph, chunks[0]);
}
```

### Event Handling

```rust
fn handle_event(model: &Model) -> std::io::Result<Option<Message>> {
    if crossterm::event::poll(std::time::Duration::from_millis(100))? {
        if let Event::Key(key) = crossterm::event::read()? {
            if key.kind == KeyEventKind::Press {
                return Ok(match key.code {
                    KeyCode::Char('q') => Some(Message::Quit),
                    KeyCode::Up => Some(Message::Increment),
                    KeyCode::Down => Some(Message::Decrement),
                    KeyCode::Char('r') => Some(Message::Reset),
                    _ => None,
                });
            }
        }
    }
    Ok(None)
}
```

### Main Loop

```rust
fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let mut terminal = ratatui::init();
    let mut model = Model::default();

    while model.running_state != RunningState::Done {
        terminal.draw(|f| view(&model, f))?;

        let mut current_msg = handle_event(&model)?;
        while let Some(msg) = current_msg {
            current_msg = update(&mut model, msg);
        }
    }

    ratatui::restore();
    Ok(())
}
```

### Advantages of TEA

- **Predictability**: Same state always produces same UI
- **Testability**: Pure functions are easy to test
- **Message Cascading**: Complex state transitions via chained messages
- **Separation of Concerns**: Clear boundaries between state, logic, rendering

## 3. Component Architecture

Component Architecture structures applications as a tree of self-contained, reusable components with their own state and logic.

### Component Trait

```rust
pub trait Component {
    type Message;

    fn update(&mut self, msg: Self::Message) -> Option<Self::Message>;
    fn view(&self, frame: &mut Frame, area: Rect);
    fn handle_event(&mut self, event: Event) -> Option<Self::Message>;
}
```

### Example Component

```rust
pub struct Counter {
    value: i32,
    focused: bool,
}

pub enum CounterMessage {
    Increment,
    Decrement,
    Focus,
    Blur,
}

impl Component for Counter {
    type Message = CounterMessage;

    fn update(&mut self, msg: Self::Message) -> Option<Self::Message> {
        match msg {
            CounterMessage::Increment => self.value += 1,
            CounterMessage::Decrement => self.value = self.value.saturating_sub(1),
            CounterMessage::Focus => self.focused = true,
            CounterMessage::Blur => self.focused = false,
        }
        None
    }

    fn view(&self, frame: &mut Frame, area: Rect) {
        let style = if self.focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .title("Counter")
            .border_style(style);

        let paragraph = Paragraph::new(format!("{}", self.value))
            .block(block)
            .alignment(Alignment::Center);

        frame.render_widget(paragraph, area);
    }

    fn handle_event(&mut self, event: Event) -> Option<Self::Message> {
        if !self.focused {
            return None;
        }

        if let Event::Key(key) = event {
            if key.kind == KeyEventKind::Press {
                return match key.code {
                    KeyCode::Up => Some(CounterMessage::Increment),
                    KeyCode::Down => Some(CounterMessage::Decrement),
                    _ => None,
                };
            }
        }
        None
    }
}
```

### Parent Component

```rust
pub struct App {
    counters: Vec<Counter>,
    active_counter: usize,
}

impl App {
    fn view(&self, frame: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Percentage(50); self.counters.len()])
            .split(frame.area());

        for (i, counter) in self.counters.iter().enumerate() {
            counter.view(frame, chunks[i]);
        }
    }
}
```

### Advantages of Component Architecture

- **Reusability**: Components can be reused across applications
- **Encapsulation**: State and logic contained within components
- **Composability**: Build complex UIs from simple components
- **Testability**: Test components in isolation

## 4. Flux Architecture

Flux implements unidirectional data flow: Actions -> Dispatcher -> Stores -> Views.

### Actions

```rust
#[derive(Debug, Clone)]
pub enum Action {
    IncrementCounter(usize),
    DecrementCounter(usize),
    SetActiveTab(usize),
    LoadData,
    DataLoaded(Vec<String>),
}
```

### Store

```rust
pub struct Store {
    counters: Vec<i32>,
    active_tab: usize,
    data: Vec<String>,
    loading: bool,
}

impl Store {
    pub fn dispatch(&mut self, action: Action) {
        match action {
            Action::IncrementCounter(idx) => {
                if let Some(counter) = self.counters.get_mut(idx) {
                    *counter += 1;
                }
            }
            Action::DecrementCounter(idx) => {
                if let Some(counter) = self.counters.get_mut(idx) {
                    *counter = counter.saturating_sub(1);
                }
            }
            Action::SetActiveTab(tab) => {
                self.active_tab = tab;
            }
            Action::LoadData => {
                self.loading = true;
            }
            Action::DataLoaded(data) => {
                self.data = data;
                self.loading = false;
            }
        }
    }

    // Selectors
    pub fn get_counter(&self, idx: usize) -> Option<i32> {
        self.counters.get(idx).copied()
    }

    pub fn is_loading(&self) -> bool {
        self.loading
    }
}
```

### View with Store

```rust
fn view(store: &Store, frame: &mut Frame) {
    let tabs = Tabs::new(vec!["Counter", "Data"])
        .select(store.active_tab);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(frame.area());

    frame.render_widget(tabs, chunks[0]);

    match store.active_tab {
        0 => render_counter_view(store, frame, chunks[1]),
        1 => render_data_view(store, frame, chunks[1]),
        _ => {}
    }
}
```

### Advantages of Flux

- **Predictable State**: Single source of truth
- **Debugging**: Easy to track state changes
- **Scalability**: Handles complex state well
- **Time Travel**: Can implement undo/redo

## 5. Pattern Selection Guide

### Choose Elm Architecture When

- Building simple to medium complexity apps
- Want predictable, testable code
- Prefer functional programming style
- State can be centralized

### Choose Component Architecture When

- Building complex UIs with reusable parts
- Need encapsulated, self-contained components
- Want to compose UIs from smaller pieces
- Different parts of UI have independent state

### Choose Flux Architecture When

- Building large applications
- State is complex and shared across views
- Need strict unidirectional data flow
- Want clear separation between state and UI

## 6. Practical Considerations

### StatefulWidget Integration

Note that `StatefulWidget`s require mutable references during rendering, which may require relaxing immutability principles:

```rust
fn view(model: &mut Model, frame: &mut Frame) {
    let list = List::new(model.items.clone());
    frame.render_stateful_widget(list, area, &mut model.list_state);
}
```

### Async Operations

For async operations, consider using channels:

```rust
use std::sync::mpsc::{channel, Receiver, Sender};

struct App {
    action_tx: Sender<Action>,
    action_rx: Receiver<Action>,
}

impl App {
    fn spawn_async_task(&self) {
        let tx = self.action_tx.clone();
        tokio::spawn(async move {
            let data = fetch_data().await;
            tx.send(Action::DataLoaded(data)).ok();
        });
    }
}
```

## 7. Testing Patterns

### Testing Update Functions

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_increment_updates_counter() {
        let mut model = Model::default();
        update(&mut model, Message::Increment);
        assert_eq!(model.counter, 1);
    }

    #[test]
    fn test_cascade_to_reset() {
        let mut model = Model { counter: 50, ..Default::default() };
        let next = update(&mut model, Message::Increment);
        assert_eq!(next, Some(Message::Reset));
    }
}
```

### Testing Components

```rust
#[test]
fn test_counter_component() {
    let mut counter = Counter::default();
    counter.update(CounterMessage::Increment);
    assert_eq!(counter.value, 1);
}
```

## Resources

- [Elm Architecture Original](https://guide.elm-lang.org/architecture/)
- [Ratatui Application Patterns](https://ratatui.rs/concepts/application-patterns/)
- [Component-based TUI](https://ratatui.rs/concepts/application-patterns/component-architecture/)
