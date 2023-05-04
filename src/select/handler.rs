use std::io;

use crate::{crossterm::event::Event, keybind::KeyBind, select::State, Result, UpstreamContext};

pub struct EventHandler {
    pub keybind: KeyBind<State>,
}

impl EventHandler {
    pub fn handle_event(
        &self,
        ev: &Event,
        out: &mut io::Stdout,
        context: &UpstreamContext,
        select: &mut State,
    ) -> Result<Option<String>> {
        self.keybind.handle(ev, out, context, select)
    }
}
