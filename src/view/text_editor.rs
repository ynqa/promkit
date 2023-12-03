use std::any::Any;

use crate::{
    crossterm::{
        event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers},
        style::ContentStyle,
    },
    grapheme::{matrixify, Graphemes},
    pane::Pane,
    text::Text,
};

use super::{AsAny, State, Viewable};

mod history;
pub use history::History;
mod suggest;
pub use suggest::Suggest;

/// Edit mode.
#[derive(Clone, Default)]
pub enum Mode {
    #[default]
    /// Insert a char at the current position.
    Insert,
    /// Overwrite a char at the current position.
    Overwrite,
}

#[derive(Clone)]
pub struct TextEditorViewer {
    pub text: Text,
    pub history: Option<History>,
    pub suggest: Suggest,

    pub prefix: String,
    pub prefix_style: ContentStyle,
    pub style: ContentStyle,
    pub cursor_style: ContentStyle,
    pub mode: Mode,
    pub mask: Option<char>,
    pub lines: Option<usize>,
}

impl TextEditorViewer {
    fn text_to_graphemes(&self) -> Graphemes {
        let text = match self.mask {
            Some(mask) => self.text.masking(mask),
            None => self.text.content(),
        };
        Graphemes::new_with_style(text, self.style).stylize(self.text.position, self.cursor_style)
    }
}

impl Viewable for TextEditorViewer {
    fn make_pane(&self, width: u16) -> Pane {
        let mut buf = Graphemes::default();
        buf.append(&mut Graphemes::new_with_style(
            &self.prefix,
            self.prefix_style,
        ));
        buf.append(&mut self.text_to_graphemes());

        Pane::new(
            matrixify(width as usize, &buf),
            self.text.position / width as usize,
            self.lines,
        )
    }

    /// Default key bindings for text editor.
    ///
    /// | Key                    | Description
    /// | :--                    | :--
    /// | <kbd> Enter </kbd>     | Exit the event-loop
    /// | <kbd> CTRL + C </kbd>  | Exit the event-loop with an error
    /// | <kbd> ← </kbd>         | Move the cursor backward
    /// | <kbd> → </kbd>         | Move the cursor forward
    /// | <kbd> CTRL + A </kbd>  | Move the cursor to the beginning of the input buffer
    /// | <kbd> CTRL + E </kbd>  | Move the cursor to the end of the input buffer
    /// | <kbd> ↑ </kbd>         | Retrieve the previous input from history
    /// | <kbd> ↓ </kbd>         | Retrieve the next input from history
    /// | <kbd> Backspace </kbd> | Erase a character at the current cursor position
    /// | <kbd> CTRL + U </kbd>  | Erase all characters on the current line
    /// | <kbd> TAB </kbd>       | Perform tab completion by searching for suggestions
    fn handle_event(&mut self, event: &Event) {
        match event {
            // Move cursor.
            Event::Key(KeyEvent {
                code: KeyCode::Left,
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE,
            }) => self.text.backward(),
            Event::Key(KeyEvent {
                code: KeyCode::Right,
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE,
            }) => self.text.forward(),
            Event::Key(KeyEvent {
                code: KeyCode::Char('a'),
                modifiers: KeyModifiers::CONTROL,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE,
            }) => self.text.move_to_head(),
            Event::Key(KeyEvent {
                code: KeyCode::Char('e'),
                modifiers: KeyModifiers::CONTROL,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE,
            }) => self.text.move_to_tail(),

            // Erase char(s).
            Event::Key(KeyEvent {
                code: KeyCode::Backspace,
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE,
            }) => self.text.erase(),
            Event::Key(KeyEvent {
                code: KeyCode::Char('u'),
                modifiers: KeyModifiers::CONTROL,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE,
            }) => self.text.erase_all(),

            // Choose history
            Event::Key(KeyEvent {
                code: KeyCode::Up,
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE,
            }) => {
                if let Some(ref mut history) = &mut self.history {
                    if history.backward() {
                        self.text.replace(&history.get())
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
                        self.text.replace(&history.get())
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
                if let Some(new) = self.suggest.search(self.text.content_without_cursor()) {
                    self.text.replace(new)
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
            }) => match self.mode {
                Mode::Insert => self.text.insert(*ch),
                Mode::Overwrite => self.text.overwrite(*ch),
            },

            _ => (),
        }
    }

    fn postrun(&mut self) {
        if let Some(ref mut history) = &mut self.history {
            history.insert(self.text.content_without_cursor());
        }
        self.text = Text::default();
    }
}

impl AsAny for TextEditorViewer {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl State<TextEditorViewer> {
    pub fn text_changed(&self) -> bool {
        self.before.text.content() != self.after.borrow().text.content()
    }
}
