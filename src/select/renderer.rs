use std::cmp::Ordering;

use std::io;

use crate::{
    crossterm::{style, terminal},
    grapheme::Graphemes,
    grid::UpstreamContext,
    select::State,
    termutil::{self, Boundary},
    Result,
};

pub struct Renderer {}

impl Renderer {
    pub fn render<W: io::Write>(
        &self,
        out: &mut W,
        context: &UpstreamContext,
        state: &State,
    ) -> Result<()> {
        if !state.editor.data.is_empty() {
            let selector_position = state.editor.position();
            let from = selector_position - state.screen_position as usize;
            let to = selector_position
                + (state.selector_lines(context)? - state.screen_position) as usize;

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
                        &state.editor.get_with_index(i),
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
