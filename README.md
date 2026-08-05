# promkit

[![ci](https://github.com/ynqa/promkit/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/ynqa/promkit/actions/workflows/ci.yml)
[![docs.rs](https://img.shields.io/docsrs/promkit)](https://docs.rs/promkit)

A toolkit for building your own interactive prompt in Rust.

## Getting Started

Put the package in your `Cargo.toml`.

```toml
[dependencies]
promkit = { version = "0.15.0", features = ["runtime", "texteditor"] }
```

## Features

- **Application-owned composition** — Implement the
  [`Prompt`](./promkit/src/runtime.rs) lifecycle and combine only the widget
  states an application needs. Key bindings, focus, validation, background
  work, and other event policy remain in the application instead of
  framework-owned presets.
- **Optional runtime and terminal lifecycle** — Use the asynchronous prompt
  runtime when useful, and independently manage raw mode, the alternate screen,
  cursor visibility, and mouse capture with
  [`TerminalSession`](./promkit/src/terminal_session.rs).
- **Viewport-aware rendering** — [`promkit-core`](./promkit-core/) handles
  wrapping or truncation, vertical pane allocation, cursor scrolling, clipping,
  resizing, and screen-to-widget hit testing.
- **Reusable widget states** — [`promkit-widgets`](./promkit-widgets/) provides
  text editing and display, list and checkbox selection, prefix search,
  spinners, trees, JSON and YAML documents, and CSV tables.
- **Efficient large-content projection** — JSON, YAML, and table widgets can
  project only the visible terminal viewport instead of rebuilding all content
  on every redraw.
- **Feature-gated modules** — No Cargo features are enabled by default. Choose
  the runtime, terminal session, capabilities, and individual widgets needed by
  the application.
- **Cross-platform terminal support** — UNIX and Windows are supported through
  [crossterm](https://github.com/crossterm-rs/crossterm).

## Concept

See [here](./Concept.md).

## Projects using *promkit*

- [ynqa/empiriqa](https://github.com/ynqa/empiriqa)
- [ynqa/jnv](https://github.com/ynqa/jnv)
- [ynqa/logu](https://github.com/ynqa/logu)
- [ynqa/sig](https://github.com/ynqa/sig)

## Examples/Demos

*promkit* provides a runtime and reusable widgets for building interactive
terminal applications. The examples own their event policies and demonstrate
how applications can compose the widgets they need.

### Table of contents

| Example | Description |
| --- | --- |
| [Readline](#readline) | Single-line editing with history and completion |
| [Confirm](#confirm) | Validated yes-or-no input |
| [Password](#password) | Masked input with validation |
| [Form](#form) | Multiple editable fields in one prompt |
| [Listbox](#listbox) | Keyboard-driven single selection |
| [QuerySelector](#queryselector) | Prefix search over selectable candidates |
| [Checkbox](#checkbox) | Keyboard-driven multiple selection |
| [Tree](#tree) | Navigation and folding for hierarchical data |
| [JSON](#json) | Line-numbered navigation and folding for JSON |
| [YAML](#yaml) | Line-numbered navigation and folding for YAML |
| [CSV](#csv) | Vertical and horizontal navigation for tabular data |
| [Text](#text) | Scrollable styled text |
| [Async Task with Spinner](#async-task-with-spinner) | Background work with live input and progress feedback |
| [Multiline REPL](#multiline-repl) | Multiline editing in an interactive loop |

Each section includes a command, source link, and recorded demo.

### Readline

<details>
<summary>Command</summary>

```bash
cargo run --bin readline
```

</details>

[Composition](./examples/readline/src/lib.rs) /
[Usage](./examples/readline/src/readline.rs)

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
cargo run --bin query-selector
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

### YAML

<details>
<summary>Command</summary>

```bash
cargo run --bin yaml ${PATH_TO_YAML_FILE}
```
</details>

[Code](./examples/yaml/src/yaml.rs)

<img src="https://github.com/ynqa/ynqa/blob/master/demo/promkit/yaml.gif" width="50%" height="auto">

### CSV

<details>
<summary>Command</summary>

```bash
cargo run --bin csv ${PATH_TO_CSV_FILE}
```
</details>

[Code](./examples/csv/src/csv.rs)

<img src="https://github.com/ynqa/ynqa/blob/master/demo/promkit/csv.gif" width="50%" height="auto">

### Text

<details>
<summary>Command</summary>

```bash
cargo run --bin text
```

</details>

[Code](./examples/text/src/text.rs)

<img src="https://github.com/ynqa/ynqa/blob/master/demo/promkit/text.gif" width="50%" height="auto">

## Advanced Examples

These examples demonstrate how to compose widgets and rendering primitives for
application-specific event loops and asynchronous workflows.

### Async Task with Spinner

<details>
<summary>Command</summary>

```bash
cargo run --bin async_task
```

</details>

[Code](./examples/async_task/src/async_task.rs)

<img src="https://github.com/ynqa/ynqa/blob/master/demo/promkit/async_task.gif" width="50%" height="auto">

### Multiline REPL

<details>
<summary>Command</summary>

```bash
cargo run --bin repl
```

</details>

[Code](./examples/repl/src/repl.rs)

<img src="https://github.com/ynqa/ynqa/blob/master/demo/promkit/repl.gif" width="50%" height="auto">

## License

[MIT License](./LICENSE)
