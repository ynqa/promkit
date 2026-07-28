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
promkit = { version = "0.15.0", features = ["runtime", "texteditor"] }
```

The widget states can also be used directly:

```toml
[dependencies]
promkit-widgets = { version = "0.8", features = ["json", "yaml"] }
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
| `status` | Status display |
| `serde` | Serde support for widget configuration |
| `all` | All features above |

## Responsibilities

Widgets manage state and project it into renderable content. They intentionally
do not own event loops or key bindings: application `Prompt` implementations
define input and focus behavior, and `promkit-core` handles terminal layout and
drawing.

See [Concept.md](../Concept.md) for the architecture and the repository
[examples](../examples/) for complete compositions.
