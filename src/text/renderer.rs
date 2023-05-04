use std::io;

use crate::{
    crossterm::{style, terminal},
    text::State,
    Result,
};

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
