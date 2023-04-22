use std::collections::HashMap;
use std::io;

use crate::{crossterm::event::Event, Action, Output, Result};

/// Map key-events and their handlers.
pub struct KeyBind<S: Output> {
    pub event_mapping: HashMap<Event, Box<Action<S>>>,
}

impl<S: Output> KeyBind<S> {
    pub fn assign<I>(&mut self, items: I)
    where
        I: IntoIterator<Item = (Event, Box<Action<S>>)>,
    {
        for (_, elem) in items.into_iter().enumerate() {
            self.event_mapping.insert(elem.0, elem.1);
        }
    }

    pub fn handle(
        &mut self,
        ev: &Event,
        out: &mut io::Stdout,
        state: &mut S,
    ) -> Result<Option<S::Output>> {
        match self.event_mapping.get(ev) {
            Some(handle) => handle(out, state),
            _ => Ok(None),
        }
    }
}

#[cfg(test)]
mod test {
    use super::{io, Action, HashMap, KeyBind, Output};
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
        };
        b.assign(vec![(
            Event::Key(KeyEvent {
                code: KeyCode::Enter,
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                state: KeyEventState::empty(),
            }),
            Box::new(|_: &mut io::Stdout, state: &mut Box<dyn Any>| Ok(Some(state.output())))
                as Box<Action<Box<dyn Any>>>,
        )]);
        assert_eq!(b.event_mapping.len(), 1);
    }
}
