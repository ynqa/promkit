use std::io;

use crate::{select::State, EventHandleFn};

/// Move up from the current selected position in the candidates.
pub fn move_up() -> Box<EventHandleFn<State>> {
    Box::new(|_, _, _: &mut io::Stdout, state: &mut State| {
        // cyclical movement
        if !state.editor.prev() {
            state.editor.to_tail();
            state.cursor.to_tail(state.screen_size(&state.editor)?);
        } else {
            state.cursor.prev();
        }
        Ok(None)
    })
}

/// Move down from the current selected position in the candidates.
pub fn move_down() -> Box<EventHandleFn<State>> {
    Box::new(|_, _, _: &mut io::Stdout, state: &mut State| {
        // cyclical movement
        if !state.editor.next() {
            state.editor.to_head();
            state.cursor.to_head();
        } else {
            state.cursor.next(state.screen_size(&state.editor)?)?;
        }
        Ok(None)
    })
}

/// Move the selected position to head.
pub fn move_head() -> Box<EventHandleFn<State>> {
    Box::new(|_, _, _: &mut io::Stdout, state: &mut State| {
        state.editor.to_head();
        state.cursor.to_head();
        Ok(None)
    })
}

/// Move the selected position to tail.
pub fn move_tail() -> Box<EventHandleFn<State>> {
    Box::new(|_, _, _: &mut io::Stdout, state: &mut State| {
        state.editor.to_tail();
        state.cursor.to_tail(state.screen_size(&state.editor)?);
        Ok(None)
    })
}
