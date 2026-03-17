use promkit_core::{
    Widget,
    crossterm::style::{Color, ContentStyle},
    grapheme::StyledGraphemes,
};

use crate::text::{State as TextState, Text};

/// Represents status levels shown by a wrapped text widget.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Severity {
    #[default]
    Success,
    Warning,
    Error,
}

impl Severity {
    pub fn style(self) -> ContentStyle {
        ContentStyle {
            foreground_color: Some(match self {
                Self::Success => Color::Green,
                Self::Warning => Color::Yellow,
                Self::Error => Color::Red,
            }),
            ..Default::default()
        }
    }
}

/// Wraps `text::State` and applies a color style based on severity.
#[derive(Clone)]
pub struct State {
    pub text: TextState,
    pub severity: Severity,
}

impl Default for State {
    fn default() -> Self {
        Self::new("", Severity::Success)
    }
}

impl State {
    pub fn new<T: AsRef<str>>(text: T, severity: Severity) -> Self {
        let mut state = Self {
            text: TextState {
                text: Text::from(text),
                ..Default::default()
            },
            severity,
        };
        state.text.config.style = Some(state.severity.style());
        state
    }
}

impl Widget for State {
    fn create_graphemes(&self, width: u16, height: u16) -> StyledGraphemes {
        self.text.create_graphemes(width, height)
    }
}
