use std::io;

use crate::{
    crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers},
    grapheme::Grapheme,
    internal::buffer::Buffer,
    readline::{Mode, State},
    register::Register,
    termutil, Dispatcher, Result, Runnable,
};

impl Dispatcher<State> {
    fn handle_resize(&mut self, _: (u16, u16), out: &mut io::Stdout) -> Result<Option<String>> {
        termutil::clear(out)?;
        self.state.render_static(out)?;
        // Overwrite the prev as default.
        self.state.prev = Buffer::default();
        Ok(None)
    }

    fn handle_input(&mut self, ch: char, _: &mut io::Stdout) -> Result<Option<String>> {
        if let Some(limit) = self.state.buffer_limit()? {
            if limit <= self.state.editor.data.width() {
                return Ok(None);
            }
        }
        match self.state.edit_mode {
            Mode::Insert => self.state.editor.insert(Grapheme::from(ch)),
            Mode::Overwrite => self.state.editor.overwrite(Grapheme::from(ch)),
        }
        Ok(None)
    }
}

impl Runnable for Dispatcher<State> {
    fn used_lines(&self) -> Result<u16> {
        self.state.used_lines()
    }

    fn handle_event(&mut self, ev: &Event, out: &mut io::Stdout) -> Result<Option<String>> {
        if let Event::Resize(x, y) = ev {
            if let Some(ret) = self.handle_resize((*x, *y), out)? {
                return Ok(Some(ret));
            }
        }

        if let Some(ret) = self.keybind.handle(ev, out, &mut self.state)? {
            return Ok(Some(ret));
        }

        if let Event::Key(KeyEvent {
            code: KeyCode::Char(ch),
            modifiers: KeyModifiers::NONE,
            ..
        }) = ev
        {
            if let Some(ret) = self.handle_input(*ch, out)? {
                return Ok(Some(ret));
            }
        }

        Ok(None)
    }

    fn initialize(&mut self, out: &mut io::Stdout) -> Result<Option<String>> {
        self.state.render_static(out)?;
        Ok(None)
    }

    fn finalize(&mut self, out: &mut io::Stdout) -> Result<Option<String>> {
        termutil::move_down(out, 1)?;
        if let Some(hstr) = &mut self.state.hstr {
            hstr.register(self.state.editor.data.clone());
        }
        self.state.editor = Buffer::default();
        self.state.prev = Buffer::default();
        self.state.next = Buffer::default();
        Ok(None)
    }

    fn pre_run(&mut self, out: &mut io::Stdout) -> Result<Option<String>> {
        self.state.render(out)?;
        self.state.prev = self.state.editor.clone();
        Ok(None)
    }

    fn post_run(&mut self, _: &mut io::Stdout) -> Result<Option<String>> {
        self.state.next = self.state.editor.clone();
        Ok(None)
    }
}
