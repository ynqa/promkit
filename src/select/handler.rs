use std::io;

use crate::{select::State, EventHandleFn};

/// Move up from the current selected position in the candidates.
pub fn move_up() -> Box<EventHandleFn<State>> {
    Box::new(|_, _, _: &mut io::Stdout, state: &mut State| {
        // cyclical movement
        if !state.editor.prev() {
            state.editor.to_tail();
            state.move_tail()?;
        } else {
            state.move_up()?;
        }
        Ok(false)
    })
}

/// Move down from the current selected position in the candidates.
pub fn move_down() -> Box<EventHandleFn<State>> {
    Box::new(|_, _, _: &mut io::Stdout, state: &mut State| {
        // cyclical movement
        if !state.editor.next() {
            state.editor.to_head();
            state.move_head()?;
        } else {
            state.move_down()?;
        }
        Ok(false)
    })
}

/// Move the selected position to head.
pub fn move_head() -> Box<EventHandleFn<State>> {
    Box::new(|_, _, _: &mut io::Stdout, state: &mut State| {
        state.editor.to_head();
        state.move_head()?;
        Ok(false)
    })
}

/// Move the selected position to tail.
pub fn move_tail() -> Box<EventHandleFn<State>> {
    Box::new(|_, _, _: &mut io::Stdout, state: &mut State| {
        state.editor.to_tail();
        state.move_tail()?;
        Ok(false)
    })
}
