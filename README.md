# promkit

[![.github/workflows/promkit.yml](https://github.com/ynqa/promkit/actions/workflows/promkit.yml/badge.svg)](https://github.com/ynqa/promkit/actions/workflows/promkit.yml)
[![docs.rs](https://img.shields.io/docsrs/promkit)](https://docs.rs/promkit)

A toolkit for building your own interactive command-line tools in Rust.

## Getting Started

Put the package in your `Cargo.toml`.

```toml
[dependencies]
promkit = "0.2.0"
```

## Features

- Support cross-platform both UNIX and Windows owing to [crossterm](https://github.com/crossterm-rs/crossterm)
- Various building methods
  - Support ranging from presets for easy use to layout building using `Component`s, and even for displaying your own data structures
- Versatile customization capabilities
  - Themes for defining the outer shell style, including text and cursor colors
  - Validation for user input and error message construction
  - and so on...

## Examples

*promkit* provides presets so that users can utilize prompts immediately without
having to build complex Components for specific use cases.  

```
cargo run --example readline
```

![readline](https://github.com/ynqa/promkit/assets/6745370/afa75a49-f84b-444f-88e3-3dabca959164)

See [examples](https://github.com/ynqa/promkit/tree/main/examples/README.md)
for more examples.

## Why *promkit*?

Similar libraries in this category include the following:
- [console-rs/dialoguer](https://github.com/console-rs/dialoguer)
- [mikaelmello/inquire](https://github.com/mikaelmello/inquire/tree/main/inquire)

*promkit* offers several advantages over these libraries:

### Resilience to terminal resizing

Performing operations that involve executing a command in one pane while
simultaneously opening a new pane is a common occurrence. During such operations,
if UI corruption is caused by resizing the terminal size, it may adversely affect
the user experience.  
Other libraries can struggle when the terminal is resized, making typing and
interaction difficult or impossible. For example:

 - [(console-rs/dialoguer) Automatic re-render on terminal window resize](https://github.com/console-rs/dialoguer/issues/178)

*promkit* processes the data to fit the screen size, reducing the likelihood of
rendering issues, such as misalignment. This approach ensures that UI elements
remain consistent even when the terminal is resized, providing a smoother user
experience.

### Unified component approach

*promkit* takes a unified approach by having all of its components inherit the
same `Component` trait. This design choice enables users to seamlessly support
their custom data structures for display, similar to the relationships seen in
TUI projects like [ratatui-org/ratatui](https://github.com/ratatui-org/ratatui)
and
[EdJoPaTo/tui-rs-tree-widget](https://github.com/EdJoPaTo/tui-rs-tree-widget).
In other words, it's straightforward for anyone to display their own data
structures using widgets within promkit.  
In contrast, other libraries tend to treat each prompt as a mostly independent
entity. If you want to display a new data structure, you often have to build the
UI from scratch, which can be a time-consuming and less flexible process.

  ```rust
  pub trait Component {
      fn make_pane(&self, width: u16) -> Pane;
      fn handle_event(&mut self, event: &Event);
      fn postrun(&mut self);
  }
  ```

In the provided presets of *promkit*, this mechanism is implemented. If you'd
like to try it out, you can refer to
[the implementations of preset](https://github.com/ynqa/promkit/tree/dev-0.2.0/src/preset)
for guidance.

In summary, *promkit*'s resilience to terminal resizing and its unified component
approach make it a compelling choice for interactive command-line applications,
especially when compared to
[console-rs/dialoguer](https://github.com/console-rs/dialoguer) and
[mikaelmello/inquire](https://github.com/mikaelmello/inquire/tree/main/inquire).
These features provide a more reliable and extensible experience for developers,
allowing them to focus on building powerful command-line interfaces.

## Concept

### Dataflow from receiving events to rendering

This diagram shows the data flow for `TextEditor` component.

```mermaid
graph
  subgraph Dataflow
    Event --> EventHandler
    subgraph TextEditor as Compoent
      EventHandler --> |edit| TextBuffer
      TextBuffer --> |matrixify| Pane
    end
      Pane -->|extract| Lines
    Lines --> F([Draw])
  end
```

When an event comes in, it is handled by the handler inside the `TextEditor`
component. The handler then edits (e.g. insert character) `TextBuffer`.
This `TextBuffer` is used to construct a `Pane`, which is essentially a matrix of
lines divided by a specific width. The panes are extracted a certain number of
lines in order to fit within the terminal screen when rendering.
Finally, these Lines are passed to a `draw` function which renders them on the screen.
