use crate::{
    crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers},
    grapheme::{Grapheme, Graphemes},
    pane::Pane,
    text::TextBuffer,
};

use super::Editor;

pub struct TextEditor {
    pub(crate) textbuffer: TextBuffer,

    pub label: Graphemes,
}

impl Editor for TextEditor {
    fn gen_pane(&self, width: u16) -> Pane {
        let mut buf = vec![];
        buf.append(&mut self.label.clone());
        buf.append(&mut self.textbuffer.buf.clone());

        let mut layout = vec![];
        let mut row = Graphemes::default();
        for ch in buf.iter() {
            let width_with_next_char = row.iter().fold(0, |mut layout, g| {
                layout += g.width;
                layout
            }) + ch.width;
            if !row.is_empty() && (width as usize) < width_with_next_char {
                layout.push(row);
                row = Graphemes::default();
            }
            if (width as usize) >= ch.width {
                row.push(ch.clone());
            }
        }
        layout.push(row);
        Pane {
            layout,
            offset: self.textbuffer.position / width as usize,
        }
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

#[cfg(test)]
mod test {
    mod matrixify {
        use super::super::*;

        #[test]
        fn test() {
            let expect = vec![
                Graphemes::from(">>"),
                Graphemes::from(" a"),
                Graphemes::from("aa"),
                Graphemes::from(" "),
            ];
            assert_eq!(
                expect,
                TextEditor {
                    textbuffer: TextBuffer {
                        buf: Graphemes::from("aaa "),
                        position: 0,
                    },
                    label: Graphemes::from(">> "),
                }
                .gen_pane(2)
                .layout,
            );
        }

        #[test]
        fn test_with_emoji() {
            let expect = vec![
                Graphemes::from(">>"),
                Graphemes::from(" "),
                Graphemes::from("😎"),
                Graphemes::from("😎"),
                Graphemes::from(" "),
            ];
            assert_eq!(
                expect,
                TextEditor {
                    textbuffer: TextBuffer {
                        buf: Graphemes::from("😎😎 "),
                        position: 0,
                    },
                    label: Graphemes::from(">> "),
                }
                .gen_pane(2)
                .layout,
            );
        }

        #[test]
        fn test_with_emoji_at_narrow_terminal() {
            let expect = vec![
                Graphemes::from(">"),
                Graphemes::from(">"),
                Graphemes::from(" "),
                Graphemes::from(" "),
            ];
            assert_eq!(
                expect,
                TextEditor {
                    textbuffer: TextBuffer {
                        buf: Graphemes::from("😎😎 "),
                        position: 0,
                    },
                    label: Graphemes::from(">> "),
                }
                .gen_pane(1)
                .layout,
            );
        }
    }
}
