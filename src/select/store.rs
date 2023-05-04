use std::io;

use crate::{
    crossterm::{cursor, event::Event},
    select::{EventHandler, Renderer, State},
    termutil, Controller, Result, UpstreamContext,
};

pub struct Store {
    pub select: State,
    pub handler: EventHandler,
    pub renderer: Renderer,
}

impl Controller for Store {
    fn used_rows(&self, context: &UpstreamContext) -> Result<u16> {
        self.select.selector_lines(context.unused_rows)
    }

    fn run_on_resize(&mut self) -> Result<()> {
        self.select.editor.to_head();
        self.select.screen_position = 0;
        Ok(())
    }

    fn handle_event(
        &mut self,
        ev: &Event,
        out: &mut io::Stdout,
        context: &UpstreamContext,
    ) -> Result<Option<String>> {
        self.handler
            .handle_event(ev, out, context, &mut self.select)
    }

    fn render_static(&self, out: &mut io::Stdout) -> Result<()> {
        termutil::hide_cursor(out)
    }

    fn finalize(&mut self, out: &mut io::Stdout) -> Result<()> {
        termutil::show_cursor(out)
    }

    fn render(&mut self, out: &mut io::Stdout, context: &UpstreamContext) -> Result<()> {
        crossterm::execute!(out, cursor::SavePosition)?;
        self.renderer
            .render(out, context.unused_rows, &self.select)?;
        // Return to the initial position before rendering.
        crossterm::execute!(out, cursor::RestorePosition)
    }
}
