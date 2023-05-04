use std::io;

use crate::{
    crossterm::event::Event,
    grid::UpstreamContext,
    internal::buffer::Buffer,
    readline::{handler::EventHandler, renderer::Renderer, State},
    register::Register,
    termutil, Controller, Result,
};

pub struct Store {
    pub readline: State,
    pub handler: EventHandler,
    pub renderer: Renderer,
}

impl Controller for Store {
    fn used_rows(&self, context: &UpstreamContext) -> Result<u16> {
        self.readline.buffer_lines(context)
    }

    fn run_on_resize(&mut self) -> Result<()> {
        // Overwrite the prev as default.
        self.readline.prev = Buffer::default();
        Ok(())
    }

    fn handle_event(
        &mut self,
        ev: &Event,
        out: &mut io::Stdout,
        context: &UpstreamContext,
    ) -> Result<Option<String>> {
        self.handler
            .handle_event(ev, out, context, &mut self.readline)
    }

    fn render_static(&self, out: &mut io::Stdout) -> Result<()> {
        // Render label.
        self.renderer.render_static(out, &self.readline)
    }

    fn finalize(&mut self, out: &mut io::Stdout) -> Result<()> {
        termutil::move_down(out, 1)?;
        if let Some(hstr) = &mut self.readline.hstr {
            hstr.register(self.readline.editor.data.clone());
        }
        self.readline.editor = Buffer::default();
        self.readline.prev = Buffer::default();
        self.readline.next = Buffer::default();
        Ok(())
    }

    fn render(&mut self, out: &mut io::Stdout, _context: &UpstreamContext) -> Result<()> {
        self.readline.next = self.readline.editor.clone();
        self.renderer.render(out, &self.readline)?;
        self.readline.prev = self.readline.editor.clone();
        Ok(())
    }
}
