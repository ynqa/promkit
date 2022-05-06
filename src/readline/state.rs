use std::io;

use crossterm::{style, terminal};

use crate::{
    edit::{Buffer, History, Suggest},
    grapheme::{Grapheme, Graphemes},
    state, termutil, Output, Result,
};

/// Edit mode.
#[derive(Debug, Clone)]
pub enum Mode {
    /// Insert a char at the current position.
    Insert,
    /// Overwrite a char at the current position.
    Overwrite,
}

pub type State = state::State<Buffer, With>;

/// Readline specific state.
#[derive(Debug)]
pub struct With {
    /// A label as prompt (e.g. ">>").
    pub label: Graphemes,
    pub label_color: style::Color,
    /// A char to mask the input chars (e.g. "*"),
    /// for example when you type the passwords.
    pub mask: Option<Grapheme>,
    pub edit_mode: Mode,
    pub num_lines: Option<usize>,
    pub hstr: Option<Box<History>>,
    pub suggest: Option<Box<Suggest>>,
}

impl Output for State {
    type Output = String;

    fn output(&self) -> Self::Output {
        self.0.editor.data.to_string()
    }
}

// TODO: multi-line inputs.
// TODO: select range of buffer and replace string to the additional input.
// TODO: input validation.
impl<W: io::Write> state::Render<W> for State {
    fn pre_render(&self, out: &mut W) -> Result<()> {
        crossterm::execute!(
            out,
            style::SetForegroundColor(self.1.label_color),
            style::Print(self.1.label.to_owned()),
            style::SetForegroundColor(style::Color::Reset),
        )
    }

    fn render(&mut self, out: &mut W) -> Result<()> {
        if let Some((mut prev, mut next)) = self.0.input_stream.pop() {
            // Masking.
            prev.data = match &self.1.mask {
                None => prev.data.clone(),
                Some(mask) => prev
                    .data
                    .iter()
                    .map(|_| mask.clone())
                    .collect::<Graphemes>(),
            };
            next.data = match &self.1.mask {
                None => next.data.clone(),
                Some(mask) => next
                    .data
                    .iter()
                    .map(|_| mask.clone())
                    .collect::<Graphemes>(),
            };

            // Go backward/forward to the position of lcp.
            let lcp = prev.data.longest_common_prefix(&next.data);
            if lcp.width() > prev.width_to_pos() {
                termutil::move_right(out, (lcp.width() - prev.width_to_pos()) as u16)?;
            } else {
                termutil::move_left(out, (prev.width_to_pos() - lcp.width()) as u16)?;
            }

            // Render the suffix of next buffer.
            crossterm::execute!(out, terminal::Clear(terminal::ClearType::FromCursorDown))?;
            let input = next
                .data
                .iter()
                .enumerate()
                .filter(|&(i, _)| i >= lcp.len())
                .fold(Graphemes::default(), |mut g, (_, ch)| {
                    g.push(ch.clone());
                    g
                });
            crossterm::execute!(
                out,
                style::Print(
                    input
                        .iter()
                        .fold(String::new(), |s, g| format!("{}{}", s, g.ch))
                )
            )?;

            // Go backward to the next position from the end of graphemes.
            termutil::move_left(out, next.width_from_pos() as u16)?;
        }
        Ok(())
    }
}

impl State {
    pub fn buffer_limit(&self) -> Result<Option<usize>> {
        if let Some(lines) = self.1.num_lines {
            if lines > 0 {
                return Ok(Some(
                    terminal::size()?.0 as usize * lines as usize - self.1.label.width(),
                ));
            }
        }
        Ok(None)
    }
}
