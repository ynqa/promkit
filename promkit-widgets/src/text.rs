use promkit_core::{
    ContentPosition, CreatedGraphemes, Widget, WidgetLayout, grapheme::StyledGraphemes,
};

#[path = "text/text.rs"]
mod inner;
pub use inner::Text;
pub mod config;
pub use config::Config;

/// Represents the state of a text-based component within the application.
///
/// This state encapsulates the properties and
/// behaviors specific to text handling,
#[derive(Clone, Default)]
pub struct State {
    /// The text to be rendered.
    pub text: Text,
    /// Configuration for rendering and behavior.
    pub config: Config,
}

impl State {
    pub fn replace(&mut self, state: Self) {
        *self = state;
    }

    pub fn replace_text(&mut self, text: Vec<StyledGraphemes>) {
        self.text.replace_contents(text);
    }
}

impl Widget for State {
    fn create_graphemes(&self) -> CreatedGraphemes {
        let lines = self.text.items().iter().map(|item| {
            if let Some(style) = &self.config.style {
                item.clone().apply_style(*style)
            } else {
                item.clone()
            }
        });

        CreatedGraphemes {
            graphemes: StyledGraphemes::from_lines(lines),
            layout: WidgetLayout {
                max_height: self.config.lines,
                ..Default::default()
            },
            cursor: (!self.text.items().is_empty()).then_some(ContentPosition {
                row: self.text.position(),
                column: 0,
            }),
        }
    }
}
