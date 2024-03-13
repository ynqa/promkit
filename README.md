# promkit

[![ci](https://github.com/ynqa/promkit/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/ynqa/promkit/actions/workflows/ci.yml)
[![docs.rs](https://img.shields.io/docsrs/promkit)](https://docs.rs/promkit)

A toolkit for building your own interactive prompt in Rust.

## Getting Started

Put the package in your `Cargo.toml`.

```toml
[dependencies]
promkit = "0.3.0"
```

## Features

- Support cross-platform both UNIX and Windows owing to
[crossterm](https://github.com/crossterm-rs/crossterm)
- Various building methods
  - Preset; Support for quickly setting up a UI by providing simple parameters
    - [Readline](#readline)
    - [Confirm](#confirm)
    - [Password](#password)
    - [Listbox](#listbox)
    - [QuerySelector](#queryselector)
    - [Checkbox](#checkbox)
    - [Tree](#tree)
    - [JSON](#json)
  - Combining various UI components
    - They are provided with the same interface, allowing users to choose and
      assemble them according to their preferences
  - (Upcoming) Stronger support to display yor own data structures
- Versatile customization capabilities
  - Theme for designing the appearance of the prompt
    - e.g. cursor, text and prompt string
  - Validation for user input and error message construction
  - Customizable key mappings
- Mouse support (partially)
  - Allows scrolling through lists with the mouse wheel

## Examples/Demos

*promkit* provides presets so that users can try prompts immediately without
having to build complex components for specific use cases.

Show you commands, code, and actual demo screens for examples
that can be executed immediately below.

### Readline

<details>
<summary>Command</summary>

```bash
cargo run --example readline
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

![readline](https://github.com/ynqa/promkit/assets/6745370/d124268e-9496-4c4b-83be-c734e4d03591)

### Confirm

<details>
<summary>Command</summary>

```bash
cargo run --example confirm
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

![confirm](https://github.com/ynqa/promkit/assets/6745370/ac9bac78-66cd-4653-a39f-6c9c0c24131f)

### Password

<details>
<summary>Command</summary>

```bash
cargo run --example password
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

![password](https://github.com/ynqa/promkit/assets/6745370/396356ef-47de-44bc-a8d4-d03c7ac66a2f)

### Listbox

<details>
<summary>Command</summary>

```bash
cargo run --example listbox
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

![listbox](https://github.com/ynqa/promkit/assets/6745370/0da1b1d0-bb17-4951-8ea8-3b09cd2eb86a)

### QuerySelector

<details>
<summary>Command</summary>

```bash
cargo run --example query_selector
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

![query_selector](https://github.com/ynqa/promkit/assets/6745370/7ac2ed54-9f9e-4735-bffb-72f7cee06f6d)

### Checkbox

<details>
<summary>Command</summary>

```bash
cargo run --example checkbox
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

![checkbox](https://github.com/ynqa/promkit/assets/6745370/350b16ce-6ef4-46f2-9466-d01b9dab4eaf)

### Tree

<details>
<summary>Command</summary>

```bash
cargo run --example tree
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

![tree](https://github.com/ynqa/promkit/assets/6745370/624cd902-5362-4baf-ad5a-f3478ed6b579)

### JSON

<details>
<summary>Command</summary>

```bash
cargo run --example json
```
</details>

<details>
<summary>Code</summary>

```rust
use promkit::{json::JsonNode, preset::json::Json, Result};

fn main() -> Result {
    let mut p = Json::new(JsonNode::try_from(
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
    )?)
    .title("JSON viewer")
    .json_lines(5)
    .prompt()?;
    println!("result: {:?}", p.run()?);
    Ok(())
}
```
</details>

![json](https://github.com/ynqa/promkit/assets/6745370/751af3ae-5aff-45ca-8729-34cd004ee7d9)

## Why *promkit*?

Related libraries in this category include the following:
- [console-rs/dialoguer](https://github.com/console-rs/dialoguer)
- [mikaelmello/inquire](https://github.com/mikaelmello/inquire/tree/main/inquire)

*promkit* offers several advantages over these libraries:

### Unified interface approach for UI components

*promkit* takes a unified approach by having all of its components inherit the
same `Renderer` trait. This design choice enables users to seamlessly support
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
  pub trait Renderer {
      fn create_panes(&self, width: u16) -> Vec<Pane>;
  }
  ```

### Variety of Pre-built UI Preset Components

One of the compelling reasons to choose *promkit* is its extensive range of pre-built UI preset components.
These presets allow developers to quickly implement various interactive prompts without the need to design and
build each component from scratch. The availability of these presets not only speeds up the development process
but also ensures consistency and reliability across different applications.
Here are some of the preset components available, see [Examples](#examplesdemos)

### Resilience to terminal resizing

Performing operations that involve executing a command in one pane while
simultaneously opening a new pane is a common occurrence. During such operations,
if UI corruption is caused by resizing the terminal size, it may adversely affect
the user experience.  
Other libraries can struggle when the terminal is resized, making typing and
interaction difficult or impossible. For example:

 - [(console-rs/dialoguer) Automatic re-render on terminal window resize](https://github.com/console-rs/dialoguer/issues/178)

*promkit* introduces a step to align data with the screen size before rendering.
This approach ensures consistency in UI elements even when
the terminal size changes, providing a smoother user experience.

## License

This project is licensed under the MIT License.
See the [LICENSE](https://github.com/ynqa/promkit/blob/main/LICENSE)
file for details.

## Stargazers over time
[![Stargazers over time](https://starchart.cc/ynqa/promkit.svg?variant=adaptive)](https://starchart.cc/ynqa/promkit)
