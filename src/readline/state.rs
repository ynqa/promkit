use std::fmt;
use std::io;

use crate::{
    crossterm::{style, terminal},
    grapheme::{Grapheme, Graphemes},
    internal::buffer::Buffer,
    internal::selector::history::History,
    suggest::Suggest,
    termutil, Result,
};

/// Edit mode.
#[derive(Debug, Clone)]
pub enum Mode {
    /// Insert a char at the current position.
    Insert,
    /// Overwrite a char at the current position.
    Overwrite,
}

/// Readline specific state.
#[derive(Debug)]
pub struct State {
    pub editor: Buffer,
    pub prev: Buffer,
    pub next: Buffer,
    pub title_lines: u16,
    /// A label as prompt (e.g. ">>").
    pub label: Graphemes,
    pub label_color: style::Color,
    /// A char to mask the input chars (e.g. "*"),
    /// for example when you type the passwords.
    pub mask: Option<Grapheme>,
    pub edit_mode: Mode,
    /// How many lines to receive the user input string.
    pub num_lines: Option<u16>,
    pub hstr: Option<History>,
    pub suggest: Option<Suggest>,
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.editor.data)
    }
}

impl State {
    pub fn buffer_lines(&self) -> Result<u16> {
        let left_space = terminal::size()?.1 - self.title_lines;
        Ok(self.num_lines.unwrap_or(left_space))
    }

    pub fn buffer_limit(&self) -> Result<u16> {
        Ok(terminal::size()?.0 * self.buffer_lines()? - self.label.width() as u16 - 1)
    }

    pub fn render_static<W: io::Write>(&mut self, out: &mut W) -> Result<()> {
        // Render the label.
        crossterm::execute!(
            out,
            style::SetForegroundColor(self.label_color),
            style::Print(self.label.to_owned()),
            style::SetForegroundColor(style::Color::Reset),
        )
    }

    pub fn render<W: io::Write>(&mut self, out: &mut W) -> Result<()> {
        let (mut prev, mut next) = (self.prev.clone(), self.next.clone());
        if prev.data == next.data {
            return Ok(());
        }

        // Masking.
        prev.data = match &self.mask {
            None => prev.data.clone(),
            Some(mask) => prev
                .data
                .iter()
                .map(|_| mask.clone())
                .collect::<Graphemes>(),
        };
        next.data = match &self.mask {
            None => next.data.clone(),
            Some(mask) => next
                .data
                .iter()
                .map(|_| mask.clone())
                .collect::<Graphemes>(),
        };

        // Go backward/forward to the position of lcp.
        let lcp = prev.data.longest_common_prefix(&next.data);
        if lcp.width() > prev.width_to_position() {
            termutil::move_right(out, (lcp.width() - prev.width_to_position()) as u16)?;
        } else {
            termutil::move_left(out, (prev.width_to_position() - lcp.width()) as u16)?;
        }

        // Render the suffix of next buffer.
        let mut input = next
            .data
            .iter()
            .enumerate()
            .filter(|&(i, _)| i >= lcp.len())
            .fold(Graphemes::default(), |mut g, (_, ch)| {
                g.push(ch.clone());
                g
            });

        // FromCursorDown remove the last char
        // if the cursor is at the end of line.
        // Therefore, put the space the last of input string.
        input.push(Grapheme::from(' '));
        crossterm::execute!(
            out,
            style::Print(input),
            terminal::Clear(terminal::ClearType::FromCursorDown),
        )?;

        // Go backward to the next position from the end of graphemes.
        // +1 is for input.push(Grapheme::from(' ')) step above.
        termutil::move_left(out, next.width_from_position() as u16 + 1)
    }
}
