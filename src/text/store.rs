use std::io;

use crate::{crossterm::event::Event, text, Controller, Result};

pub struct Store {
    pub state: text::State,
    pub renderer: text::Renderer,
}

impl Controller for Store {
    fn used_rows(&self, _: u16) -> Result<u16> {
        self.state.text_lines()
    }

    fn render_static(&self, out: &mut io::Stdout, _: u16) -> Result<()> {
        self.renderer.render(out, &self.state)
    }

    fn handle_event(&mut self, _: &Event, _: &mut io::Stdout, _: u16) -> Result<Option<String>> {
        Ok(None)
    }

    fn render(&mut self, _: &mut io::Stdout, _: u16) -> Result<()> {
        Ok(())
    }

    fn finalize(&mut self, _: &mut io::Stdout) -> Result<()> {
        Ok(())
    }
}
