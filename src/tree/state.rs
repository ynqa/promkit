use std::cmp::Ordering;
use std::io;

use crate::{
    crossterm::{cursor, style, terminal},
    state,
    termutil::{self, Boundary},
    treeview::TreeView,
    Output, Result,
};

pub type State = state::State<TreeView, With>;

/// Tree specific state.
pub struct With {
    pub folded_label: char,
    pub unfolded_label: char,
    pub selected_color: style::Color,
    pub selected_cursor_position: u16,
}

impl Output for State {
    type Output = String;

    fn output(&self) -> Self::Output {
        String::new()
    }
}

impl<W: io::Write> state::Render<W> for State {
    fn pre_render(&self, _out: &mut W) -> Result<()> {
        Ok(())
    }

    fn render(&mut self, out: &mut W) -> Result<()> {
        let next = self.0.next.clone();
        let flatten_tree = next.flatten();
        let flatten_tree_position = next.position();
        let from = flatten_tree_position - self.1.selected_cursor_position as usize;
        let to = flatten_tree_position;

        crossterm::execute!(out, cursor::SavePosition)?;
        for i in from..to {
            crossterm::execute!(out, terminal::Clear(terminal::ClearType::CurrentLine))?;
            if i == flatten_tree_position {
                crossterm::execute!(out, style::SetForegroundColor(self.1.selected_color))?;
            }
            crossterm::execute!(out, style::Print(&flatten_tree[i]))?;
            if i == flatten_tree_position {
                crossterm::execute!(out, style::SetForegroundColor(style::Color::Reset))?;
            }
            if termutil::compare_cursor_position(Boundary::Bottom)? == Ordering::Less {
                crossterm::execute!(out, cursor::MoveToNextLine(1))?;
            }
        }

        // Return to the initial position.
        crossterm::execute!(out, cursor::RestorePosition)?;
        Ok(())
    }
}

impl State {
    pub fn move_up(&mut self) -> Result<()> {
        if self.1.selected_cursor_position == 0 {
            self.1.selected_cursor_position = 0;
        } else {
            self.1.selected_cursor_position -= 1;
        }
        Ok(())
    }

    pub fn move_down(&mut self) -> Result<()> {
        if self.1.selected_cursor_position >= terminal::size()?.1 {
            self.1.selected_cursor_position = terminal::size()?.1;
        } else {
            self.1.selected_cursor_position += 1;
        }
        Ok(())
    }
}
