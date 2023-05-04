use std::io;

use crate::{
    crossterm::event::Event,
    internal::buffer::Buffer,
    readline::{self, handler::EventHandler, render::Renderer},
    register::Register,
    termutil, Controller, Result,
};

pub struct Store {
    pub readline: readline::State,
    pub handler: EventHandler,
    pub renderer: Renderer,
}

impl Store {
    fn handle_resize(
        &mut self,
        _: (u16, u16),
        out: &mut io::Stdout,
        unused_rows: u16,
    ) -> Result<()> {
        termutil::clear(out)?;

        // Overwrite the prev as default.
        self.readline.prev = Buffer::default();

        // self.can_render()?;
        self.render_static(out, unused_rows)
    }
}

impl Controller for Store {
    fn used_rows(&self, unused_rows: u16) -> Result<u16> {
        self.readline.buffer_lines(unused_rows)
    }

    fn handle_event(
        &mut self,
        ev: &Event,
        out: &mut io::Stdout,
        unused_rows: u16,
    ) -> Result<Option<String>> {
        if let Event::Resize(x, y) = ev {
            self.handle_resize((*x, *y), out, unused_rows)?;
        }

        self.handler
            .handle_event(ev, out, unused_rows, &mut self.readline)
    }

    fn render_static(&self, out: &mut io::Stdout, _unused_rows: u16) -> Result<()> {
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

    fn render(&mut self, out: &mut io::Stdout, _unused_rows: u16) -> Result<()> {
        self.readline.next = self.readline.editor.clone();
        self.renderer.render(out, &self.readline)?;
        self.readline.prev = self.readline.editor.clone();
        Ok(())
    }
}
