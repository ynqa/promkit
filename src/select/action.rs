use std::io;

use crate::{select::State, Action};

/// Move up from the current selected position in the candidates.
pub fn move_up() -> Box<Action<State>> {
    Box::new(|out: &mut io::Stdout, state: &mut State| {
        // cyclical movement
        if !state.editor.prev() {
            move_tail()(out, state)?;
        } else {
            state.cursor.prev();
        }
        Ok(None)
    })
}

/// Move down from the current selected position in the candidates.
pub fn move_down() -> Box<Action<State>> {
    Box::new(|out: &mut io::Stdout, state: &mut State| {
        // cyclical movement
        if !state.editor.next() {
            move_head()(out, state)?;
        } else {
            state.cursor.move_to(
                *vec![state.cursor.position.get() + 1, state.selector_lines()? - 1]
                    .iter()
                    .min()
                    .unwrap_or(&0),
            );
        }
        Ok(None)
    })
}

/// Move the selected position to head.
pub fn move_head() -> Box<Action<State>> {
    Box::new(|_: &mut io::Stdout, state: &mut State| {
        state.editor.to_head();
        state.cursor.to_head();
        Ok(None)
    })
}

/// Move the selected position to tail.
pub fn move_tail() -> Box<Action<State>> {
    Box::new(|_: &mut io::Stdout, state: &mut State| {
        state.editor.to_tail();
        state.cursor.move_to(state.selector_lines()? - 1);
        Ok(None)
    })
}
