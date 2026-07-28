# promkit-widgets

Reusable widget states for [promkit](https://github.com/ynqa/promkit).

Each state implements `promkit_core::Widget` and projects its current state into
styled graphemes and layout hints. Widgets do not own event loops or key
bindings: event handling belongs to application `Prompt` implementations,
while terminal layout and drawing belong to `promkit-core`.
See [Concept.md](../Concept.md) for the full responsibility boundaries.

## Features

No widget is enabled by default.

| Feature | Widget or capability |
| --- | --- |
| `checkbox` | Checkbox selection; enables `listbox` |
| `json` | Navigable JSON document |
| `yaml` | Navigable YAML document |
| `listbox` | List selection |
| `prefixsearch` | Prefix-matched candidate selection backed by a radix trie |
| `spinner` | Asynchronous spinner; enables Tokio |
| `status` | Status display; enables `text` |
| `text` | Styled text |
| `texteditor` | Editable text with history |
| `tree` | Navigable structured tree |
| `serde` | Serde support for widget configuration |
| `all` | All features above |

Enable only the widgets an application uses:

```toml
[dependencies]
promkit-widgets = { version = "0.7", features = ["json", "yaml"] }
```

The crate re-exports `promkit-core` as `promkit_widgets::core`. JSON and YAML
states provide viewport-bounded projection so callers do not need to materialize
every visible row of a large document on each cursor movement.

## Structured document loading

JSON and YAML documents can be built directly from strings or readers without
first materializing a `serde_json::Value` or `serde_yaml::Value` tree:

```rust
use std::{fs::File, io::BufReader};

use promkit_widgets::{json, yaml};

let json_document = json::Document::from_str(r#"{"name":"alice"}"#).unwrap();
let yaml_file = File::open("input.yaml").unwrap();
let yaml_document = yaml::Document::from_reader(BufReader::new(yaml_file)).unwrap();
```

`Document::new` remains available when an application already has deserialized
Serde values.

## Structured benchmark

The Criterion benchmark covers file reading, deserialization, document
construction, cursor movement, and viewport projection for the bundled JSON and
YAML fixtures:

```bash
cargo bench -p promkit-widgets --bench structured --features json,yaml
```

Criterion compares a run with its previous local result. Named baselines can be
saved and compared across revisions with `--save-baseline NAME` and
`--baseline NAME`. The benchmark currently reports regressions but does not
enforce a CI failure threshold.

Override either fixture when needed:

```bash
PROMKIT_STRUCTURED_JSON=/path/to/input.json \
PROMKIT_STRUCTURED_YAML=/path/to/input.yaml \
cargo bench -p promkit-widgets --bench structured --features json,yaml
```

This benchmark exercises the `promkit-widgets` projection layer. It does not
measure `promkit-core::Renderer`, terminal layout, or terminal I/O.

## Structured line numbers

The `json`, `yaml`, and `tree` widgets can display stable, one-based line
numbers by enabling `show_line_numbers` in their `Config`. Numbers refer to the
fully expanded structure, so collapsing a node leaves gaps for its hidden rows.

Applications configure line numbers directly on widget state:

```rust
let state = json::State {
    document,
    config: json::Config {
        show_line_numbers: true,
        ..Default::default()
    },
};
```
