use std::io;

use crate::{grid::UpstreamContext, select::State, Action};

/// Move up from the current selected position in the candidates.
pub fn move_up() -> Box<Action<State>> {
    Box::new(
        |out: &mut io::Stdout, context: &UpstreamContext, state: &mut State| {
            // cyclical movement
            if !state.editor.prev() {
                move_tail()(out, context, state)?;
            } else {
                (0 < state.screen_position).then(|| state.screen_position -= 1);
            }
            Ok(None)
        },
    )
}

/// Move down from the current selected position in the candidates.
pub fn move_down() -> Box<Action<State>> {
    Box::new(
        |out: &mut io::Stdout, context: &UpstreamContext, state: &mut State| {
            // cyclical movement
            if !state.editor.next() {
                move_head()(out, context, state)?;
            } else {
                state.screen_position = *vec![
                    state.screen_position + 1,
                    state.selector_lines(context)? - 1,
                ]
                .iter()
                .min()
                .unwrap_or(&0);
            }
            Ok(None)
        },
    )
}

/// Move the selected position to head.
pub fn move_head() -> Box<Action<State>> {
    Box::new(
        |_: &mut io::Stdout, _: &UpstreamContext, state: &mut State| {
            state.editor.to_head();
            state.screen_position = 0;
            Ok(None)
        },
    )
}

/// Move the selected position to tail.
pub fn move_tail() -> Box<Action<State>> {
    Box::new(
        |_: &mut io::Stdout, context: &UpstreamContext, state: &mut State| {
            state.editor.to_tail();
            state.screen_position = state.selector_lines(context)? - 1;
            Ok(None)
        },
    )
}
