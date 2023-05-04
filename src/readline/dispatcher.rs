use std::io;

use crate::{
    crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers},
    grapheme::Grapheme,
    internal::buffer::Buffer,
    keybind::KeyBind,
    readline::{Mode, State},
    register::Register,
    termutil, text, Result, Runnable,
};

pub struct Dispatcher {
    pub keybind: KeyBind<State>,
    /// Title displayed on the initial line.
    pub title: Option<text::State>,
    pub readline: State,
}

impl Dispatcher {
    fn handle_resize(&mut self, _: (u16, u16), out: &mut io::Stdout) -> Result<Option<String>> {
        termutil::clear(out)?;
        // Render the title.
        if let Some(ref mut title) = self.title {
            title.render(out)?;
        }
        self.readline.render_static(out)?;
        // Overwrite the prev as default.
        self.readline.prev = Buffer::default();
        Ok(None)
    }

    fn handle_input(&mut self, ch: char, _: &mut io::Stdout) -> Result<Option<String>> {
        if self.readline.buffer_limit()? <= self.readline.editor.data.width() as u16 {
            return Ok(None);
        }
        match self.readline.edit_mode {
            Mode::Insert => self.readline.editor.insert(Grapheme::from(ch)),
            Mode::Overwrite => self.readline.editor.overwrite(Grapheme::from(ch)),
        }
        Ok(None)
    }
}

impl Runnable for Dispatcher {
    fn used_lines(&self) -> Result<u16> {
        let title_lines = self.title.as_ref().map_or(Ok(0), |t| t.text_lines())?;
        let buffer_lines = self.readline.buffer_lines()?;
        Ok(title_lines + buffer_lines)
    }

    fn handle_event(&mut self, ev: &Event, out: &mut io::Stdout) -> Result<Option<String>> {
        if let Event::Resize(x, y) = ev {
            if let Some(ret) = self.handle_resize((*x, *y), out)? {
                return Ok(Some(ret));
            }
        }

        if let Some(ret) = self.keybind.handle(ev, out, &mut self.readline)? {
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
