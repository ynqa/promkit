use std::{future::Future, pin::Pin};

use crate::{
    core::{
        crossterm::event::{
            Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers, MouseEvent,
            MouseEventKind,
        },
        PaneFactory,
    },
    preset::tree::{Index, Tree},
    Signal,
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
pub fn boxed_default<'a>(event: &'a Event, ctx: &'a mut Tree) -> Pin<Box<dyn Future<Output = anyhow::Result<Signal>> + Send + 'a>> {
    Box::pin(default(event, ctx))
}

async fn default(event: &Event, ctx: &mut Tree) -> anyhow::Result<Signal> {
    match event {
        Event::Resize(width, height) => {
            ctx.renderer
            .as_ref()
            .unwrap()
            .update([
                (Index::Title, ctx.title.create_pane(*width, *height)),
                (Index::Tree, ctx.tree.create_pane(*width, *height)),
            ])
            .render()
            .await?;
        }

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
            ctx.tree.tree.backward();
        }
        Event::Mouse(MouseEvent {
            kind: MouseEventKind::ScrollUp,
            column: _,
            row: _,
            modifiers: KeyModifiers::NONE,
        }) => {
            ctx.tree.tree.backward();
        }

        Event::Key(KeyEvent {
            code: KeyCode::Down,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => {
            ctx.tree.tree.forward();
        }
        Event::Mouse(MouseEvent {
            kind: MouseEventKind::ScrollDown,
            column: _,
            row: _,
            modifiers: KeyModifiers::NONE,
        }) => {
            ctx.tree.tree.forward();
        }

        // Fold/Unfold
        Event::Key(KeyEvent {
            code: KeyCode::Char(' '),
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => {
            ctx.tree.tree.toggle();
        }

        _ => (),
    }
    Ok(Signal::Continue)
}
