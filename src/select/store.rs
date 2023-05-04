use std::io;

use crate::{
    crossterm::{cursor, event::Event, terminal},
    select::{EventHandler, Renderer, State},
    termutil, text, Controller, Result,
};

pub struct Store {
    pub state: State,
    pub handler: EventHandler,
    pub renderer: Renderer,
    pub title_store: Option<text::Store>,
}

impl Controller for Store {
    fn can_render(&self) -> Result<()> {
        let title_lines = self
            .title_store
            .as_ref()
            .map_or(Ok(0), |s| s.state.text_lines())?;
        if terminal::size()?.1 < title_lines {
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
            self.handler.handle_event(ev, out, &mut self.state)
        }
    }

    fn render_static(&self, out: &mut io::Stdout) -> Result<()> {
        // Render title
        if let Some(ref title_store) = self.title_store {
            title_store.render_static(out)?;
            termutil::move_down(out, 1)?;
        }
        termutil::hide_cursor(out)
    }

    fn finalize(&mut self, out: &mut io::Stdout) -> Result<()> {
        termutil::show_cursor(out)
    }

    fn render(&mut self, out: &mut io::Stdout) -> Result<()> {
        crossterm::execute!(out, cursor::SavePosition)?;
        self.renderer.render(out, &self.state)?;
        // Return to the initial position before rendering.
        crossterm::execute!(out, cursor::RestorePosition)
    }
}
