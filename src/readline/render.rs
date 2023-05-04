use std::io;

use crate::{
    crossterm::{style, terminal},
    grapheme::{Grapheme, Graphemes},
    readline::State,
    termutil, Result,
};

pub struct Renderer {}

impl Renderer {
    pub fn render_static<W: io::Write>(&self, out: &mut W, state: &State) -> Result<()> {
        // Render the label.
        crossterm::execute!(
            out,
            style::SetForegroundColor(state.label_color),
            style::Print(state.label.to_owned()),
            style::SetForegroundColor(style::Color::Reset),
        )
    }

    pub fn render<W: io::Write>(&self, out: &mut W, state: &State) -> Result<()> {
        let (mut prev, mut next) = (state.prev.clone(), state.next.clone());
        if prev.data == next.data {
            return Ok(());
        }

        // Masking.
        prev.data = match &state.mask {
            None => prev.data.clone(),
            Some(mask) => prev
                .data
                .iter()
                .map(|_| mask.clone())
                .collect::<Graphemes>(),
        };
        next.data = match &state.mask {
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
