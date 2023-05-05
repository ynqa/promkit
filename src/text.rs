use std::io;

use crate::{crossterm::style, crossterm::terminal, grapheme::Graphemes, termutil, text, Result};

pub struct Store {
    pub state: text::State,
    pub renderer: text::Renderer,
}

/// Text specific state.
#[derive(Debug)]
pub struct State {
    pub text: Graphemes,
    pub text_color: style::Color,
}

impl Default for State {
    fn default() -> Self {
        Self {
            text: Default::default(),
            text_color: style::Color::Reset,
        }
    }
}

impl State {
    pub fn text_lines(&self) -> Result<u16> {
        termutil::num_lines(&self.text)
    }
}

pub struct Renderer {}

impl Renderer {
    pub fn render<W: io::Write>(&self, out: &mut W, state: &State) -> Result<()> {
        crossterm::execute!(
            out,
            terminal::Clear(terminal::ClearType::CurrentLine),
            style::SetForegroundColor(state.text_color),
            style::Print(
                state
                    .text
                    .iter()
                    .fold(String::new(), |s, g| format!("{}{}", s, g.ch))
            ),
            style::SetForegroundColor(style::Color::Reset),
        )
    }
}
