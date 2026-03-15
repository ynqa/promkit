# promkit

[![ci](https://github.com/ynqa/promkit/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/ynqa/promkit/actions/workflows/ci.yml)
[![docs.rs](https://img.shields.io/docsrs/promkit)](https://docs.rs/promkit)

A toolkit for building your own interactive prompt in Rust.

## Getting Started

Put the package in your `Cargo.toml`.

```toml
[dependencies]
promkit = "0.12.0"
```

## Features

- Cross-platform support for both UNIX and Windows utilizing [crossterm](https://github.com/crossterm-rs/crossterm)
- Modularized architecture
  - [promkit-core](./promkit-core/)
    - Core functionality for terminal rendering and keyed grapheme chunk management
  - [promkit-widgets](./promkit-widgets/)
    - Various UI components (text, listbox, tree, etc.)
  - [promkit](./promkit/)
    - High-level presets and user interfaces
  - [promkit-derive](./promkit-derive/)
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

See [here](./Concept.md).

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
cargo run --bin readline
```

</details>

[Code](./examples/readline/src/readline.rs)

<img src="https://github.com/ynqa/ynqa/blob/master/demo/promkit/readline.gif" width="50%" height="auto">

### Confirm

<details>
<summary>Command</summary>

```bash
cargo run --bin confirm
```

</details>

[Code](./examples/confirm/src/confirm.rs)

<img src="https://github.com/ynqa/ynqa/blob/master/demo/promkit/confirm.gif" width="50%" height="auto">

### Password

<details>
<summary>Command</summary>

```bash
cargo run --bin password
```

</details>

[Code](./examples/password/src/password.rs)

<img src="https://github.com/ynqa/ynqa/blob/master/demo/promkit/password.gif" width="50%" height="auto">

### Form

<details>
<summary>Command</summary>

```bash
cargo run --bin form
```

</details>

[Code](./examples/form/src/form.rs)

<img src="https://github.com/ynqa/ynqa/blob/master/demo/promkit/form.gif" width="50%" height="auto">

### Listbox

<details>
<summary>Command</summary>

```bash
cargo run --bin listbox
```
</details>

[Code](./examples/listbox/src/listbox.rs)

<img src="https://github.com/ynqa/ynqa/blob/master/demo/promkit/listbox.gif" width="50%" height="auto">

### QuerySelector

<details>
<summary>Command</summary>

```bash
cargo run --bin query_selector
```
</details>

[Code](./examples/query_selector/src/query_selector.rs)

<img src="https://github.com/ynqa/ynqa/blob/master/demo/promkit/query_selector.gif" width="50%" height="auto">

### Checkbox

<details>
<summary>Command</summary>

```bash
cargo run --bin checkbox
```
</details>

[Code](./examples/checkbox/src/checkbox.rs)

<img src="https://github.com/ynqa/ynqa/blob/master/demo/promkit/checkbox.gif" width="50%" height="auto">

### Tree

<details>
<summary>Command</summary>

```bash
cargo run --bin tree
```
</details>

[Code](./examples/tree/src/tree.rs)

<img src="https://github.com/ynqa/ynqa/blob/master/demo/promkit/tree.gif" width="50%" height="auto">

### JSON

<details>
<summary>Command</summary>

```bash
cargo run --bin json ${PATH_TO_JSON_FILE}
```
</details>

[Code](./examples/json/src/json.rs)

<img src="https://github.com/ynqa/ynqa/blob/master/demo/promkit/json.gif" width="50%" height="auto">

### Text

<details>
<summary>Command</summary>

```bash
cargo run --bin text
```

</details>

[Code](./examples/text/src/text.rs)

<img src="https://github.com/ynqa/ynqa/blob/master/demo/promkit/text.gif" width="50%" height="auto">

## License

[MIT License](./LICENSE)

## Stargazers over time
[![Stargazers over time](https://starchart.cc/ynqa/promkit.svg?variant=adaptive)](https://starchart.cc/ynqa/promkit)
