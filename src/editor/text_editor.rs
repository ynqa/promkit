use crate::{
    crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers},
    grapheme::{matrixify, Grapheme, Graphemes},
    pane::Pane,
    text_buffer::TextBuffer,
};

use super::Editor;

pub struct TextEditor {
    pub textbuffer: TextBuffer,

    pub label: Graphemes,
}

impl Editor for TextEditor {
    fn gen_pane(&self, width: u16) -> Pane {
        let mut buf = Graphemes::default();
        buf.append(&mut self.label.clone());
        buf.append(&mut self.textbuffer.buf.clone());

        Pane::new(
            matrixify(width as usize, buf),
            self.textbuffer.position / width as usize,
        )
    }

    fn handle_event(&mut self, event: &Event) {
        match event {
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
            }) => self.textbuffer.to_head(),
            Event::Key(KeyEvent {
                code: KeyCode::Char('e'),
                modifiers: KeyModifiers::CONTROL,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE,
            }) => self.textbuffer.to_tail(),

            // Erase char.
            Event::Key(KeyEvent {
                code: KeyCode::Backspace,
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE,
            }) => self.textbuffer.erase(),

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
            }) => self.textbuffer.insert(Grapheme::from(*ch)),

            _ => [TextBuffer::new(), TextBuffer::new()],
        };
    }

    fn reset(&mut self) {
        self.textbuffer = TextBuffer::new();
    }

    fn to_string(&self) -> String {
        self.textbuffer.text().to_string()
    }
}
