use crate::{
    crossterm::{
        event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers},
        style::ContentStyle,
    },
    grapheme::{matrixify, Graphemes},
    history::History,
    pane::Pane,
    suggest::Suggest,
    text_buffer::TextBuffer,
};

use super::Editor;

pub struct TextEditor {
    pub textbuffer: TextBuffer,
    pub history: History,
    pub suggest: Suggest,

    pub label: String,
    pub label_style: ContentStyle,
    pub style: ContentStyle,
    pub cursor_style: ContentStyle,
}

impl Editor for TextEditor {
    fn gen_pane(&self, width: u16) -> Pane {
        let mut buf = Graphemes::default();
        buf.append(&mut Graphemes::new_with_style(
            &self.label,
            self.label_style,
        ));
        buf.append(&mut self.textbuffer.graphemes(self.style, self.cursor_style));

        Pane::new(
            matrixify(width as usize, buf),
            self.textbuffer.position / width as usize,
        )
    }

    fn handle_event(&mut self, event: &Event) {
        match event {
            // Before finishing on enter event.
            Event::Key(KeyEvent {
                code: KeyCode::Enter,
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE,
            }) => {
                // Insert the result to history.
                self.history.insert(self.textbuffer.to_string());
                [TextBuffer::default(), TextBuffer::default()]
            }

            // Move cursor.
            Event::Key(KeyEvent {
                code: KeyCode::Left,
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE,
            }) => self.textbuffer.prev(),
            Event::Key(KeyEvent {
                code: KeyCode::Right,
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE,
            }) => self.textbuffer.next(),
            Event::Key(KeyEvent {
                code: KeyCode::Char('a'),
                modifiers: KeyModifiers::CONTROL,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE,
            }) => self.textbuffer.move_to_head(),
            Event::Key(KeyEvent {
                code: KeyCode::Char('e'),
                modifiers: KeyModifiers::CONTROL,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE,
            }) => self.textbuffer.move_to_tail(),

            // Erase char.
            Event::Key(KeyEvent {
                code: KeyCode::Backspace,
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE,
            }) => self.textbuffer.erase(),

            // Choose history
            Event::Key(KeyEvent {
                code: KeyCode::Up,
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE,
            }) => {
                if self.history.prev() {
                    self.textbuffer.replace(self.history.get())
                } else {
                    [TextBuffer::default(), TextBuffer::default()]
                }
            }

            Event::Key(KeyEvent {
                code: KeyCode::Down,
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE,
            }) => {
                if self.history.next() {
                    self.textbuffer.replace(self.history.get())
                } else {
                    [TextBuffer::default(), TextBuffer::default()]
                }
            }

            // Choose suggestion
            Event::Key(KeyEvent {
                code: KeyCode::Tab,
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE,
            }) => {
                if let Some(new) = self.suggest.search(self.textbuffer.to_string()) {
                    self.textbuffer.replace(new)
                } else {
                    [TextBuffer::default(), TextBuffer::default()]
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
            }) => self.textbuffer.insert(*ch),

            _ => [TextBuffer::default(), TextBuffer::default()],
        };
    }

    fn reset(&mut self) {
        self.textbuffer = TextBuffer::default();
    }

    fn output(&self) -> String {
        self.textbuffer.to_string()
    }
}
