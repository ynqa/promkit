use std::io;

use crate::{tree::State, EventHandleFn};

/// Move up from the current selected position in the candidates.
pub fn move_up() -> Box<EventHandleFn<State>> {
    Box::new(|_, _, _: &mut io::Stdout, state: &mut State| {
        if state.0.editor.prev() {
            state.move_up()?;
        }
        Ok(false)
    })
}

/// Move down from the current selected position in the candidates.
pub fn move_down() -> Box<EventHandleFn<State>> {
    Box::new(|_, _, _: &mut io::Stdout, state: &mut State| {
        if state.0.editor.next() {
            state.move_down()?;
        }
        Ok(false)
    })
}

pub fn toggle() -> Box<EventHandleFn<State>> {
    Box::new(|_, _, _: &mut io::Stdout, state: &mut State| {
        state.0.editor.toggle();
        Ok(false)
    })
}
