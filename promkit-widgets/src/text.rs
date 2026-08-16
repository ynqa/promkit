use promkit_core::{
    ContentPosition, CreatedGraphemes, Widget, WidgetLayout, WidthMode, grapheme::StyledGraphemes,
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
                width_mode: match self.config.overflow_mode {
                    config::OverflowMode::Truncate => WidthMode::Truncate,
                    config::OverflowMode::Wrap => WidthMode::Wrap,
                },
                ..Default::default()
            },
            cursor: self
                .text
                .current_line()
                .map(|current_line| ContentPosition {
                    row: current_line,
                    column: 0,
                }),
        }
    }
}

impl State {
    /// Interprets a text content position as a selectable line.
    ///
    /// Wrapped visual rows are normalized to their logical content row by the
    /// core renderer before this method resolves the line index.
    pub fn hit_at(&self, position: ContentPosition) -> Option<TextHit> {
        self.text
            .items()
            .get(position.row)
            .map(|_| TextHit::Select {
                index: position.row,
            })
    }
}

/// Semantic targets exposed by the text widget.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextHit {
    Select { index: usize },
}

#[cfg(test)]
mod tests {
    use super::*;

    mod state {
        use super::*;

        mod create_graphemes {
            use super::*;

            #[test]
            fn uses_configured_overflow_mode() {
                for (overflow_mode, width_mode) in [
                    (config::OverflowMode::Truncate, WidthMode::Truncate),
                    (config::OverflowMode::Wrap, WidthMode::Wrap),
                ] {
                    let state = State {
                        text: Text::from("text"),
                        config: Config {
                            overflow_mode,
                            ..Default::default()
                        },
                    };

                    assert_eq!(state.create_graphemes().layout.width_mode, width_mode);
                }
            }
        }

        mod hit_at {
            use super::*;

            #[test]
            fn resolves_line_rows() {
                let state = State {
                    text: Text::from("first\nsecond"),
                    config: Config::default(),
                };

                assert_eq!(
                    state.hit_at(ContentPosition { row: 1, column: 20 }),
                    Some(TextHit::Select { index: 1 })
                );
                assert_eq!(state.hit_at(ContentPosition { row: 2, column: 0 }), None);
            }
        }
    }
}
