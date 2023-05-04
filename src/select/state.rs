use std::cmp::Ordering;
use std::fmt;
use std::io;

use crate::{
    crossterm::{style, terminal},
    grapheme::Graphemes,
    internal::selector::Selector,
    termutil::{self, Boundary},
    Result,
};

/// Select specific state.
pub struct State {
    pub editor: Selector,
    pub prev: Selector,
    pub next: Selector,
    pub title_lines: u16,
    /// A symbol to emphasize the selected item (e.g. ">").
    pub label: Graphemes,
    pub label_color: style::Color,
    pub screen_position: u16,
    pub window: Option<u16>,
    pub suffix_after_trim: Graphemes,
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.editor.get())
    }
}

impl State {
    pub fn selector_lines(&self) -> Result<u16> {
        let left_space = terminal::size()?.1 - self.title_lines;
        Ok(*vec![
            left_space,
            self.window.unwrap_or(left_space),
            self.editor.data.len() as u16,
        ]
        .iter()
        .min()
        .unwrap_or(&left_space))
    }

    pub fn render<W: io::Write>(&mut self, out: &mut W) -> Result<()> {
        let next = self.next.clone();
        if !next.data.is_empty() {
            let selector_position = next.position();
            let from = selector_position - self.screen_position as usize;
            let to = selector_position + (self.selector_lines()? - self.screen_position) as usize;

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
        }
        Ok(())
    }
}
