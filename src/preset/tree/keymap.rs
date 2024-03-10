use crate::{
    crossterm::event::{
        Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers, MouseEvent,
        MouseEventKind,
    },
    preset, Error, PromptSignal, Result,
};

/// Default key bindings for the tree.
///
/// | Key                    | Action
/// | :--------------------- | :-------------------------------------------
/// | <kbd>Enter</kbd>       | Exit the tree view
/// | <kbd>Ctrl + C</kbd>    | Interrupt the current operation
/// | <kbd>↑</kbd>           | Move the selection up
/// | <kbd>↓</kbd>           | Move the selection down
/// | <kbd>Space</kbd>       | Toggle fold/unfold at the current node
pub fn default(
    renderer: &mut preset::tree::render::Renderer,
    event: &Event,
) -> Result<PromptSignal> {
    let mut tree_borrowed_after = renderer.tree_snapshot.borrow_mut_after();

    match event {
        Event::Key(KeyEvent {
            code: KeyCode::Enter,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => return Ok(PromptSignal::Quit),
        Event::Key(KeyEvent {
            code: KeyCode::Char('c'),
            modifiers: KeyModifiers::CONTROL,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => return Err(Error::Interrupted("ctrl+c".into())),

        // Move cursor.
        Event::Key(KeyEvent {
            code: KeyCode::Up,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => {
            tree_borrowed_after.tree.backward();
        }
        Event::Mouse(MouseEvent {
            kind: MouseEventKind::ScrollUp,
            column: _,
            row: _,
            modifiers: KeyModifiers::NONE,
        }) => {
            tree_borrowed_after.tree.backward();
        }

        Event::Key(KeyEvent {
            code: KeyCode::Down,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => {
            tree_borrowed_after.tree.forward();
        }
        Event::Mouse(MouseEvent {
            kind: MouseEventKind::ScrollDown,
            column: _,
            row: _,
            modifiers: KeyModifiers::NONE,
        }) => {
            tree_borrowed_after.tree.forward();
        }

        // Fold/Unfold
        Event::Key(KeyEvent {
            code: KeyCode::Char(' '),
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => {
            tree_borrowed_after.tree.toggle();
        }

        _ => (),
    }
    Ok(PromptSignal::Continue)
}
