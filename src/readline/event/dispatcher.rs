use std::io;

use crate::{
    crossterm::event::Event,
    internal::buffer::Buffer,
    readline::{self, event::handler::EventHandler},
    register::Register,
    termutil, text, Result, Runnable,
};

pub struct Dispatcher {
    /// Title displayed on the initial line.
    pub title: Option<text::State>,
    pub readline: readline::State,
    pub handler: EventHandler,
}

impl Runnable for Dispatcher {
    fn used_lines(&self) -> Result<u16> {
        let title_lines = self.title.as_ref().map_or(Ok(0), |t| t.text_lines())?;
        let buffer_lines = self.readline.buffer_lines()?;
        Ok(title_lines + buffer_lines)
    }

    fn handle_event(&mut self, ev: &Event, out: &mut io::Stdout) -> Result<Option<String>> {
        self.handler
            .handle_event(ev, out, &mut self.title, &mut self.readline)
    }

    fn initialize(&mut self, out: &mut io::Stdout) -> Result<Option<String>> {
        // Render the title.
        if let Some(ref mut title) = self.title {
            title.render(out)?;
        }
        self.readline.render_static(out)?;
        Ok(None)
    }

    fn finalize(&mut self, out: &mut io::Stdout) -> Result<Option<String>> {
        termutil::move_down(out, 1)?;
        if let Some(hstr) = &mut self.readline.hstr {
            hstr.register(self.readline.editor.data.clone());
        }
        self.readline.editor = Buffer::default();
        self.readline.prev = Buffer::default();
        self.readline.next = Buffer::default();
        Ok(None)
    }

    fn pre_run(&mut self, out: &mut io::Stdout) -> Result<Option<String>> {
        // Sync number of title lines with select state.
        self.readline.title_lines = self.title.as_ref().map_or(Ok(0), |t| t.text_lines())?;
        self.readline.render(out)?;
        self.readline.prev = self.readline.editor.clone();
        Ok(None)
    }

    fn post_run(&mut self, _: &mut io::Stdout) -> Result<Option<String>> {
        self.readline.next = self.readline.editor.clone();
        Ok(None)
    }
}
