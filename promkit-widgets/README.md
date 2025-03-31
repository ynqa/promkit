# promkit-widgets

## Conventions (rough guidelines)

- Each widget contains a single `State`
- All `State` must implement
   [PaneFactory](https://github.com/ynqa/promkit/blob/v0.1.0/promkit-core/src/pane.rs)
- Place files that represent state implementations at the same level as lib.rs
   - e.g. [Text Editor](https://github.com/ynqa/promkit/blob/v0.1.0/promkit-widgets/src/text_editor.rs)
