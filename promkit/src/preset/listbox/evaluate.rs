use crate::{
    core::crossterm::event::{
        Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers, MouseEvent,
        MouseEventKind,
    },
    preset::listbox::Listbox,
    Signal,
};

/// Default key bindings for the listbox.
///
/// | Key                    | Action
/// | :--------------------- | :-------------------------------------------
/// | <kbd>Enter</kbd>       | Exit the listbox
/// | <kbd>Ctrl + C</kbd>    | Interrupt the current operation
/// | <kbd>↑</kbd>           | Move the selection up
/// | <kbd>↓</kbd>           | Move the selection down
pub async fn default(event: &Event, ctx: &mut Listbox) -> anyhow::Result<Signal> {
    match event {
        // Resize the listbox.
        Event::Resize(width, height) => {
            ctx.render(*width, *height).await?;
        }

        // Exit the listbox.
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
        })
        | Event::Mouse(MouseEvent {
            kind: MouseEventKind::ScrollUp,
            column: _,
            row: _,
            modifiers: KeyModifiers::NONE,
        }) => {
            ctx.listbox.listbox.backward();
        }

        Event::Key(KeyEvent {
            code: KeyCode::Down,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        })
        | Event::Mouse(MouseEvent {
            kind: MouseEventKind::ScrollDown,
            column: _,
            row: _,
            modifiers: KeyModifiers::NONE,
        }) => {
            ctx.listbox.listbox.forward();
        }

        _ => (),
    }
    Ok(Signal::Continue)
}
