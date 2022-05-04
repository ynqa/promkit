# promkit

[![.github/workflows/promkit.yml](https://github.com/ynqa/promkit/actions/workflows/promkit.yml/badge.svg)](https://github.com/ynqa/promkit/actions/workflows/promkit.yml)
[![docs.rs](https://img.shields.io/docsrs/promkit)](https://docs.rs/promkit)

A toolkit for building your own interactive command-line tools in Rust,
utilizing [crossterm](https://github.com/crossterm-rs/crossterm).

## Getting Started

Put the package in your `Cargo.toml`.

```toml
[dependencies]
promkit = "0.1.0"
```

## Features

- Readline
  - Provide the lines to receive and display user inputs
  - Masking, switch insert/overwrite modes, and suggestions
- Select
  - Provides the selectbox to choose the items from
- Customization
  - Enable to define your own command-line applications.
  - See [examples/advanced](./examples/advanced/)
    for more concrete examples.

## Examples

Readline:

```rust
use promkit::{build::Builder, readline, Result};

fn main() -> Result<()> {
    let mut p = readline::Builder::default().build()?;
    loop {
        let (line, exit_code) = p.run()?;
        if exit_code == 0 {
            println!("result: {:?}", line);
        } else {
            return Ok(());
        }
    }
}
```

Select:

```rust
use crossterm::style;
use promkit::{
    build::Builder,
    edit::{Register, SelectBox},
    select, Result,
};

fn main() -> Result<()> {
    let mut selectbox = Box::new(SelectBox::default());
    selectbox.register_all((0..100).map(|v| v.to_string()).collect::<Vec<String>>());
    let mut p = select::Builder::default()
        .title("Q: What number do you like?")
        .title_color(style::Color::DarkGreen)
        .selectbox(selectbox)
        .build()?;
    let (line, exit_code) = p.run()?;
    if exit_code == 0 {
        println!("result: {:?}", line)
    }
    Ok(())
}
```
