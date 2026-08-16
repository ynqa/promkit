use promkit::{
    core::{
        crossterm::{
            event::{
                Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers, MouseButton,
                MouseEvent, MouseEventKind,
            },
            style::ContentStyle,
        },
        ScreenPosition,
    },
    widgets::{
        prefix_search::PrefixSearchHit,
        text::Text,
        text_editor::{self, TextEditorHit},
    },
    Signal,
};

use crate::{Focus, Index, Readline};

pub async fn default(event: &Event, ctx: &mut Readline) -> anyhow::Result<Signal> {
    match event {
        Event::Key(KeyEvent {
            code: KeyCode::Char('c'),
            modifiers: KeyModifiers::CONTROL,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => Err(anyhow::anyhow!("ctrl+c")),
        Event::Mouse(MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column,
            row,
            modifiers: KeyModifiers::NONE,
        }) => {
            click(*column, *row, ctx);
            Ok(Signal::Continue)
        }
        Event::Mouse(_) => Ok(Signal::Continue),
        _ => match ctx.focus {
            Focus::Readline => readline(event, ctx).await,
            Focus::Suggestion => suggestion(event, ctx).await,
        },
    }
}

fn click(column: u16, row: u16, ctx: &mut Readline) {
    let renderer = ctx.renderer.clone();
    let Some(position) = renderer
        .as_ref()
        .and_then(|renderer| renderer.hit_test(ScreenPosition { row, column }))
    else {
        return;
    };

    match position.index {
        Index::Readline => {
            if let Some(TextEditorHit::Cursor { index }) =
                ctx.readline.hit_at(position.content_position())
            {
                ctx.readline.texteditor.move_to(index);
            }
        }
        Index::Suggestion => {
            if let Some(PrefixSearchHit::Select { index }) =
                ctx.suggestions.hit_at(position.content_position())
            {
                select_suggestion(index, ctx);
            }
        }
        Index::Title | Index::ErrorMessage => {}
    }
}

fn select_suggestion(index: usize, ctx: &mut Readline) {
    if ctx.suggestions.result.move_to(index) {
        ctx.focus = Focus::Suggestion;
    }
}

fn apply_suggestion(ctx: &mut Readline) {
    let Some(suggestion) = ctx.suggestions.result.get() else {
        dismiss_suggestions(ctx);
        return;
    };

    ctx.readline.texteditor.replace(suggestion);
    dismiss_suggestions(ctx);
}

fn dismiss_suggestions(ctx: &mut Readline) {
    ctx.suggestions.result.clear();
    ctx.focus = Focus::Readline;
}

pub async fn readline(event: &Event, ctx: &mut Readline) -> anyhow::Result<Signal> {
    match event {
        Event::Key(KeyEvent {
            code: KeyCode::Enter,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => {
            let text = ctx.readline.texteditor.text_without_cursor().to_string();
            let valid = ctx
                .validator
                .as_ref()
                .map(|validator| {
                    let valid = validator.validate(&text);
                    if !valid {
                        ctx.error_message.text =
                            Text::from(validator.generate_error_message(&text));
                    }
                    valid
                })
                .unwrap_or(true);
            return if valid {
                if let Some(ref mut history) = &mut ctx.readline.history {
                    history.insert(text);
                }
                ctx.readline.config.active_char_style = ContentStyle::default();
                Ok(Signal::Quit)
            } else {
                Ok(Signal::Continue)
            };
        }
        Event::Key(KeyEvent {
            code: KeyCode::Tab,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => {
            let text = ctx.readline.texteditor.text_without_cursor().to_string();
            ctx.suggestions.result = ctx.prefix_search.query(text);
            if !ctx.suggestions.result.is_empty() {
                ctx.focus = Focus::Suggestion;
            } else {
                dismiss_suggestions(ctx);
            }
        }
        Event::Key(KeyEvent {
            code: KeyCode::Left,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => {
            ctx.readline.texteditor.backward();
        }
        Event::Key(KeyEvent {
            code: KeyCode::Right,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => {
            ctx.readline.texteditor.forward();
        }
        Event::Key(KeyEvent {
            code: KeyCode::Char('a'),
            modifiers: KeyModifiers::CONTROL,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => ctx.readline.texteditor.move_to_head(),
        Event::Key(KeyEvent {
            code: KeyCode::Char('e'),
            modifiers: KeyModifiers::CONTROL,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => ctx.readline.texteditor.move_to_tail(),
        Event::Key(KeyEvent {
            code: KeyCode::Char('b'),
            modifiers: KeyModifiers::ALT,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => ctx
            .readline
            .texteditor
            .move_to_previous_nearest(&ctx.readline.config.word_break_chars),
        Event::Key(KeyEvent {
            code: KeyCode::Char('f'),
            modifiers: KeyModifiers::ALT,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => ctx
            .readline
            .texteditor
            .move_to_next_nearest(&ctx.readline.config.word_break_chars),
        Event::Key(KeyEvent {
            code: KeyCode::Backspace,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => ctx.readline.texteditor.erase(),
        Event::Key(KeyEvent {
            code: KeyCode::Char('u'),
            modifiers: KeyModifiers::CONTROL,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => ctx.readline.texteditor.erase_all(),
        Event::Key(KeyEvent {
            code: KeyCode::Char('w'),
            modifiers: KeyModifiers::CONTROL,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => ctx
            .readline
            .texteditor
            .erase_to_previous_nearest(&ctx.readline.config.word_break_chars),
        Event::Key(KeyEvent {
            code: KeyCode::Char('d'),
            modifiers: KeyModifiers::ALT,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => ctx
            .readline
            .texteditor
            .erase_to_next_nearest(&ctx.readline.config.word_break_chars),
        Event::Key(KeyEvent {
            code: KeyCode::Up,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => {
            if let Some(ref mut history) = &mut ctx.readline.history {
                if history.backward() {
                    ctx.readline.texteditor.replace(&history.get())
                }
            }
        }
        Event::Key(KeyEvent {
            code: KeyCode::Down,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => {
            if let Some(ref mut history) = &mut ctx.readline.history {
                if history.forward() {
                    ctx.readline.texteditor.replace(&history.get())
                }
            }
        }
        Event::Key(KeyEvent {
            code: KeyCode::Char(ch),
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        })
        | Event::Key(KeyEvent {
            code: KeyCode::Char(ch),
            modifiers: KeyModifiers::SHIFT,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => match ctx.readline.config.edit_mode {
            text_editor::Mode::Insert => ctx.readline.texteditor.insert(*ch),
            text_editor::Mode::Overwrite => ctx.readline.texteditor.overwrite(*ch),
        },
        _ => (),
    }
    Ok(Signal::Continue)
}

pub async fn suggestion(event: &Event, ctx: &mut Readline) -> anyhow::Result<Signal> {
    match event {
        Event::Key(KeyEvent {
            code: KeyCode::Enter,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => {
            apply_suggestion(ctx);
        }
        Event::Key(KeyEvent {
            code: KeyCode::Tab,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        })
        | Event::Key(KeyEvent {
            code: KeyCode::Down,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => {
            ctx.suggestions.result.forward();
        }
        Event::Key(KeyEvent {
            code: KeyCode::Up,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => {
            ctx.suggestions.result.backward();
        }
        _ => {
            let before = ctx.readline.texteditor.text_without_cursor();
            let signal = readline(event, ctx).await?;
            let after = ctx.readline.texteditor.text_without_cursor();
            if before != after {
                dismiss_suggestions(ctx);
            }
            return Ok(signal);
        }
    }
    Ok(Signal::Continue)
}
