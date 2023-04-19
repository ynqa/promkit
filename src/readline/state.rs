use std::cmp::Ordering;
use std::io;

use crate::{
    crossterm::{cursor, style, terminal},
    grapheme::{Grapheme, Graphemes},
    internal::buffer::Buffer,
    internal::selector::history::History,
    suggest::Suggest,
    termutil, Output, Result,
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
    /// Title displayed on the initial line.
    pub title: Option<Graphemes>,
    pub title_color: Option<style::Color>,
    /// A label as prompt (e.g. ">>").
    pub label: Graphemes,
    pub label_color: style::Color,
    /// A char to mask the input chars (e.g. "*"),
    /// for example when you type the passwords.
    pub mask: Option<Grapheme>,
    pub edit_mode: Mode,
    /// How many lines to receive the user input string.
    pub num_lines: Option<usize>,
    /// Minimum length of chars to start searching.
    pub min_len_to_search: usize,
    /// How many inputs to be stored into history.
    pub limit_history_size: Option<usize>,
    pub hstr: Option<History>,
    pub suggest: Option<Suggest>,
}

impl Output for State {
    type Output = String;

    fn output(&self) -> Self::Output {
        self.editor.data.to_string()
    }
}

impl State {
    pub fn pre_render<W: io::Write>(&self, out: &mut W) -> Result<()> {
        // Render the title.
        if let Some(title) = &self.title {
            if let Some(color) = self.title_color {
                crossterm::execute!(out, style::SetForegroundColor(color))?;
            }
            crossterm::execute!(out, style::Print(title), cursor::MoveToNextLine(1))?;
            if self.title_color.is_some() {
                crossterm::execute!(out, style::SetForegroundColor(style::Color::Reset))?;
            }
            if termutil::compare_cursor_position(termutil::Boundary::Bottom)? == Ordering::Equal {
                let title_lines =
                    termutil::num_lines(self.title.as_ref().unwrap_or(&Graphemes::default()))?;
                crossterm::execute!(out, terminal::ScrollUp(title_lines))?;
            }
        }

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

        // Check to leave the space to render the data.
        let used_space = termutil::num_lines(self.title.as_ref().unwrap_or(&Graphemes::default()))?;
        if terminal::size()?.1 <= used_space {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Terminal does not leave the space to render.",
            ));
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
        termutil::move_left(out, next.width_from_position() as u16)?;
        Ok(())
    }
}

impl State {
    pub fn buffer_limit(&self) -> Result<Option<usize>> {
        if let Some(lines) = self.num_lines {
            if lines > 0 {
                return Ok(Some(
                    terminal::size()?.0 as usize * lines as usize - self.label.width(),
                ));
            }
        }
        Ok(None)
    }
}
