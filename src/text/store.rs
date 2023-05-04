use std::io;

use crate::{crossterm::event::Event, text, Controller, Result, UpstreamContext};

pub struct Store {
    pub state: text::State,
    pub renderer: text::Renderer,
}

impl Controller for Store {
    fn used_rows(&self, _: &UpstreamContext) -> Result<u16> {
        self.state.text_lines()
    }

    fn render_static(&self, out: &mut io::Stdout) -> Result<()> {
        self.renderer.render(out, &self.state)
    }

    fn handle_event(
        &mut self,
        _: &Event,
        _: &mut io::Stdout,
        _: &UpstreamContext,
    ) -> Result<Option<String>> {
        Ok(None)
    }

    fn render(&mut self, _: &mut io::Stdout) -> Result<()> {
        Ok(())
    }

    fn finalize(&mut self, _: &mut io::Stdout) -> Result<()> {
        Ok(())
    }
}
