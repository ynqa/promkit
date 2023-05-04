use std::io;

use crate::{
    crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers},
    grapheme::Grapheme,
    internal::buffer::Buffer,
    keybind::KeyBind,
    readline::{self, Mode},
    termutil, text, Result,
};

pub struct EventHandler {
    pub keybind: KeyBind<readline::State>,
}

impl EventHandler {
    fn handle_resize(
        &mut self,
        _: (u16, u16),
        out: &mut io::Stdout,
        title: &mut Option<text::State>,
        readline: &mut readline::State,
    ) -> Result<Option<String>> {
        termutil::clear(out)?;
        // Render the title.
        if let Some(ref mut title) = title {
            title.render(out)?;
        }
        readline.render_static(out)?;
        // Overwrite the prev as default.
        readline.prev = Buffer::default();
        Ok(None)
    }

    fn handle_input(&mut self, ch: char, readline: &mut readline::State) -> Result<Option<String>> {
        if readline.buffer_limit()? <= readline.editor.data.width() as u16 {
            return Ok(None);
        }
        match readline.edit_mode {
            Mode::Insert => readline.editor.insert(Grapheme::from(ch)),
            Mode::Overwrite => readline.editor.overwrite(Grapheme::from(ch)),
        }
        Ok(None)
    }

    pub fn handle_event(
        &mut self,
        ev: &Event,
        out: &mut io::Stdout,
        title: &mut Option<text::State>,
        readline: &mut readline::State,
    ) -> Result<Option<String>> {
        if let Event::Resize(x, y) = ev {
            if let Some(ret) = self.handle_resize((*x, *y), out, title, readline)? {
                return Ok(Some(ret));
            }
        }

        if let Some(ret) = self.keybind.handle(ev, out, readline)? {
            return Ok(Some(ret));
        }

        if let Event::Key(KeyEvent {
            code: KeyCode::Char(ch),
            modifiers: KeyModifiers::NONE,
            ..
        }) = ev
        {
            if let Some(ret) = self.handle_input(*ch, readline)? {
                return Ok(Some(ret));
            }
        }

        Ok(None)
    }
}
