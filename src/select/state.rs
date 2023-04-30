use std::cmp::Ordering;
use std::fmt;
use std::io;

use crate::{
    crossterm::{cursor, style, terminal},
    grapheme::Graphemes,
    internal::selector::Selector,
    select::cursor::Cursor,
    termutil::{self, Boundary},
    text, Result,
};

/// Select specific state.
pub struct State {
    pub editor: Selector,
    pub prev: Selector,
    pub next: Selector,
    /// Title displayed on the initial line.
    pub title: Option<text::State>,
    /// A symbol to emphasize the selected item (e.g. ">").
    pub label: Graphemes,
    pub label_color: style::Color,
    pub init_move_down_lines: u16,
    pub cursor: Cursor,
    pub window: Option<u16>,
    pub suffix_after_trim: Graphemes,
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.editor.get())
    }
}

impl State {
    pub fn used_lines(&self) -> Result<u16> {
        let title_lines = self.title.as_ref().map_or(Ok(0), |t| t.num_lines())?;
        let left_space = terminal::size()?.1 - (self.init_move_down_lines + title_lines);
        let selector_lines = *vec![
            left_space,
            self.window.unwrap_or(left_space),
            self.next.data.len() as u16,
        ]
        .iter()
        .min()
        .unwrap_or(&left_space);
        Ok(self.init_move_down_lines + title_lines + selector_lines)
    }

    pub fn screen_size(&self, selector: &Selector) -> Result<u16> {
        let title_lines = self.title.as_ref().map_or(Ok(0), |t| t.num_lines())?;
        let left_space = terminal::size()?.1 - (self.init_move_down_lines + title_lines);
        Ok(*vec![
            left_space,
            self.window.unwrap_or(left_space),
            selector.data.len() as u16,
        ]
        .iter()
        .min()
        .unwrap_or(&left_space))
    }

    pub fn render_static<W: io::Write>(&mut self, out: &mut W) -> Result<()> {
        // Move down with init_move_down_lines.
        if 0 < self.init_move_down_lines {
            termutil::move_down(out, self.init_move_down_lines)?;
        }

        // Render the title.
        if let Some(ref mut title) = self.title {
            title.render(out)?;
        }

        // Return to the initial position.
        crossterm::execute!(out, cursor::MoveTo(0, 0))
    }

    pub fn render<W: io::Write>(&mut self, out: &mut W) -> Result<()> {
        let next = self.next.clone();
        if !next.data.is_empty() {
            crossterm::execute!(out, cursor::SavePosition)?;

            // Move down the lines already written.
            let title_lines = self.title.as_ref().map_or(Ok(0), |t| t.num_lines())?;
            let move_down_lines = self.init_move_down_lines + title_lines;
            if 0 < move_down_lines {
                termutil::move_down(out, move_down_lines)?;
            }

            let selector_position = next.position();
            let from = selector_position - self.cursor.position.get() as usize;
            let to = selector_position
                + (self.screen_size(&next)? - self.cursor.position.get()) as usize;

            for i in from..to {
                crossterm::execute!(out, terminal::Clear(terminal::ClearType::CurrentLine))?;
                if i == selector_position {
                    crossterm::execute!(out, style::SetForegroundColor(self.label_color))?;
                }
                crossterm::execute!(
                    out,
                    style::Print(termutil::append_prefix_and_trim_suffix(
                        &if i == selector_position {
                            self.label.to_owned()
                        } else {
                            Graphemes::from(" ".repeat(self.label.width()))
                        },
                        &next.get_with_index(i),
                        &self.suffix_after_trim,
                    )?)
                )?;
                if i == selector_position {
                    crossterm::execute!(out, style::SetForegroundColor(style::Color::Reset))?;
                }
                if termutil::compare_cursor_position(Boundary::Bottom)? == Ordering::Less {
                    termutil::move_down(out, 1)?;
                }
            }

            // Return to the initial position.
            crossterm::execute!(out, cursor::RestorePosition)?;
        }
        Ok(())
    }
}
