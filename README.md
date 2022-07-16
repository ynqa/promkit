# promkit

[![.github/workflows/promkit.yml](https://github.com/ynqa/promkit/actions/workflows/promkit.yml/badge.svg)](https://github.com/ynqa/promkit/actions/workflows/promkit.yml)
[![docs.rs](https://img.shields.io/docsrs/promkit)](https://docs.rs/promkit)

A toolkit for building your own interactive command-line tools in Rust,
utilizing [crossterm](https://github.com/crossterm-rs/crossterm).

## Getting Started

Put the package in your `Cargo.toml`.

```toml
[dependencies]
promkit = "0.1.1"
```

## Features

- Applications
  - Readline: provide the lines to receive and display user inputs
    - Masking
    - Switch insert/overwrite modes
    - Suggestions
  - Select: provide the selectbox to choose the items from
    - Move cyclically in the selectbox
- Customizations
  - Edit key-bindings
  - Edit crossterm [Event](https://docs.rs/crossterm/0.23.0/crossterm/event/enum.Event.html) handler
  - Define your own command-line applications
    - e.g. [ynqa/hstr-rs](https://github.com/ynqa/hstr-rs)

## Examples

Readline:

![readline](https://user-images.githubusercontent.com/6745370/175757317-94e75ddd-f968-43ba-8a3e-0e1e70191128.gif)

```rust
use promkit::{build::Builder, readline, Result};

fn main() -> Result<()> {
    let mut p = readline::Builder::default().build()?;
    loop {
        let line = p.run()?;
        println!("result: {:?}", line);
    }
}
```

Select:

![select](https://user-images.githubusercontent.com/6745370/175757316-8499ace6-e520-465b-a3fe-671182015431.gif)

```rust
use promkit::{
    build::Builder, crossterm::style, register::Register, select, selectbox::SelectBox, Result,
};

fn main() -> Result<()> {
    let mut selectbox = Box::new(SelectBox::default());
    selectbox.register_all((0..100).map(|v| v.to_string()).collect::<Vec<String>>());
    let mut p = select::Builder::default()
        .title("Q: What number do you like?")
        .title_color(style::Color::DarkGreen)
        .selectbox(selectbox)
        .build()?;
    let line = p.run()?;
    println!("result: {:?}", line);
    Ok(())
}
```
