# Ratatui Macro for JSX Syntax

## Installation
- Make sure you've got `ratatui` installed and a basic project setup.
- Crate a ratatui project. For this example, I'll be referencing code from [here](https://ratatui.rs/tutorials/hello-ratatui/).
- Clone this repo, and add the folder of the crate `flint-macros` to your project's directory.
- Setup a cargo workspace, and add both the ratatui project and `flint-macros` as members.
- Please see the example folder structure below
```
example\
    flint-macros\
    ratatui-hello-world\
    Cargo.toml
```
- The Cargo.toml in your project root (`example`) should look like this:
```
[workspace]
resolver = "2"

members = ["flint-macros", "ratatui-hello-world"]
```

- Make sure to add the `flint-macros` crate to your project's dependencies in the `ratatui-hello-world` Cargo.toml file.
```
[dependencies]
flint-macros = { path = "../flint-macros" }
```

## Usage
The macro simplifies the process of creating, compose and rendering widgets. Let's compare the code to create and render different widgets.
I'll be omitting the boilerplate parts of the code like setting up the terminal, etc.

If you're trying this out, I trust you have some ratatui experience, and will be able to fill in the parts I've omitted fairly easily.

### Rendering Widgets
Let's see the difference for rendering a simple text widget.

```rs
// Normal
use ratatui::text::Text;
let text = Text::raw("Hello World");
frame.render_widget("hello world", frame.area());
```

```rs
// with macro
use flint_macros::ui;
use ratatui::text::Text;

ui!(frame => {
    Text::raw("Hello World")
})
```

There isn't much of a difference here, in fact - the normal code might bemore readable. Let's take a look at a more involved example.

### Rendering and nesting Layouts
```rs
// normal

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::Text,
    widgets::Paragraph,
    Frame,
};

// Create outer layout
let outer_chunks = Layout::default()
    .direction(Direction::Vertical)
    .constraints([
        Constraint::Length(1),
        Constraint::Min(0),
    ])
    .split(frame.size());

// Create inner layout
let inner_chunks = Layout::default()
    .direction(Direction::Horizontal)
    .constraints([
        Constraint::Percentage(50),
        Constraint::Percentage(50),
    ])
    .split(outer_chunks[1]);

// Render text in each section
frame.render_widget(
    Paragraph::new("Header")
        .style(Style::default().fg(Color::Blue)),
    outer_chunks[0]
);

frame.render_widget(
    Paragraph::new("Left Content")
        .style(Style::default().fg(Color::Green)),
    inner_chunks[0]
);

frame.render_widget(
    Paragraph::new("Right Content")
        .style(Style::default().fg(Color::Yellow)),
    inner_chunks[1]
);
```

```rs
// with macros

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::Text,
    widgets::Paragraph,
    Frame,
};
use flint_macros::ui;

ui!(frame => {
    Layout(
        direction: Direction::Vertical,
        constraints: [Constraint::Length(1), Constraint::Min(0)]
    ) {
        Paragraph::new(
            "Header",
            style: Style::default().fg(Color::Blue)
        ),

        Layout(
            direction: Direction::Horizontal,
            constraints: [Constraint::Percentage(50), Constraint::Percentage(50)]
        ) {
            Paragraph::new(
                "Left Content",
                style: Style::default().fg(Color::Green)
            ),
            Paragraph::new(
                "Right Content",
                style: Style::default().fg(Color::Yellow)
            )
        }
    }
});
```
In this example, the `ui!()` macro make it easier to compose layouts and widgets, as well as make the structure of the TUI easier to see.

### Rendering with iterators

```rs
// normal

use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::Text,
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use flint_macros::{ui, widget};

struct Item {
    title: String,
    status: bool,
}

let chunks = Layout::default()
    .direction(Direction::Vertical)
    .constraints(
        Constraint::from_lengths(vec![3; items.len()])
    )
    .split(frame.size());

for (idx, (item, chunk)) in items.iter().zip(chunks.iter()).enumerate() {
    let block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().fg(if item.status { Color::Green } else { Color::Red }));

    frame.render_widget(
        Paragraph::new(format!("{}. {}", idx + 1, item.title))
            .block(block),
        *chunk
    );
}
```

```rs
// with macros
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::Text,
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use flint_macros::{ui, widget};

struct Item {
    title: String,
    status: bool,
}

ui!(frame => {
    Layout(
        direction: Direction::Vertical
    ) {
        For (
            (idx, item) in items.iter().enumerate(),
            constraints: Constraint::from_lengths(vec![3; items.len()]),
            direction: Direction::Vertical
        ) {
            Paragraph::new(
                format!("{}. {}", idx + 1, item.title),
                block: widget!({
                    Block::bordered(
                        borders: Borders::ALL,
                        style: Style::default().fg(if item.status { Color::Green } else { Color::Red })
                    )
                })
            )
        }
    }
});
```


## Advanced Features
I believe by this point, you're convinced of the benefits of this macro. The below examples show how the macro simplifies some complicated UI patterns.

### Variable Widgets

Variables containing widgets can be rendered using a double brace syntax:

```rust
// Store a widget in a variable
let my_widget = Paragraph::new("Hello");

// Render it using { }
ui!(frame => {
    { my_widget }
});

// The double braces can contain any expression that returns a Widget
// here widget1 and widget2 must be of the same type
ui!(frame => {
    { if complex_condition {
        widget1
    } else {
        widget2
    } }
});
```

### Conditional Rendering
If the example just above, both the widgets must be of the same type. If they aren't, you'll need to use the If-Else syntax.
The macro supports if/else conditional rendering using a syntax similar to JSX:

```rust
// Simple if condition
ui!(frame => {
    If(show_header) {
        Paragraph::new("Header")
    }
});

// If-else condition
ui!(frame => {
    If(is_loading) {
        Spinner::new()
    } Else {
        Paragraph::new("Content loaded!")
    }
});

// Can be nested in layouts
ui!(frame => {
    Layout(direction: Direction::Vertical) {
        If(show_header) {
            Paragraph::new("Header", style: header_style)
        },
        If(has_error) {
            Paragraph::new("Error!", style: error_style)
        } Else {
            Paragraph::new("Success!", style: success_style)
        }
    }
});
```

### Stateful Widgets

For widgets that maintain state (like List or Table), use the Stateful wrapper:
If you create a widget using Stateful in the `widget!()` macro, you won't need to pass
the state in again - you can use the variable rendering syntax directly.

```rust
// Create the state
let mut list_state = ListState::default();

// Wrap the widget with Stateful
ui!(frame => {
    Stateful(list_state) {
        List::new(items)
    }
});

// Can be combined with other features
ui!(frame => {
    Layout(direction: Direction::Horizontal) {
        Stateful(left_state) {
            List::new(left_items)
        },
        Stateful(right_state) {
            Table::new(right_items)
        }
    }
});
```

### Rendering by Reference
If any of your widgets implement WidgetRef instead of widget, you cannot directly render them.
To simplify this process, just prefix the widget with `&`.

Note that this doesn't work with the `For`, `If-Else` and `Stateful` syntax.


```rust
// Regular widget
ui!(frame => {
    Paragraph::new("By value")
});

// Widget rendered by reference
ui!(frame => {
    &Paragraph::new("By reference")
});

// Useful for certain stateful widgets
ui!(frame => {
    Stateful(table_state) {
        &Table::new(items)
    }
});

// Can be used with variable widgets too
let widget = Table::new(items);
ui!(frame => {
    &{{ widget }}
});
```


I'll add more docs soon, but here's some quick examples of additional functionality.
1.
```rs
// Default constructor: Widget::default()
ui!(frame => { Widget() });

// Named constructor with positional args: Widget::new("title", 42)
ui!(frame => { Widget::new("title", 42) });

// Default constructor with named args (chains .color(Red) after construction)
// "named" arguments are converted to cgained function calls.
// So code like this:
ui!(frame => { Widget(color: Color::Red) });
// will be converted to this (and some extra rendering stuff involving the frame):
Widget::default().color(Color::Red);

// Named constructor with both positional and named args
// Results in: Widget::custom("title", 42).color(Red).bold(true)
ui!(frame => { Widget::custom("title", 42, color: Color::Red, bold: true) });

// example with Block widget:
ui!(frame => { Block::bordered(title: "My Block", borders: Borders::ALL, style: Style::default().fg(Color::Blue)) });

// Variable widget: renders any expression that implements Widget trait
ui!(frame => { { my_custom_widget } });

// For loop with iterator and layout constraints
// Under the hood, this converts to a layout, hence you can pas all the named arguments
// you could to a Layout widget
ui!(frame => {
    For (item in items.iter(), constraints: [Constraint::Length(3)]) {
        { item.to_widget() }
    }
});

// widget!() macro - used for creating widget instances that will be used as parameters
Paragraph::new("text", block: widget!({ Block::bordered(title: "Title") }));

// widget!() is especially useful for blocks and other nested widget parameters
List::new(items, highlight_style: widget!({ Style::default().fg(Color::Blue) }));

// though layouts aren't exactly widgets, they can be used as such with the widget macro
// for example, we can create separate layouts and compose them quite easily.
// under the hood, this works by wrapping the layout in a struct that implements
// the Widget trait, allow it to be stored in a variable and be used as a widget.

let sidebar = widget!({
    Layout(direction: Direction::Vertical) {
        Block::bordered(title: "Navigation"),
        For (item in menu_items, constraints: [Constraint::Length(1)]) {
            Text::raw(item)
        }
    }
});

// Create another layout component
let content_panel = widget!({
    Layout(direction: Direction::Horizontal) {
        Paragraph::new("Left panel"),
        Paragraph::new("Right panel")
    }
});

// Use them together in the main UI using {{ }}
ui!(frame => {
    Layout(direction: Direction::Horizontal) {
        { sidebar },
        { content_panel }
    }
});
```
```
