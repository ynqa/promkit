use crate::{
    core::{
        crossterm::event::{
            Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers, MouseButton,
            MouseEvent, MouseEventKind,
        },
        ScreenPosition,
    },
    preset::text::{Index, Text},
    widgets::text::TextHit,
    Signal,
};

/// Default key bindings for the text.
///
/// | Key                    | Action
/// | :--------------------- | :-------------------------------------------
/// | <kbd>Enter</kbd>       | Exit the text
/// | <kbd>Ctrl + C</kbd>    | Interrupt the current operation
/// | <kbd>↑</kbd>           | Move the selection up
/// | <kbd>↓</kbd>           | Move the selection down
/// | Left click             | Move the selection to the clicked line
pub async fn default(event: &Event, ctx: &mut Text) -> anyhow::Result<Signal> {
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
        })
        | Event::Mouse(MouseEvent {
            kind: MouseEventKind::ScrollUp,
            column: _,
            row: _,
            modifiers: KeyModifiers::NONE,
        }) => {
            ctx.text.text.backward();
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
            ctx.text.text.forward();
        }

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
                .filter(|position| position.index == Index::Text)
            {
                if let Some(TextHit::Select { index }) =
                    ctx.text.hit_at(position.content_position())
                {
                    ctx.text.text.move_to(index);
                }
            }
        }

        _ => (),
    }
    Ok(Signal::Continue)
}
