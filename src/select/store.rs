use std::io;

use crate::{
    crossterm::{cursor, event::Event, terminal},
    readline,
    select::{EventHandler, Renderer, State},
    termutil, text, Controller, Result,
};

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
