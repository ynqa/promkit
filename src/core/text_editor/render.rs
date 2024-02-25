use std::any::Any;

use crate::{
    crossterm::{
        event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers},
        style::ContentStyle,
    },
    grapheme::{matrixify, Graphemes},
    pane::Pane,
    render::{AsAny, Renderable, State},
};

use super::{History, Mode, Suggest, TextEditor};

/// Represents a renderer for the `TextEditor` component,
/// capable of visualizing text input in a pane.
/// It supports a variety of features including history navigation,
/// input suggestions, input masking,
/// customizable prompt strings,
/// and styles for different parts of the input. It also handles different
/// edit modes such as insert and overwrite,
/// and can be configured to render a specific number of lines.
#[derive(Clone)]
pub struct Renderer {
    /// The `TextEditor` component to be rendered.
    pub texteditor: TextEditor,
    /// Optional history for navigating through previous inputs.
    pub history: Option<History>,
    /// Suggestion engine for input completion.
    pub suggest: Suggest,

    /// Prompt string displayed before the input text.
    pub prefix: String,
    /// Optional character used for masking the input string (e.g., for password fields).
    pub mask: Option<char>,

    /// Style applied to the prompt string.
    pub prefix_style: ContentStyle,
    /// Style applied to the currently selected character.
    pub active_char_style: ContentStyle,
    /// Style applied to characters that are not currently selected.
    pub inactive_char_style: ContentStyle,

    /// Current edit mode, determining whether input inserts or overwrites existing text.
    pub edit_mode: Mode,
    /// Number of lines available for rendering.
    pub lines: Option<usize>,
}

impl Renderable for Renderer {
    fn make_pane(&self, width: u16) -> Pane {
        let mut buf = Graphemes::default();
        buf.append(&mut Graphemes::new_with_style(
            &self.prefix,
            self.prefix_style,
        ));

        let text = match self.mask {
            Some(mask) => self.texteditor.masking(mask),
            None => self.texteditor.text(),
        };

        let mut styled = Graphemes::new_with_style(text, self.inactive_char_style)
            .apply_style_at(self.texteditor.position(), self.active_char_style);

        buf.append(&mut styled);

        Pane::new(
            matrixify(width as usize, &buf),
            self.texteditor.position() / width as usize,
            self.lines,
        )
    }

    /// Default key bindings for text editor.
    ///
    /// | Key                    | Description
    /// | :--                    | :--
    /// | <kbd> ← </kbd>         | Move the cursor backward
    /// | <kbd> → </kbd>         | Move the cursor forward
    /// | <kbd> Ctrl + A </kbd>  | Move the cursor to the beginning of the input buffer
    /// | <kbd> Ctrl + E </kbd>  | Move the cursor to the end of the input buffer
    /// | <kbd> ↑ </kbd>         | Retrieve the previous input from history
    /// | <kbd> ↓ </kbd>         | Retrieve the next input from history
    /// | <kbd> Backspace </kbd> | Erase a character at the current cursor position
    /// | <kbd> Ctrl + U </kbd>  | Erase all characters on the current line
    /// | <kbd> TAB </kbd>       | Perform tab completion by searching for suggestions
    fn handle_event(&mut self, event: &Event) {
        match event {
            // Move cursor.
            Event::Key(KeyEvent {
                code: KeyCode::Left,
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE,
            }) => {
                self.texteditor.backward();
            }
            Event::Key(KeyEvent {
                code: KeyCode::Right,
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE,
            }) => {
                self.texteditor.forward();
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('a'),
                modifiers: KeyModifiers::CONTROL,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE,
            }) => self.texteditor.move_to_head(),
            Event::Key(KeyEvent {
                code: KeyCode::Char('e'),
                modifiers: KeyModifiers::CONTROL,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE,
            }) => self.texteditor.move_to_tail(),

            // Erase char(s).
            Event::Key(KeyEvent {
                code: KeyCode::Backspace,
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE,
            }) => self.texteditor.erase(),
            Event::Key(KeyEvent {
                code: KeyCode::Char('u'),
                modifiers: KeyModifiers::CONTROL,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE,
            }) => self.texteditor.erase_all(),

            // Choose history
            Event::Key(KeyEvent {
                code: KeyCode::Up,
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE,
            }) => {
                if let Some(ref mut history) = &mut self.history {
                    if history.backward() {
                        self.texteditor.replace(&history.get())
                    }
                }
            }
            Event::Key(KeyEvent {
                code: KeyCode::Down,
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE,
            }) => {
                if let Some(ref mut history) = &mut self.history {
                    if history.forward() {
                        self.texteditor.replace(&history.get())
                    }
                }
            }

            // Choose suggestion
            Event::Key(KeyEvent {
                code: KeyCode::Tab,
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE,
            }) => {
                if let Some(new) = self.suggest.search(self.texteditor.text_without_cursor()) {
                    self.texteditor.replace(new)
                }
            }

            // Input char.
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
            }) => match self.edit_mode {
                Mode::Insert => self.texteditor.insert(*ch),
                Mode::Overwrite => self.texteditor.overwrite(*ch),
            },

            _ => (),
        }
    }

    fn postrun(&mut self) {
        if let Some(ref mut history) = &mut self.history {
            history.insert(self.texteditor.text_without_cursor());
        }
        self.texteditor = TextEditor::default();
    }
}

impl AsAny for Renderer {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl State<Renderer> {
    pub fn text_changed(&self) -> bool {
        self.before.texteditor.text() != self.after.borrow().texteditor.text()
    }
}
