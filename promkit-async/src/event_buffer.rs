use std::time::Duration;

use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
use futures::future::{Future, FutureExt};
use futures_timer::Delay;
use tokio::sync::mpsc::{Receiver, Sender};

#[derive(Clone, Debug, PartialEq)]
pub enum WrappedEvent {
    KeyBuffer(Vec<char>),
    VerticalCursorBuffer(usize, usize),   // (up, down)
    HorizontalCursorBuffer(usize, usize), // (left, right)
    Other(Event),
}

pub struct EventBuffer {
    delay_duration: Duration,
}

impl EventBuffer {
    pub fn new() -> Self {
        EventBuffer {
            delay_duration: Duration::from_millis(10),
        }
    }

    pub fn run(
        &mut self,
        mut event_receiver: Receiver<Event>,
        sequential_events_sender: Sender<Vec<WrappedEvent>>,
    ) -> impl Future<Output = anyhow::Result<()>> + Send {
        let mut buffer = Vec::new();
        let delay_duration = self.delay_duration;

        async move {
            loop {
                let delay = Delay::new(delay_duration).fuse();
                futures::pin_mut!(delay);

                futures::select! {
                    maybe_event = event_receiver.recv().fuse() => {
                        if let Some(event) = maybe_event {
                            buffer.push(event);
                        } else {
                            break;
                        }
                    },
                    _ = delay => {
                        if !buffer.is_empty() {
                            sequential_events_sender.send(Self::sequential_buffer(buffer.clone())).await?;
                            buffer.clear();
                        }
                    },
                }
            }
            Ok(())
        }
    }

    fn sequential_buffer(events: Vec<Event>) -> Vec<WrappedEvent> {
        let mut ret = Vec::new();
        let mut charbuf = Vec::new();
        let mut vertical_cursor = (0, 0); // (up, down)
        let mut horizontal_cursor = (0, 0); // (left, right)

        for event in events {
            if let Some(ch) = Self::extract_char(&event) {
                charbuf.push(ch);
                // Check and insert if other aggregates are not edited
                if vertical_cursor != (0, 0) {
                    ret.push(WrappedEvent::VerticalCursorBuffer(
                        vertical_cursor.0,
                        vertical_cursor.1,
                    ));
                } else if horizontal_cursor != (0, 0) {
                    ret.push(WrappedEvent::HorizontalCursorBuffer(
                        horizontal_cursor.0,
                        horizontal_cursor.1,
                    ));
                }
                // Initialize other aggregates
                vertical_cursor = (0, 0);
                horizontal_cursor = (0, 0);
            } else if let Some(direction) = Self::detect_vertical_direction(&event) {
                vertical_cursor.0 += direction.0;
                vertical_cursor.1 += direction.1;
                // Check and insert if other aggregates are not edited
                if !charbuf.is_empty() {
                    ret.push(WrappedEvent::KeyBuffer(charbuf.clone()));
                } else if horizontal_cursor != (0, 0) {
                    ret.push(WrappedEvent::HorizontalCursorBuffer(
                        horizontal_cursor.0,
                        horizontal_cursor.1,
                    ));
                }
                // Initialize other aggregates
                charbuf.clear();
                horizontal_cursor = (0, 0);
            } else if let Some(direction) = Self::detect_horizontal_direction(&event) {
                horizontal_cursor.0 += direction.0;
                horizontal_cursor.1 += direction.1;
                // Check and insert if other aggregates are not edited
                if !charbuf.is_empty() {
                    ret.push(WrappedEvent::KeyBuffer(charbuf.clone()));
                } else if vertical_cursor != (0, 0) {
                    ret.push(WrappedEvent::VerticalCursorBuffer(
                        vertical_cursor.0,
                        vertical_cursor.1,
                    ));
                }
                // Initialize other aggregates
                charbuf.clear();
                vertical_cursor = (0, 0);
            } else {
                // Check and insert if other aggregates are not edited
                if !charbuf.is_empty() {
                    ret.push(WrappedEvent::KeyBuffer(charbuf.clone()));
                } else if vertical_cursor != (0, 0) {
                    ret.push(WrappedEvent::VerticalCursorBuffer(
                        vertical_cursor.0,
                        vertical_cursor.1,
                    ));
                } else if horizontal_cursor != (0, 0) {
                    ret.push(WrappedEvent::HorizontalCursorBuffer(
                        horizontal_cursor.0,
                        horizontal_cursor.1,
                    ));
                }
                // Without buffering for other events
                ret.push(WrappedEvent::Other(event));
                // Initialize other aggregates
                charbuf.clear();
                vertical_cursor = (0, 0);
                horizontal_cursor = (0, 0);
            }
        }

        // Handle the last event
        if !charbuf.is_empty() {
            ret.push(WrappedEvent::KeyBuffer(charbuf.clone()));
        } else if vertical_cursor != (0, 0) {
            ret.push(WrappedEvent::VerticalCursorBuffer(
                vertical_cursor.0,
                vertical_cursor.1,
            ));
        } else if horizontal_cursor != (0, 0) {
            ret.push(WrappedEvent::HorizontalCursorBuffer(
                horizontal_cursor.0,
                horizontal_cursor.1,
            ));
        }

        ret
    }

    fn extract_char(event: &Event) -> Option<char> {
        match event {
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
            }) => Some(*ch),
            _ => None,
        }
    }

    fn detect_vertical_direction(event: &Event) -> Option<(usize, usize)> {
        match event {
            Event::Key(KeyEvent {
                code: KeyCode::Up, ..
            }) => Some((1, 0)),
            Event::Key(KeyEvent {
                code: KeyCode::Down,
                ..
            }) => Some((0, 1)),
            _ => None,
        }
    }

    fn detect_horizontal_direction(event: &Event) -> Option<(usize, usize)> {
        match event {
            Event::Key(KeyEvent {
                code: KeyCode::Left,
                ..
            }) => Some((1, 0)),
            Event::Key(KeyEvent {
                code: KeyCode::Right,
                ..
            }) => Some((0, 1)),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod sequential_buffer {
        use super::*;

        #[test]
        fn test() {
            let events = vec![
                Event::Key(KeyEvent {
                    code: KeyCode::Char('a'),
                    modifiers: KeyModifiers::NONE,
                    kind: KeyEventKind::Press,
                    state: KeyEventState::NONE,
                }),
                Event::Key(KeyEvent {
                    code: KeyCode::Char('B'),
                    modifiers: KeyModifiers::SHIFT,
                    kind: KeyEventKind::Press,
                    state: KeyEventState::NONE,
                }),
                Event::Key(KeyEvent {
                    code: KeyCode::Char('c'),
                    modifiers: KeyModifiers::NONE,
                    kind: KeyEventKind::Press,
                    state: KeyEventState::NONE,
                }),
                Event::Key(KeyEvent {
                    code: KeyCode::Up,
                    modifiers: KeyModifiers::NONE,
                    kind: KeyEventKind::Press,
                    state: KeyEventState::NONE,
                }),
                Event::Key(KeyEvent {
                    code: KeyCode::Down,
                    modifiers: KeyModifiers::NONE,
                    kind: KeyEventKind::Press,
                    state: KeyEventState::NONE,
                }),
                Event::Key(KeyEvent {
                    code: KeyCode::Up,
                    modifiers: KeyModifiers::NONE,
                    kind: KeyEventKind::Press,
                    state: KeyEventState::NONE,
                }),
                Event::Key(KeyEvent {
                    code: KeyCode::Left,
                    modifiers: KeyModifiers::NONE,
                    kind: KeyEventKind::Press,
                    state: KeyEventState::NONE,
                }),
                Event::Key(KeyEvent {
                    code: KeyCode::Right,
                    modifiers: KeyModifiers::NONE,
                    kind: KeyEventKind::Press,
                    state: KeyEventState::NONE,
                }),
                Event::Key(KeyEvent {
                    code: KeyCode::Left,
                    modifiers: KeyModifiers::NONE,
                    kind: KeyEventKind::Press,
                    state: KeyEventState::NONE,
                }),
                Event::Key(KeyEvent {
                    code: KeyCode::Char('f'),
                    modifiers: KeyModifiers::CONTROL,
                    kind: KeyEventKind::Press,
                    state: KeyEventState::NONE,
                }),
                Event::Key(KeyEvent {
                    code: KeyCode::Up,
                    modifiers: KeyModifiers::NONE,
                    kind: KeyEventKind::Press,
                    state: KeyEventState::NONE,
                }),
                Event::Key(KeyEvent {
                    code: KeyCode::Char('d'),
                    modifiers: KeyModifiers::NONE,
                    kind: KeyEventKind::Press,
                    state: KeyEventState::NONE,
                }),
            ];

            let expected = vec![
                WrappedEvent::KeyBuffer(vec!['a', 'B', 'c']),
                WrappedEvent::VerticalCursorBuffer(2, 1),
                WrappedEvent::HorizontalCursorBuffer(2, 1),
                WrappedEvent::Other(Event::Key(KeyEvent {
                    code: KeyCode::Char('f'),
                    modifiers: KeyModifiers::CONTROL,
                    kind: KeyEventKind::Press,
                    state: KeyEventState::NONE,
                })),
                WrappedEvent::VerticalCursorBuffer(1, 0),
                WrappedEvent::KeyBuffer(vec!['d']),
            ];

            assert_eq!(EventBuffer::sequential_buffer(events), expected);
        }
    }
}
