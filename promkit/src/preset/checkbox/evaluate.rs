use crate::{
    core::{
        crossterm::event::{
            Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers, MouseButton,
            MouseEvent, MouseEventKind,
        },
        ScreenPosition,
    },
    preset::checkbox::{Checkbox, Index},
    widgets::checkbox::CheckboxHit,
    Signal,
};

/// Default key bindings for the checkbox interface.
///
/// | Key                    | Action
/// | :--------------------- | :-------------------------------------------
/// | <kbd>Enter</kbd>       | Exit the interface
/// | <kbd>Ctrl + C</kbd>    | Interrupt the current operation
/// | <kbd>↑</kbd>           | Move the selection up
/// | <kbd>↓</kbd>           | Move the selection down
/// | <kbd>Space</kbd>       | Toggle the checkbox state for the current item
/// | Left click             | Move to and toggle the clicked item
pub async fn default(event: &Event, ctx: &mut Checkbox) -> anyhow::Result<Signal> {
    match event {
        // Quit
        Event::Key(KeyEvent {
            code: KeyCode::Enter,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => return Ok(Signal::Quit),
        Event::Key(KeyEvent {
            code: KeyCode::Char('c'),
            modifiers: KeyModifiers::CONTROL,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => return Err(anyhow::anyhow!("ctrl+c")),

        // Move cursor.
        Event::Key(KeyEvent {
            code: KeyCode::Up,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => {
            ctx.checkbox.checkbox.backward();
        }
        Event::Mouse(MouseEvent {
            kind: MouseEventKind::ScrollUp,
            column: _,
            row: _,
            modifiers: KeyModifiers::NONE,
        }) => {
            ctx.checkbox.checkbox.backward();
        }

        Event::Key(KeyEvent {
            code: KeyCode::Down,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => {
            ctx.checkbox.checkbox.forward();
        }
        Event::Mouse(MouseEvent {
            kind: MouseEventKind::ScrollDown,
            column: _,
            row: _,
            modifiers: KeyModifiers::NONE,
        }) => {
            ctx.checkbox.checkbox.forward();
        }

        Event::Key(KeyEvent {
            code: KeyCode::Char(' '),
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => ctx.checkbox.checkbox.toggle(),

        Event::Mouse(MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column,
            row,
            modifiers: KeyModifiers::NONE,
        }) => {
            let renderer = ctx.renderer.clone();
            if let Some(position) = renderer
                .as_ref()
                .and_then(|renderer| {
                    renderer.hit_test(ScreenPosition {
                        row: *row,
                        column: *column,
                    })
                })
                .filter(|position| position.index == Index::Checkbox)
            {
                if let Some(CheckboxHit::Toggle { index }) =
                    ctx.checkbox.hit_at(position.content_position())
                {
                    ctx.checkbox.checkbox.toggle_at(index);
                }
            }
        }

        _ => (),
    }
    Ok(Signal::Continue)
}
