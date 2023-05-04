use std::io;

use crate::{
    crossterm::{event::Event, terminal},
    text, Controller, Result,
};

pub struct Store {
    pub state: text::State,
    pub renderer: text::Renderer,
}

impl Controller for Store {
    fn can_render(&self) -> Result<()> {
        if terminal::size()?.1 < self.state.text_lines()? {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Terminal does not leave the space to render.",
            ));
        }
        Ok(())
    }

    fn render_static(&self, out: &mut io::Stdout) -> Result<()> {
        self.renderer.render(out, &self.state)
    }

    fn handle_event(&mut self, _: &Event, _: &mut io::Stdout) -> Result<Option<String>> {
        Ok(None)
    }

    fn render(&mut self, _: &mut io::Stdout) -> Result<()> {
        Ok(())
    }

    fn finalize(&mut self, _: &mut io::Stdout) -> Result<()> {
        Ok(())
    }
}
