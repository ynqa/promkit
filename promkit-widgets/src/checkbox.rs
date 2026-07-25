use promkit_core::{
    ContentPosition, CreatedGraphemes, Widget, WidgetLayout, grapheme::StyledGraphemes,
};

#[path = "checkbox/checkbox.rs"]
mod inner;
pub use inner::Checkbox;
pub mod config;
pub use config::Config;

/// Represents the state of a `Checkbox` component.
///
/// This state includes not only the checkbox itself but also various attributes
/// that determine how the checkbox and its items are displayed. These attributes
/// include symbols for indicating active and inactive items, styles for selected
/// and unselected lines, and the number of lines available for rendering.
#[derive(Clone)]
pub struct State {
    /// The `Checkbox` component to be rendered.
    pub checkbox: Checkbox,

    /// Configuration for rendering and behavior.
    pub config: Config,
}

impl Widget for State {
    fn create_graphemes(&self) -> CreatedGraphemes {
        let f = |idx: usize| -> StyledGraphemes {
            if self.checkbox.picked_indexes().contains(&idx) {
                StyledGraphemes::from(format!("{} ", self.config.active_mark))
            } else {
                StyledGraphemes::from(format!("{} ", self.config.inactive_mark))
            }
        };

        let lines = self.checkbox.items().iter().enumerate().map(|(i, item)| {
            if i == self.checkbox.position() {
                StyledGraphemes::from_iter([
                    &StyledGraphemes::from(&self.config.cursor),
                    &f(i),
                    item,
                ])
                .apply_style(self.config.active_item_style)
            } else {
                StyledGraphemes::from_iter([
                    &StyledGraphemes::from(
                        " ".repeat(StyledGraphemes::from(&self.config.cursor).widths()),
                    ),
                    &f(i),
                    item,
                ])
                .apply_style(self.config.inactive_item_style)
            }
        });

        CreatedGraphemes {
            graphemes: StyledGraphemes::from_lines(lines),
            layout: WidgetLayout {
                max_height: self.config.lines,
                ..Default::default()
            },
            cursor: (!self.checkbox.items().is_empty()).then_some(ContentPosition {
                row: self.checkbox.position(),
                column: 0,
            }),
        }
    }
}
