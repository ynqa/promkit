use crate::{
    core::{
        crossterm::event::{
            Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers, MouseButton,
            MouseEvent, MouseEventKind,
        },
        ScreenPosition,
    },
    preset::yaml::{Index, Yaml},
    widgets::yaml::YamlHit,
    Signal,
};

/// Default key bindings for YAML navigation and manipulation.
///
/// | Key                    | Action
/// | :--------------------- | :-------------------------------------------|
/// | <kbd>Enter</kbd>       | Exit the YAML viewer
/// | <kbd>Ctrl + C</kbd>    | Interrupt the current operation
/// | <kbd>↑</kbd>           | Move the cursor up to the previous node
/// | <kbd>↓</kbd>           | Move the cursor down to the next node
/// | <kbd>Space</kbd>       | Toggle fold/unfold on the current node
/// | Left click             | Toggle fold/unfold on the clicked node
pub async fn default(event: &Event, ctx: &mut Yaml) -> anyhow::Result<Signal> {
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
            ctx.yaml.document.up();
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
            ctx.yaml.document.down();
        }

        // Fold/Unfold
        Event::Key(KeyEvent {
            code: KeyCode::Char(' '),
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => {
            ctx.yaml.document.toggle();
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
                .filter(|position| position.index == Index::Yaml)
            {
                if let Some(YamlHit::Toggle { row_index }) =
                    ctx.yaml.hit_at_viewport(position.content_position())
                {
                    ctx.yaml.document.toggle_at(row_index);
                }
            }
        }

        _ => (),
    }
    Ok(Signal::Continue)
}
