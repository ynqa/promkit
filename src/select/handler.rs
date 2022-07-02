use std::io;

use crate::{select::State, EventHandleFn};

/// Move up from the current selected position in the candidates.
pub fn move_up() -> Box<EventHandleFn<State>> {
    Box::new(|_, _, _: &mut io::Stdout, state: &mut State| {
        let prev = state.0.editor.clone();
        state.0.editor.prev();
        state.move_up()?;
        state.0.input_stream.push((prev, state.0.editor.clone()));
        Ok(None)
    })
}

/// Move down from the current selected position in the candidates.
pub fn move_down() -> Box<EventHandleFn<State>> {
    Box::new(|_, _, _: &mut io::Stdout, state: &mut State| {
        let prev = state.0.editor.clone();
        state.0.editor.next();
        state.move_down()?;
        state.0.input_stream.push((prev, state.0.editor.clone()));
        Ok(None)
    })
}

/// Move the selected position to head.
pub fn move_head() -> Box<EventHandleFn<State>> {
    Box::new(|_, _, _: &mut io::Stdout, state: &mut State| {
        let prev = state.0.editor.clone();
        state.0.editor.to_head();
        state.move_head()?;
        state.0.input_stream.push((prev, state.0.editor.clone()));
        Ok(None)
    })
}

/// Move the selected position to tail.
pub fn move_tail() -> Box<EventHandleFn<State>> {
    Box::new(|_, _, _: &mut io::Stdout, state: &mut State| {
        let prev = state.0.editor.clone();
        state.0.editor.to_tail();
        state.move_tail()?;
        state.0.input_stream.push((prev, state.0.editor.clone()));
        Ok(None)
    })
}
