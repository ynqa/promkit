use std::cmp::Ordering;

use std::io;

use crate::{
    crossterm::{style, terminal},
    grapheme::Graphemes,
    select::State,
    termutil::{self, Boundary},
    Result,
};

pub struct Renderer {}

impl Renderer {
    pub fn render<W: io::Write>(
        &mut self,
        out: &mut W,
        unused_rows: u16,
        state: &State,
    ) -> Result<()> {
        let next = state.next.clone();
        if !next.data.is_empty() {
            let selector_position = next.position();
            let from = selector_position - state.screen_position as usize;
            let to = selector_position
                + (state.selector_lines(unused_rows)? - state.screen_position) as usize;

            for i in from..to {
                crossterm::execute!(out, terminal::Clear(terminal::ClearType::CurrentLine))?;
                if i == selector_position {
                    crossterm::execute!(out, style::SetForegroundColor(state.label_color))?;
                }
                crossterm::execute!(
                    out,
                    style::Print(termutil::append_prefix_and_trim_suffix(
                        &if i == selector_position {
                            state.label.to_owned()
                        } else {
                            Graphemes::from(" ".repeat(state.label.width()))
                        },
                        &next.get_with_index(i),
                        &state.suffix_after_trim,
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
