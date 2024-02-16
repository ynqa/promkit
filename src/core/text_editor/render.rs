use std::any::Any;

use crate::{
    crossterm::{
        event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers},
        style::ContentStyle,
    },
    error::Result,
    grapheme::{matrixify, Graphemes},
    pane::Pane,
    render::{AsAny, Renderable, State},
};

use super::{History, Mode, Suggest, TextEditor};

#[derive(Clone)]
pub struct Renderer {
    pub texteditor: TextEditor,
    pub history: Option<History>,
    pub suggest: Suggest,

    /// Prompt string.
    pub ps: String,
    /// Character to use for masking the input string.
    pub mask: Option<char>,

    /// Style for prompt string.
    pub ps_style: ContentStyle,
    /// Style for selected character.
    pub active_char_style: ContentStyle,
    /// Style for un-selected character.
    pub inactive_char_style: ContentStyle,

    /// Edit mode: insert or overwrite.
    pub mode: Mode,
    /// Window size.
    pub window_size: Option<usize>,
}

impl State<Renderer> {
    #![allow(clippy::too_many_arguments)]
    pub fn try_new(
        texteditor: TextEditor,
        history: Option<History>,
        suggest: Suggest,
        ps: String,
        mask: Option<char>,
        ps_style: ContentStyle,
        active_char_style: ContentStyle,
        inactive_char_style: ContentStyle,
        mode: Mode,
        window_size: Option<usize>,
    ) -> Result<Box<State<Renderer>>> {
        Ok(Box::new(State::<Renderer>::new(Renderer {
            texteditor,
            history,
            suggest,
            ps,
            ps_style,
            active_char_style,
            inactive_char_style,
            mode,
            mask,
            window_size,
        })))
    }
}

impl Renderable for Renderer {
    fn make_pane(&self, width: u16) -> Pane {
        let mut buf = Graphemes::default();
        buf.append(&mut Graphemes::new_with_style(&self.ps, self.ps_style));

        let text = match self.mask {
            Some(mask) => self.texteditor.masking(mask),
            None => self.texteditor.text(),
        };

        let mut styled = Graphemes::new_with_style(text, self.inactive_char_style)
            .stylize(self.texteditor.position(), self.active_char_style);

        buf.append(&mut styled);

        Pane::new(
            matrixify(width as usize, &buf),
            self.texteditor.position() / width as usize,
            self.window_size,
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
            }) => match self.mode {
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
