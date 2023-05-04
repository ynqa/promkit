use std::io;

use crate::{
    crossterm::{style, terminal},
    grapheme::Graphemes,
    termutil, Result,
};

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

    pub fn render<W: io::Write>(&mut self, out: &mut W) -> Result<()> {
        crossterm::execute!(
            out,
            terminal::Clear(terminal::ClearType::CurrentLine),
            style::SetForegroundColor(self.text_color),
            style::Print(
                self.text
                    .iter()
                    .fold(String::new(), |s, g| format!("{}{}", s, g.ch))
            ),
            style::SetForegroundColor(style::Color::Reset),
        )?;
        // Move to next line.
        termutil::move_down(out, 1)
    }
}
