# promkit

[![ci](https://github.com/ynqa/promkit/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/ynqa/promkit/actions/workflows/ci.yml)
[![docs.rs](https://img.shields.io/docsrs/promkit)](https://docs.rs/promkit)

A toolkit for building your own interactive prompt in Rust.

## Getting Started

Put the package in your `Cargo.toml`.

```toml
[dependencies]
promkit = "0.9.0"
```

## Features

- Cross-platform support for both UNIX and Windows utilizing [crossterm](https://github.com/crossterm-rs/crossterm)
- Modularized architecture
  - `promkit-core`
    - Core functionality for basic terminal operations and pane management
  - `promkit-widgets`
    - Various UI components (text, listbox, tree, etc.)
  - `promkit`
    - High-level presets and user interfaces
  - `promkit-derive`
    - A Derive macro that simplifies interactive form input
- Rich preset components
  - [Readline](#readline) - Text input with auto-completion
  - [Confirm](#confirm) - Yes/no confirmation prompt
  - [Password](#password) - Password input with masking and validation
  - [Form](#form) - Manage multiple text input fields
  - [Listbox](#listbox) - Single selection interface from a list
  - [QuerySelector](#queryselector) - Searchable selection interface
  - [Checkbox](#checkbox) - Multiple selection checkbox interface
  - [Tree](#tree) - Tree display for hierarchical data like file systems
  - [JSON](#json) - Parse and interactively display JSON data
  - [Text](#text) - Static text display

## Concept

See [here](Concept.md).

## Projects using *promkit*

- [ynqa/empiriqa](https://github.com/ynqa/empiriqa)
- [ynqa/jnv](https://github.com/ynqa/jnv)
- [ynqa/logu](https://github.com/ynqa/logu)
- [ynqa/sig](https://github.com/ynqa/sig)

## Examples/Demos

*promkit* provides presets so that users can try prompts immediately without
having to build complex components for specific use cases.

Show you commands, code, and actual demo screens for examples
that can be executed immediately below.

### Readline

<details>
<summary>Command</summary>

```bash
cargo run --bin readline --manifest-path examples/readline/Cargo.toml
```

</details>

<details>
<summary>Code</summary>

```rust
use promkit::{preset::readline::Readline, suggest::Suggest, Result};

fn main() -> Result {
    let mut p = Readline::default()
        .title("Hi!")
        .enable_suggest(Suggest::from_iter([
            "apple",
            "applet",
            "application",
            "banana",
        ]))
        .validator(
            |text| text.len() > 10,
            |text| format!("Length must be over 10 but got {}", text.len()),
        )
        .prompt()?;
    println!("result: {:?}", p.run()?);
    Ok(())
}
```
</details>

<img src="https://github.com/ynqa/promkit/assets/6745370/d124268e-9496-4c4b-83be-c734e4d03591" width="50%" height="auto">

### Confirm

<details>
<summary>Command</summary>

```bash
cargo run --manifest-path examples/confirm/Cargo.toml
```

</details>

<details>
<summary>Code</summary>

```rust
use promkit::{preset::confirm::Confirm, Result};

fn main() -> Result {
    let mut p = Confirm::new("Do you have a pet?").prompt()?;
    println!("result: {:?}", p.run()?);
    Ok(())
}
```
</details>

<img src="https://github.com/ynqa/promkit/assets/6745370/ac9bac78-66cd-4653-a39f-6c9c0c24131f" width="50%" height="auto">

### Password

<details>
<summary>Command</summary>

```bash
cargo run --manifest-path examples/password/Cargo.toml
```

</details>

<details>
<summary>Code</summary>

```rust
use promkit::{preset::password::Password, Result};

fn main() -> Result {
    let mut p = Password::default()
        .title("Put your password")
        .validator(
            |text| 4 < text.len() && text.len() < 10,
            |text| format!("Length must be over 4 and within 10 but got {}", text.len()),
        )
        .prompt()?;
    println!("result: {:?}", p.run()?);
    Ok(())
}
```
</details>

<img src="https://github.com/ynqa/promkit/assets/6745370/396356ef-47de-44bc-a8d4-d03c7ac66a2f" width="50%" height="auto">

### Form

<details>
<summary>Command</summary>

```bash
cargo run --manifest-path examples/form/Cargo.toml
```

</details>

<details>
<summary>Code</summary>

```rust
use promkit::{crossterm::style::Color, preset::form::Form, style::StyleBuilder, text_editor};

fn main() -> anyhow::Result<()> {
    let mut p = Form::new([
        text_editor::State {
            texteditor: Default::default(),
            history: Default::default(),
            prefix: String::from("❯❯ "),
            mask: Default::default(),
            prefix_style: StyleBuilder::new().fgc(Color::DarkRed).build(),
            active_char_style: StyleBuilder::new().bgc(Color::DarkCyan).build(),
            inactive_char_style: StyleBuilder::new().build(),
            edit_mode: Default::default(),
            word_break_chars: Default::default(),
            lines: Default::default(),
        },
        text_editor::State {
            texteditor: Default::default(),
            history: Default::default(),
            prefix: String::from("❯❯ "),
            mask: Default::default(),
            prefix_style: StyleBuilder::new().fgc(Color::DarkGreen).build(),
            active_char_style: StyleBuilder::new().bgc(Color::DarkCyan).build(),
            inactive_char_style: StyleBuilder::new().build(),
            edit_mode: Default::default(),
            word_break_chars: Default::default(),
            lines: Default::default(),
        },
        text_editor::State {
            texteditor: Default::default(),
            history: Default::default(),
            prefix: String::from("❯❯ "),
            mask: Default::default(),
            prefix_style: StyleBuilder::new().fgc(Color::DarkBlue).build(),
            active_char_style: StyleBuilder::new().bgc(Color::DarkCyan).build(),
            inactive_char_style: StyleBuilder::new().build(),
            edit_mode: Default::default(),
            word_break_chars: Default::default(),
            lines: Default::default(),
        },
    ])
    .prompt()?;
    println!("result: {:?}", p.run()?);
    Ok(())
}
```

</details>

<img src="https://github.com/ynqa/promkit/assets/6745370/c3dc88a7-d0f0-42f4-90b8-bc4d2e23e36d" width="50%" height="auto">

### Listbox

<details>
<summary>Command</summary>

```bash
cargo run --manifest-path examples/listbox/Cargo.toml
```
</details>

<details>
<summary>Code</summary>

```rust
use promkit::{preset::listbox::Listbox, Result};

fn main() -> Result {
    let mut p = Listbox::new(0..100)
        .title("What number do you like?")
        .listbox_lines(5)
        .prompt()?;
    println!("result: {:?}", p.run()?);
    Ok(())
}
```
</details>

<img src="https://github.com/ynqa/promkit/assets/6745370/0da1b1d0-bb17-4951-8ea8-3b09cd2eb86a" width="50%" height="auto">

### QuerySelector

<details>
<summary>Command</summary>

```bash
cargo run --manifest-path examples/query_selector/Cargo.toml
```
</details>

<details>
<summary>Code</summary>

```rust
use promkit::{preset::query_selector::QuerySelector, Result};

fn main() -> Result {
    let mut p = QuerySelector::new(0..100, |text, items| -> Vec<String> {
        text.parse::<usize>()
            .map(|query| {
                items
                    .iter()
                    .filter(|num| query <= num.parse::<usize>().unwrap_or_default())
                    .map(|num| num.to_string())
                    .collect::<Vec<String>>()
            })
            .unwrap_or(items.clone())
    })
    .title("What number do you like?")
    .listbox_lines(5)
    .prompt()?;
    println!("result: {:?}", p.run()?);
    Ok(())
}
```
</details>

<img src="https://github.com/ynqa/promkit/assets/6745370/7ac2ed54-9f9e-4735-bffb-72f7cee06f6d" width="50%" height="auto">

### Checkbox

<details>
<summary>Command</summary>

```bash
cargo run --manifest-path examples/checkbox/Cargo.toml
```
</details>

<details>
<summary>Code</summary>

```rust
use promkit::{preset::checkbox::Checkbox, Result};

fn main() -> Result {
    let mut p = Checkbox::new(vec![
        "Apple",
        "Banana",
        "Orange",
        "Mango",
        "Strawberry",
        "Pineapple",
        "Grape",
        "Watermelon",
        "Kiwi",
        "Pear",
    ])
    .title("What are your favorite fruits?")
    .checkbox_lines(5)
    .prompt()?;
    println!("result: {:?}", p.run()?);
    Ok(())
}
```
</details>

<img src="https://github.com/ynqa/promkit/assets/6745370/350b16ce-6ef4-46f2-9466-d01b9dab4eaf" width="50%" height="auto">

### Tree

<details>
<summary>Command</summary>

```bash
cargo run --manifest-path examples/tree/Cargo.toml
```
</details>

<details>
<summary>Code</summary>

```rust
use promkit::{preset::tree::Tree, tree::Node, Result};

fn main() -> Result {
    let mut p = Tree::new(Node::try_from(&std::env::current_dir()?.join("src"))?)
        .title("Select a directory or file")
        .tree_lines(10)
        .prompt()?;
    println!("result: {:?}", p.run()?);
    Ok(())
}
```
</details>

<img src="https://github.com/ynqa/promkit/assets/6745370/61aefcd0-080a-443e-9dc6-ac627d306f55" width="50%" height="auto">

### JSON

<details>
<summary>Command</summary>

```bash
cargo run --manifest-path examples/json/Cargo.toml
```
</details>

<details>
<summary>Code</summary>

```rust
use promkit::{json::JsonStream, preset::json::Json, serde_json::Deserializer, Result};

fn main() -> Result {
    let stream = JsonStream::new(
        Deserializer::from_str(
            r#"{
              "number": 9,
              "map": {
                "entry1": "first",
                "entry2": "second"
              },
              "list": [
                "abc",
                "def"
              ]
            }"#,
        )
        .into_iter::<serde_json::Value>()
        .filter_map(serde_json::Result::ok),
        None,
    );

    let mut p = Json::new(stream)
        .title("JSON viewer")
        .json_lines(5)
        .prompt()?;
    println!("result: {:?}", p.run()?);
    Ok(())
}
```
</details>

<img src="https://github.com/ynqa/promkit/assets/6745370/751af3ae-5aff-45ca-8729-34cd004ee7d9" width="50%" height="auto">

## License

This project is licensed under the MIT License.
See the [LICENSE](https://github.com/ynqa/promkit/blob/main/LICENSE)
file for details.

## Stargazers over time
[![Stargazers over time](https://starchart.cc/ynqa/promkit.svg?variant=adaptive)](https://starchart.cc/ynqa/promkit)
