//! A minimal multiline REPL built directly from promkit widgets and core rendering.

use std::{collections::HashSet, io};

use futures::StreamExt;
use promkit::{TerminalModes, TerminalSession};
use promkit_widgets::{
    core::{
        crossterm::{
            event::{
                Event, EventStream, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers,
                MouseButton, MouseEvent, MouseEventKind,
            },
            execute,
            style::{Color, ContentStyle, Print},
        },
        render::{Renderer, SharedRenderer},
        ScreenPosition, Widget,
    },
    text_editor::{self, TextEditorHit},
};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Index {
    Editor,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Completion {
    Empty,
    Incomplete,
    Complete,
    Invalid,
}

#[derive(Debug, PartialEq, Eq)]
enum Control {
    Continue,
    Submit(String),
    Exit,
}

struct Repl {
    editor: text_editor::State,
    renderer: SharedRenderer<Index>,
}

impl Repl {
    async fn new() -> anyhow::Result<Self> {
        let editor = text_editor::State {
            config: text_editor::Config {
                prefix: "❯❯❯ ".into(),
                continuation_prefix: "... ".into(),
                prefix_style: ContentStyle {
                    foreground_color: Some(Color::DarkGreen),
                    ..Default::default()
                },
                active_char_style: ContentStyle {
                    background_color: Some(Color::DarkCyan),
                    ..Default::default()
                },
                word_break_chars: HashSet::from([' ', '\n']),
                lines: Some(8),
                ..Default::default()
            },
            ..Default::default()
        };
        let renderer = SharedRenderer::new(
            Renderer::try_new_with_graphemes([(Index::Editor, editor.create_graphemes())], true)
                .await?,
        );

        Ok(Self { editor, renderer })
    }

    async fn read(&mut self, events: &mut EventStream) -> anyhow::Result<Control> {
        while let Some(event) = events.next().await {
            let event = event?;
            if event.is_resize() {
                continue;
            }

            let control = self.handle_event(&event);
            if control != Control::Continue {
                return Ok(control);
            }

            self.renderer
                .update([(Index::Editor, self.editor.create_graphemes())])
                .render()
                .await?;
        }

        Ok(Control::Exit)
    }

    fn handle_event(&mut self, event: &Event) -> Control {
        match event {
            Event::Key(KeyEvent {
                code: KeyCode::Char('c'),
                modifiers: KeyModifiers::CONTROL,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE,
            }) => return Control::Exit,
            Event::Key(KeyEvent {
                code: KeyCode::Char('d'),
                modifiers: KeyModifiers::CONTROL,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE,
            }) => {
                let text = self.editor.texteditor.text_without_cursor().to_string();
                return if completion(&text) == Completion::Complete {
                    Control::Submit(text)
                } else {
                    Control::Continue
                };
            }
            Event::Key(KeyEvent {
                code: KeyCode::Enter,
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE,
            }) => return enter(&mut self.editor.texteditor),
            Event::Mouse(MouseEvent {
                kind: MouseEventKind::Down(MouseButton::Left),
                column,
                row,
                modifiers: KeyModifiers::NONE,
            }) => {
                self.move_to_click(*column, *row);
                return Control::Continue;
            }
            Event::Mouse(_) => return Control::Continue,
            _ => {}
        }

        match event {
            Event::Key(KeyEvent {
                code: KeyCode::Left,
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE,
            }) => {
                self.editor.texteditor.backward();
            }
            Event::Key(KeyEvent {
                code: KeyCode::Right,
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE,
            }) => {
                self.editor.texteditor.forward();
            }
            Event::Key(KeyEvent {
                code: KeyCode::Up,
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE,
            }) => {
                self.editor.texteditor.move_up();
            }
            Event::Key(KeyEvent {
                code: KeyCode::Down,
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE,
            }) => {
                self.editor.texteditor.move_down();
            }
            Event::Key(KeyEvent {
                code: KeyCode::Home,
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE,
            })
            | Event::Key(KeyEvent {
                code: KeyCode::Char('a'),
                modifiers: KeyModifiers::CONTROL,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE,
            }) => self.editor.texteditor.move_to_line_head(),
            Event::Key(KeyEvent {
                code: KeyCode::End,
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE,
            })
            | Event::Key(KeyEvent {
                code: KeyCode::Char('e'),
                modifiers: KeyModifiers::CONTROL,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE,
            }) => self.editor.texteditor.move_to_line_tail(),
            Event::Key(KeyEvent {
                code: KeyCode::Home,
                modifiers: KeyModifiers::CONTROL,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE,
            }) => self.editor.texteditor.move_to_head(),
            Event::Key(KeyEvent {
                code: KeyCode::End,
                modifiers: KeyModifiers::CONTROL,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE,
            }) => self.editor.texteditor.move_to_tail(),
            Event::Key(KeyEvent {
                code: KeyCode::Backspace,
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE,
            }) => self.editor.texteditor.erase(),
            Event::Key(KeyEvent {
                code: KeyCode::Delete,
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE,
            }) => self.editor.texteditor.erase_forward(),
            Event::Key(KeyEvent {
                code: KeyCode::Char('u'),
                modifiers: KeyModifiers::CONTROL,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE,
            }) => self.editor.texteditor.erase_all(),
            Event::Key(KeyEvent {
                code: KeyCode::Char('b'),
                modifiers: KeyModifiers::ALT,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE,
            }) => self
                .editor
                .texteditor
                .move_to_previous_nearest(&self.editor.config.word_break_chars),
            Event::Key(KeyEvent {
                code: KeyCode::Char('f'),
                modifiers: KeyModifiers::ALT,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE,
            }) => self
                .editor
                .texteditor
                .move_to_next_nearest(&self.editor.config.word_break_chars),
            Event::Key(KeyEvent {
                code: KeyCode::Char('w'),
                modifiers: KeyModifiers::CONTROL,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE,
            }) => self
                .editor
                .texteditor
                .erase_to_previous_nearest(&self.editor.config.word_break_chars),
            Event::Key(KeyEvent {
                code: KeyCode::Char('d'),
                modifiers: KeyModifiers::ALT,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE,
            }) => self
                .editor
                .texteditor
                .erase_to_next_nearest(&self.editor.config.word_break_chars),
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
            }) => match self.editor.config.edit_mode {
                text_editor::Mode::Insert => self.editor.texteditor.insert(*ch),
                text_editor::Mode::Overwrite => self.editor.texteditor.overwrite(*ch),
            },
            _ => {}
        }

        Control::Continue
    }

    fn move_to_click(&mut self, column: u16, row: u16) {
        let Some(position) = self.renderer.hit_test(ScreenPosition { row, column }) else {
            return;
        };
        if position.index != Index::Editor {
            return;
        }
        if let Some(TextEditorHit::Cursor { index }) =
            self.editor.hit_at(position.content_position())
        {
            self.editor.texteditor.move_to(index);
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let modes =
        TerminalModes::RAW_MODE | TerminalModes::HIDDEN_CURSOR | TerminalModes::MOUSE_CAPTURE;
    let _terminal_session = TerminalSession::try_new(modes)?;
    execute!(
        io::stdout(),
        Print("Bracket REPL (blank: submit, \"exit\": quit)\r\n"),
    )?;
    let mut events = EventStream::new();

    loop {
        let mut repl = Repl::new().await?;
        match repl.read(&mut events).await? {
            Control::Submit(text) if text.trim() == "exit" => break,
            Control::Submit(text) => {
                execute!(
                    io::stdout(),
                    Print(format!("\r\nresult:\r\n{}\r\n", text.replace('\n', "\r\n"))),
                )?;
            }
            Control::Exit => break,
            Control::Continue => unreachable!("the event loop only returns terminal actions"),
        }
    }

    execute!(io::stdout(), Print("\r\n"))?;
    Ok(())
}

fn enter(editor: &mut text_editor::TextEditor) -> Control {
    let text = editor.text_without_cursor().to_string();
    match completion(&text) {
        Completion::Complete
            if requires_blank_continuation_line(&text) && !ends_with_blank_line(&text) =>
        {
            insert_newline(editor);
            Control::Continue
        }
        Completion::Complete => Control::Submit(text),
        Completion::Empty | Completion::Incomplete => {
            insert_newline(editor);
            Control::Continue
        }
        Completion::Invalid => Control::Continue,
    }
}

/// A deliberately small completion policy for this example.
///
/// Only `{}`, and `[]` are syntax. Strings, comments, and escapes are not
/// interpreted. Balanced bracket input is submitted by an empty continuation
/// line; a language REPL should replace this with its parser or VM.
fn completion(text: &str) -> Completion {
    if text.trim().is_empty() {
        return Completion::Empty;
    }

    match delimiter_stack(text.chars()) {
        Ok(stack) if stack.is_empty() => Completion::Complete,
        Ok(_) => Completion::Incomplete,
        Err(()) => Completion::Invalid,
    }
}

fn requires_blank_continuation_line(text: &str) -> bool {
    text.chars().any(|character| matches!(character, '{' | '['))
}

fn ends_with_blank_line(text: &str) -> bool {
    text.rsplit_once('\n')
        .is_some_and(|(_, line)| line.trim().is_empty())
}

fn delimiter_stack(characters: impl IntoIterator<Item = char>) -> Result<Vec<char>, ()> {
    let mut stack = Vec::new();

    for character in characters {
        match character {
            '{' | '[' => stack.push(character),
            '}' if stack.pop() != Some('{') => return Err(()),
            ']' if stack.pop() != Some('[') => return Err(()),
            _ => {}
        }
    }

    Ok(stack)
}

fn insert_newline(editor: &mut text_editor::TextEditor) {
    let depth = delimiter_stack(
        editor
            .text_without_cursor()
            .iter()
            .take(editor.position())
            .map(|grapheme| grapheme.character()),
    )
    .map(|stack| stack.len())
    .unwrap_or_default();

    editor.insert_newline();
    editor.insert_chars(&"    ".repeat(depth).chars().collect());
}

#[cfg(test)]
mod tests {
    use super::*;
    use promkit_widgets::core::Widget;

    mod completion {
        use super::*;

        #[test]
        fn classifies_empty_balanced_unbalanced_and_invalid_input() {
            assert_eq!(completion(""), Completion::Empty);
            assert_eq!(completion("value {"), Completion::Incomplete);
            assert_eq!(completion("value {\n    [item]\n}"), Completion::Complete);
            assert_eq!(completion("{]"), Completion::Invalid);
        }
    }

    mod enter {
        use super::*;

        #[test]
        fn continues_unclosed_input_with_indentation() {
            let mut editor = text_editor::TextEditor::new("{");

            assert_eq!(enter(&mut editor), Control::Continue);
            assert_eq!(editor.text_without_cursor().to_string(), "{\n    ");
        }

        #[test]
        fn submits_complete_single_line_input() {
            let mut editor = text_editor::TextEditor::new("value");

            assert_eq!(enter(&mut editor), Control::Submit("value".to_string()));
        }

        fn assert_blank_continuation_line_submits(input: &str) {
            let mut editor = text_editor::TextEditor::new(input);

            assert_eq!(enter(&mut editor), Control::Continue);
            assert_eq!(
                editor.text_without_cursor().to_string(),
                format!("{input}\n")
            );
            assert_eq!(enter(&mut editor), Control::Submit(format!("{input}\n")));
        }

        #[test]
        fn empty_continuation_line_submits_a_curly_bracket_block() {
            assert_blank_continuation_line_submits("{\n    value\n}");
        }

        #[test]
        fn empty_continuation_line_submits_a_square_bracket_block() {
            assert_blank_continuation_line_submits("[\n    value\n]");
        }
    }

    mod text_editor_state {
        use super::*;

        mod create_graphemes {
            use super::*;

            #[test]
            fn continuation_prefix_is_only_presentational() {
                let state = text_editor::State {
                    texteditor: text_editor::TextEditor::new("first\nsecond"),
                    config: text_editor::Config {
                        prefix: "❯❯❯ ".into(),
                        continuation_prefix: "... ".into(),
                        ..Default::default()
                    },
                    ..Default::default()
                };

                assert_eq!(
                    state.create_graphemes().graphemes.to_string(),
                    "❯❯❯ first\n... second "
                );
                assert_eq!(
                    state.texteditor.text_without_cursor().to_string(),
                    "first\nsecond"
                );
            }
        }
    }
}
