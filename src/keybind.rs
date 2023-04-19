use std::collections::HashMap;
use std::io;

use crate::{
    crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers},
    EventHandleFn, Handler, Output, Result,
};

/// Map key-events and their handlers.
pub struct KeyBind<S: Output> {
    pub event_mapping: HashMap<Event, Box<EventHandleFn<S>>>,

    pub handle_input: Option<Box<EventHandleFn<S>>>,
    pub handle_resize: Option<Box<EventHandleFn<S>>>,
}

impl<S: Output> KeyBind<S> {
    pub fn assign<I>(&mut self, items: I)
    where
        I: IntoIterator<Item = (Event, Box<EventHandleFn<S>>)>,
    {
        for (_, elem) in items.into_iter().enumerate() {
            self.event_mapping.insert(elem.0, elem.1);
        }
    }
}

impl<S: 'static + Output> Handler<S> for KeyBind<S> {
    fn handle(
        &mut self,
        ev: Event,
        out: &mut io::Stdout,
        state: &mut S,
    ) -> Result<Option<S::Output>> {
        match self.event_mapping.get(&ev) {
            Some(handle) => handle(None, None, out, state),
            None => match ev {
                Event::Resize(x, y) => match &self.handle_resize {
                    Some(func) => (func)(Some((x, y)), None, out, state),
                    None => Ok(None),
                },
                Event::Key(KeyEvent {
                    code: KeyCode::Char(ch),
                    modifiers: KeyModifiers::NONE,
                    ..
                })
                | Event::Key(KeyEvent {
                    code: KeyCode::Char(ch),
                    modifiers: KeyModifiers::SHIFT,
                    ..
                }) => match &self.handle_input {
                    Some(func) => (func)(None, Some(ch), out, state),
                    None => Ok(None),
                },
                _ => Ok(None),
            },
        }
    }
}

#[cfg(test)]
mod test {
    use super::{io, EventHandleFn, HashMap, KeyBind, Output};
    use crate::crossterm::event::{
        Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers,
    };

    use std::any::Any;

    impl Output for Box<dyn Any> {
        type Output = String;

        fn output(&self) -> Self::Output {
            "".to_owned()
        }
    }

    #[test]
    fn assign() {
        let mut b = KeyBind::<Box<dyn Any>> {
            event_mapping: HashMap::default(),
            handle_input: None,
            handle_resize: None,
        };
        b.assign(vec![(
            Event::Key(KeyEvent {
                code: KeyCode::Enter,
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                state: KeyEventState::empty(),
            }),
            Box::new(|_, _, _: &mut io::Stdout, state: &mut Box<dyn Any>| Ok(Some(state.output())))
                as Box<EventHandleFn<Box<dyn Any>>>,
        )]);
        assert_eq!(b.event_mapping.len(), 1);
    }
}
