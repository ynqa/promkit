use std::io;

use crate::{
    crossterm::event::Event,
    internal::buffer::Buffer,
    readline::{self, event::handler::EventHandler, render::Renderer},
    register::Register,
    termutil, text, Result, Runnable,
};

pub struct Dispatcher {
    pub title_dispatcher: Option<text::event::dispatcher::Dispatcher>,
    pub readline: readline::State,
    pub handler: EventHandler,
    pub renderer: Renderer,
}

impl Dispatcher {
    fn handle_resize(&mut self, _: (u16, u16), out: &mut io::Stdout) -> Result<()> {
        termutil::clear(out)?;

        // Overwrite the prev as default.
        self.readline.prev = Buffer::default();

        self.can_render()?;
        self.render_static(out)
    }
}

impl Runnable for Dispatcher {
    fn can_render(&self) -> Result<()> {
        let title_lines = self
            .title_dispatcher
            .as_ref()
            .map_or(Ok(0), |t| t.state.text_lines())?;
        let buffer_lines = self.readline.buffer_lines()?;

        if crossterm::terminal::size()?.1 < title_lines + buffer_lines {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Terminal does not leave the space to render.",
            ));
        }

        Ok(())
    }

    fn handle_event(&mut self, ev: &Event, out: &mut io::Stdout) -> Result<Option<String>> {
        if let Event::Resize(x, y) = ev {
            self.handle_resize((*x, *y), out)?;
        }

        self.handler.handle_event(ev, out, &mut self.readline)
    }

    fn render_static(&mut self, out: &mut io::Stdout) -> Result<()> {
        // Render title.
        if let Some(dispatcher) = &self.title_dispatcher {
            dispatcher.renderer.render(out, &dispatcher.state)?;
        }
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

    fn render(&mut self, out: &mut io::Stdout) -> Result<()> {
        // Sync number of title lines with select state.
        self.readline.title_lines = self
            .title_dispatcher
            .as_ref()
            .map_or(Ok(0), |t| t.state.text_lines())?;

        self.readline.next = self.readline.editor.clone();
        self.renderer.render(out, &self.readline)?;
        self.readline.prev = self.readline.editor.clone();
        Ok(())
    }
}
