# promkit-widgets

Reusable widget states for building interactive terminal applications with
[promkit](https://github.com/ynqa/promkit).

`promkit-widgets` provides the state and view projection for common UI
components. Each widget implements `promkit_core::Widget` and turns its current
state into styled graphemes and layout hints for `promkit-core` to render.

## Getting started

Widgets are opt-in Cargo features; none are enabled by default. Applications
using the `promkit` runtime can enable them through the main crate:

```toml
[dependencies]
promkit = { version = "0.17.0", features = ["runtime", "texteditor"] }
```

The widget states can also be used directly:

```toml
[dependencies]
promkit-widgets = { version = "0.10", features = ["json", "yaml"] }
```

`promkit` re-exports this crate as `promkit::widgets`, while
`promkit-widgets` re-exports `promkit-core` as `promkit_widgets::core`.

## Widgets

| Feature | Widget or capability |
| --- | --- |
| `checkbox` | Multiple-choice selection |
| `listbox` | List selection |
| `prefixsearch` | Prefix-matched candidate selection |
| `json` | Navigable JSON documents |
| `yaml` | Navigable YAML documents |
| `tree` | Navigable tree structures |
| `table` | Tabular CSV data |
| `text` | Styled text |
| `texteditor` | Editable text with history |
| `spinner` | Asynchronous progress display |
| `serde` | Serde support for widget configuration |
| `all` | All features above |

## JSON and YAML child counts

JSON and YAML documents retain the number of immediate array/sequence elements
or object/mapping entries, independently of folding. Query a container with
`Document::child_count(row_index)`, using a zero-based index into `Document::rows()`.
Empty containers return `Some(0)`; closing rows return the opening container's
count. Scalars, YAML document separators, and invalid indices return `None`.

Enable annotations through the widget configuration:

```rust
use promkit_widgets::json::{Config, Document, State};

let mut state = State {
    document: Document::from_str(r#"{"items": [1, 2, 3]}"#)?,
    config: Config {
        show_child_count: true,
        ..Default::default()
    },
};
assert_eq!(state.document.child_count(1), Some(3));
state.document.toggle_at(1);
// The items row now displays: "items": […] (3 items)
```

The same APIs are available under `promkit_widgets::yaml`. Annotations appear
beside collapsed or empty containers, for example `{…} (2 keys)` and `[] (0 items)`.
`show_child_count` defaults to `false`; `child_count_style` controls annotation
styling. Changing these settings does not fold or expand containers. Existing
`toggle()` / `toggle_at()` operations control folding. Pretty-printed data exports
do not include annotations.

When constructing `ContainerNode::Open` directly, supply its new `child_count`
field. Existing exhaustive `Open` patterns must include the field or `..`.
For `Config` struct literals, use `..Default::default()` or supply the new fields;
stored Serde configurations without these fields keep the default display.

## Responsibilities

Widgets manage state and project it into renderable content. They intentionally
do not own event loops or key bindings: application `Prompt` implementations
define input and focus behavior, and `promkit-core` handles terminal layout and
drawing.

See [Concept.md](../Concept.md) for the architecture and the repository
[examples](../examples/) for complete compositions.
