use std::io;

use crate::{
    buffer::Buffer,
    grapheme::Grapheme,
    readline::{Mode, State},
    termutil, EventHandleFn,
};

/// Move the position of buffer and cursor backward.
pub fn move_left() -> Box<EventHandleFn<State>> {
    Box::new(|_, _, out: &mut io::Stdout, state: &mut State| {
        let width = state.0.editor.width_in_pos() as u16;
        if state.0.editor.prev() {
            termutil::move_left(out, width)?;
        }
        Ok(None)
    })
}

/// Move the position of buffer and cursor forward.
pub fn move_right() -> Box<EventHandleFn<State>> {
    Box::new(|_, _, out: &mut io::Stdout, state: &mut State| {
        if state.0.editor.next() {
            termutil::move_right(out, state.0.editor.width_in_pos() as u16)?;
        }
        Ok(None)
    })
}

/// Move the position of buffer and cursor to head.
pub fn move_head() -> Box<EventHandleFn<State>> {
    Box::new(|_, _, out: &mut io::Stdout, state: &mut State| {
        termutil::move_left(out, state.0.editor.width_to_pos() as u16)?;
        state.0.editor.to_head();
        Ok(None)
    })
}

/// Move the position of buffer and cursor to tail.
pub fn move_tail() -> Box<EventHandleFn<State>> {
    Box::new(|_, _, out: &mut io::Stdout, state: &mut State| {
        termutil::move_right(out, state.0.editor.width_from_pos() as u16)?;
        state.0.editor.to_tail();
        Ok(None)
    })
}

/// Look up a previous input in history.
pub fn prev_history() -> Box<EventHandleFn<State>> {
    Box::new(|_, _, _: &mut io::Stdout, state: &mut State| {
        if let Some(hstr) = &state.1.hstr {
            if hstr.prev() {
                state.0.editor.replace(&hstr.get());
            }
        }
        Ok(None)
    })
}

/// Look up a next input in history.
pub fn next_history() -> Box<EventHandleFn<State>> {
    Box::new(|_, _, _: &mut io::Stdout, state: &mut State| {
        if let Some(hstr) = &state.1.hstr {
            if hstr.next() {
                state.0.editor.replace(&hstr.get());
            }
        }
        Ok(None)
    })
}

/// Erase a char at the current position.
pub fn erase_char() -> Box<EventHandleFn<State>> {
    Box::new(|_, _, _: &mut io::Stdout, state: &mut State| {
        if state.0.editor.pos() > 0 {
            state.0.editor.erase();
        }
        Ok(None)
    })
}

/// Erase all chars at the current line.
pub fn erase_all() -> Box<EventHandleFn<State>> {
    Box::new(|_, _, _: &mut io::Stdout, state: &mut State| {
        state.0.editor = Box::new(Buffer::default());
        Ok(None)
    })
}

/// Search the item by [Suggest](../struct.Suggest.html).
pub fn complete() -> Box<EventHandleFn<State>> {
    Box::new(|_, _, _: &mut io::Stdout, state: &mut State| {
        if let Some(suggest) = &state.1.suggest {
            if state.0.editor.data.len() >= state.1.min_len_to_search {
                if let Some(res) = suggest.search(&state.0.editor.data) {
                    state.0.editor.replace(&res);
                }
            }
        }
        Ok(None)
    })
}

/// Insert or overwrite a char at the current position.
pub fn input_char() -> Box<EventHandleFn<State>> {
    Box::new(
        |_, input: Option<char>, _: &mut io::Stdout, state: &mut State| {
            if let Some(limit) = state.buffer_limit()? {
                if limit <= state.0.editor.data.width() {
                    return Ok(None);
                }
            }
            if let Some(input) = input {
                match state.1.edit_mode {
                    Mode::Insert => state.0.editor.insert(Grapheme::from(input)),
                    Mode::Overwrite => state.0.editor.overwrite(Grapheme::from(input)),
                }
            }
            Ok(None)
        },
    )
}
