use std::io;

use crate::{
    crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers},
    grapheme::Grapheme,
    keybind::KeyBind,
    readline::{Mode, State},
    Result, UpstreamContext,
};

pub struct EventHandler {
    pub keybind: KeyBind<State>,
}

impl EventHandler {
    pub fn handle_event(
        &self,
        ev: &Event,
        out: &mut io::Stdout,
        context: &UpstreamContext,
        readline: &mut State,
    ) -> Result<Option<String>> {
        if let Some(ret) = self.keybind.handle(ev, out, context, readline)? {
            return Ok(Some(ret));
        }

        if let Event::Key(KeyEvent {
            code: KeyCode::Char(ch),
            modifiers: KeyModifiers::NONE,
            ..
        }) = ev
        {
            if readline.buffer_limit(context.unused_rows)? <= readline.editor.data.width() as u16 {
                return Ok(None);
            }
            match readline.edit_mode {
                Mode::Insert => readline.editor.insert(Grapheme::from(*ch)),
                Mode::Overwrite => readline.editor.overwrite(Grapheme::from(*ch)),
            }
        }

        Ok(None)
    }
}