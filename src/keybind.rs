use std::collections::HashMap;
use std::io;

use crate::{crossterm::event::Event, Action, Result, UpstreamContext};

/// Map key-events and their handlers.
pub struct KeyBind<S> {
    pub event_mapping: HashMap<Event, Box<Action<S>>>,
}

impl<S> KeyBind<S> {
    pub fn assign<I>(&mut self, items: I)
    where
        I: IntoIterator<Item = (Event, Box<Action<S>>)>,
    {
        for (_, elem) in items.into_iter().enumerate() {
            self.event_mapping.insert(elem.0, elem.1);
        }
    }

    pub fn handle(
        &self,
        ev: &Event,
        out: &mut io::Stdout,
        context: &UpstreamContext,
        state: &mut S,
    ) -> Result<Option<String>> {
        match self.event_mapping.get(ev) {
            Some(handle) => handle(out, context, state),
            _ => Ok(None),
        }
    }
}

#[cfg(test)]
mod test {
    use std::any::Any;

    use super::{io, Action, HashMap, KeyBind, UpstreamContext};

    use crate::crossterm::event::{
        Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers,
    };

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
            Box::new(
                |_: &mut io::Stdout, _: &UpstreamContext, _: &mut Box<dyn Any>| {
                    Ok(Some(String::new()))
                },
            ) as Box<Action<Box<dyn Any>>>,
        )]);
        assert_eq!(b.event_mapping.len(), 1);
    }
}
