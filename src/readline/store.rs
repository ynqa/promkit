use std::io;

use crate::{
    crossterm::{event::Event, terminal},
    internal::buffer::Buffer,
    readline::{handler::EventHandler, renderer::Renderer, State},
    register::Register,
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
            // Overwrite the prev as default.
            self.state.prev = Buffer::default();
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
        // Render label.
        self.renderer.render_static(out, &self.state)
    }

    fn finalize(&mut self, out: &mut io::Stdout) -> Result<()> {
        termutil::move_down(out, 1)?;
        if let Some(hstr) = &mut self.state.hstr {
            hstr.register(self.state.editor.data.clone());
        }
        self.state.editor = Buffer::default();
        self.state.prev = Buffer::default();
        self.state.next = Buffer::default();
        Ok(())
    }

    fn render(&mut self, out: &mut io::Stdout) -> Result<()> {
        self.state.next = self.state.editor.clone();
        self.renderer.render(out, &self.state)?;
        self.state.prev = self.state.editor.clone();
        Ok(())
    }
}
