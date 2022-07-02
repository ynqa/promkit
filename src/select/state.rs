use std::cmp::Ordering;
use std::io;

use crossterm::{cursor, style, terminal};

use crate::{
    grapheme::Graphemes,
    selectbox::SelectBox,
    state,
    termutil::{self, Boundary},
    Output, Result,
};

pub type State = state::State<SelectBox, With>;

/// Select specific state.
pub struct With {
    /// Title displayed on the initial line.
    pub title: Option<Graphemes>,
    pub title_color: Option<style::Color>,
    /// A symbol to emphasize the selected item (e.g. ">").
    pub label: Graphemes,
    pub label_color: style::Color,
    pub selected_cursor_pos: u16,
    pub init_move_down_lines: u16,
    pub window: Option<u16>,
    pub suffix_after_trim: Graphemes,
}

impl Output for State {
    type Output = String;

    fn output(&self) -> Self::Output {
        self.0.editor.get().to_string()
    }
}

impl<W: io::Write> state::Render<W> for State {
    fn pre_render(&self, out: &mut W) -> Result<()> {
        // Move down with init_move_down_lines.
        if 0 < self.1.init_move_down_lines {
            crossterm::execute!(out, cursor::MoveToNextLine(self.1.init_move_down_lines))?;
        }

        // Render the title.
        if let Some(title) = &self.1.title {
            if let Some(color) = self.1.title_color {
                crossterm::execute!(out, style::SetForegroundColor(color))?;
            }
            crossterm::execute!(out, style::Print(title), cursor::MoveToNextLine(1))?;
            if self.1.title_color.is_some() {
                crossterm::execute!(out, style::SetForegroundColor(style::Color::Reset))?;
            }
        }

        // Return to the initial position.
        crossterm::execute!(out, cursor::MoveTo(0, 0))
    }

    fn render(&mut self, out: &mut W) -> Result<()> {
        if let Some((_, next)) = self.0.input_stream.pop() {
            if !next.data.is_empty() {
                crossterm::execute!(out, cursor::SavePosition)?;

                // Check to leave the space to render the data.
                let title_lines =
                    termutil::num_lines(self.1.title.as_ref().unwrap_or(&Graphemes::default()))?;
                let used_space = self.1.init_move_down_lines + title_lines;
                if terminal::size()?.1 <= used_space {
                    return Err(io::Error::new(
                        io::ErrorKind::Other,
                        "Terminal does not leave the space to render.",
                    ));
                }

                // Move down the lines already written.
                let move_down_lines = self.1.init_move_down_lines
                    + termutil::num_lines(self.1.title.as_ref().unwrap_or(&Graphemes::default()))?;
                if 0 < move_down_lines {
                    crossterm::execute!(out, cursor::MoveToNextLine(move_down_lines))?;
                }

                let selectbox_pos = next.pos();
                let from = selectbox_pos - self.1.selected_cursor_pos as usize;
                let to = selectbox_pos
                    + (self.selectbox_lines(&next)? - self.1.selected_cursor_pos) as usize;

                for i in from..to {
                    crossterm::execute!(out, terminal::Clear(terminal::ClearType::CurrentLine))?;
                    if i == selectbox_pos {
                        crossterm::execute!(out, style::SetForegroundColor(self.1.label_color))?;
                    }
                    crossterm::execute!(
                        out,
                        style::Print(edit(
                            &next.get_with_index(i).to_string(),
                            &if i == selectbox_pos {
                                self.1.label.to_owned()
                            } else {
                                Graphemes::from(" ".repeat(self.1.label.width()))
                            },
                            &self.1.suffix_after_trim
                        )?)
                    )?;
                    if i == selectbox_pos {
                        crossterm::execute!(out, style::SetForegroundColor(style::Color::Reset))?;
                    }
                    if termutil::compare_cursor_position(Boundary::Bottom)? == Ordering::Less {
                        crossterm::execute!(out, cursor::MoveToNextLine(1))?;
                    }
                }

                // Return to the initial position.
                crossterm::execute!(out, cursor::RestorePosition)?;
            }
        }
        Ok(())
    }
}

impl State {
    pub fn move_up(&mut self) -> Result<()> {
        if self.1.selected_cursor_pos == 0 {
            self.1.selected_cursor_pos = 0;
        } else {
            self.1.selected_cursor_pos -= 1;
        }
        Ok(())
    }

    pub fn move_down(&mut self) -> Result<()> {
        if self.selectbox_lines(&self.0.editor)? > 0 {
            let limit = self.selectbox_lines(&self.0.editor)? - 1;
            if self.1.selected_cursor_pos >= limit {
                self.1.selected_cursor_pos = limit;
            } else {
                self.1.selected_cursor_pos += 1;
            }
        }
        Ok(())
    }

    pub fn move_head(&mut self) -> Result<()> {
        self.1.selected_cursor_pos = 0;
        Ok(())
    }

    pub fn move_tail(&mut self) -> Result<()> {
        self.1.selected_cursor_pos = self.selectbox_lines(&self.0.editor)? - 1;
        Ok(())
    }

    pub fn selectbox_lines(&self, selectbox: &SelectBox) -> Result<u16> {
        let left_space = terminal::size()?.1
            - (self.1.init_move_down_lines
                + termutil::num_lines(self.1.title.as_ref().unwrap_or(&Graphemes::default()))?);
        Ok(*vec![
            left_space,
            self.1.window.unwrap_or(left_space),
            selectbox.data.len() as u16,
        ]
        .iter()
        .min()
        .unwrap_or(&left_space))
    }
}

fn edit(line: &str, prefix: &Graphemes, suffix_after_trim: &Graphemes) -> Result<String> {
    let line = prefix.to_string() + line;
    let width_limit = terminal::size()?.0 as usize;
    if width_limit < suffix_after_trim.width() {
        return Ok(String::new());
    }

    let width_without_suffix = width_limit - suffix_after_trim.width();
    let res = if width_without_suffix < line.len() {
        line.chars().take(width_without_suffix).collect::<String>() + &suffix_after_trim.to_string()
    } else {
        line
    };
    Ok(res)
}
