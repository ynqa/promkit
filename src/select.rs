use std::cmp::Ordering;
use std::fmt;
use std::io;

use crate::{
    crossterm::{cursor, event::Event, style, terminal},
    grapheme::Graphemes,
    internal::selector::Selector,
    keybind::KeyBind,
    readline, termutil, text, Controller, Result,
};

pub mod action;
mod builder;
mod keybind;

pub use self::builder::Builder;

/// Select specific state.
pub struct State {
    pub editor: Selector,
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
        let unused_rows = terminal::size()?.1 - cursor::position()?.1;
        Ok(*vec![
            unused_rows,
            self.window.unwrap_or(unused_rows),
            self.editor.data.len() as u16,
        ]
        .iter()
        .min()
        .unwrap_or(&unused_rows))
    }
}

pub struct EventHandler {
    pub keybind: KeyBind<State>,
}

impl EventHandler {
    pub fn handle_event(
        &self,
        ev: &Event,
        out: &mut io::Stdout,
        state: &mut State,
    ) -> Result<Option<String>> {
        self.keybind.handle(ev, out, state)
    }
}

pub struct Renderer {}

impl Renderer {
    pub fn render<W: io::Write>(&self, out: &mut W, state: &State) -> Result<()> {
        if !state.editor.data.is_empty() {
            let selector_position = state.editor.position();
            let from = selector_position - state.screen_position as usize;
            let to = selector_position + (state.selector_lines()? - state.screen_position) as usize;

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
                if termutil::compare_cursor_position(termutil::Boundary::Bottom)? == Ordering::Less
                {
                    termutil::move_down(out, 1)?;
                }
            }
        }
        Ok(())
    }
}

pub struct Store {
    pub state: State,
    pub handler: EventHandler,
    pub renderer: Renderer,
    pub title_store: Option<text::Store>,
    pub query_store: Option<readline::Store>,
}

impl Controller for Store {
    fn can_render(&self) -> Result<()> {
        let title_lines = self
            .title_store
            .as_ref()
            .map_or(Ok(0), |s| s.state.text_lines())?;
        let query_lines = self
            .query_store
            .as_ref()
            .map_or(Ok(0), |s| s.state.buffer_lines())?;
        if terminal::size()?.1 < title_lines + query_lines {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Terminal does not leave the space to render.",
            ));
        }
        Ok(())
    }

    fn handle_event(&mut self, ev: &Event, out: &mut io::Stdout) -> Result<Option<String>> {
        if let Event::Resize(_, _) = ev {
            self.state.editor.to_head();
            self.state.screen_position = 0;
            termutil::clear(out)?;
            self.render_static(out)?;
            Ok(None)
        } else {
            if let Some(ref mut query_store) = self.query_store {
                query_store.handle_event(ev, out)?;
            }
            self.handler.handle_event(ev, out, &mut self.state)
        }
    }

    fn render_static(&self, out: &mut io::Stdout) -> Result<()> {
        if let Some(ref title_store) = self.title_store {
            title_store.renderer.render(out, &title_store.state)?;
            termutil::move_down(out, 1)?;
        }

        if let Some(ref query_store) = self.query_store {
            query_store.render_static(out)?;
        }
        Ok(())
    }

    fn finalize(&mut self, out: &mut io::Stdout) -> Result<()> {
        if self.query_store.is_some() {
            termutil::move_down(out, 1)?;
        }
        termutil::show_cursor(out)?;
        crossterm::execute!(out, terminal::Clear(terminal::ClearType::FromCursorDown))
    }

    fn render(&mut self, out: &mut io::Stdout) -> Result<()> {
        if let Some(ref mut query_store) = self.query_store {
            query_store.render(out)?;
        }
        crossterm::execute!(out, cursor::SavePosition)?;
        termutil::hide_cursor(out)?;
        if self.query_store.is_some() {
            termutil::move_down(out, 1)?;
        }
        self.renderer.render(out, &self.state)?;
        if self.query_store.is_some() {
            termutil::show_cursor(out)?;
        }
        // Return to the initial position before rendering.
        crossterm::execute!(out, cursor::RestorePosition)
    }
}
