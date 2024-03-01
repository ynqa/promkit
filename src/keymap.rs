use std::collections::HashMap;

use crate::{
    crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers},
    Error, EventAction, EventHandler, Result,
};

fn minimum_keymap<S>(_: &mut S, event: &Event) -> Result<EventAction> {
    match event {
        Event::Key(KeyEvent {
            code: KeyCode::Enter,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => Ok(EventAction::Quit),
        Event::Key(KeyEvent {
            code: KeyCode::Char('c'),
            modifiers: KeyModifiers::CONTROL,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => Err(Error::Interrupted("ctrl+c".into())),
        _ => Ok(EventAction::Continue),
    }
}

#[derive(Clone)]
pub struct KeymapManager<S> {
    mapping: HashMap<String, EventHandler<S>>,
    current: String,
    default: EventHandler<S>,
}

impl<S> KeymapManager<S> {
    pub fn new<K: AsRef<str>>(key: K, handler: EventHandler<S>) -> Self {
        let key = key.as_ref().to_string();
        Self {
            mapping: HashMap::new(),
            current: key.clone(),
            default: minimum_keymap,
        }
        .register(key, handler)
    }

    pub fn register<K: AsRef<str>>(mut self, key: K, handler: EventHandler<S>) -> Self {
        self.mapping.insert(key.as_ref().to_string(), handler);
        self
    }

    pub fn switch<K: AsRef<str>>(&mut self, key: K) {
        let key = key.as_ref().to_string();
        if self.mapping.contains_key(&key) {
            self.current = key;
        }
    }

    pub fn get(&self) -> &EventHandler<S> {
        match self.mapping.get(&self.current) {
            Some(handler) => handler,
            None => &self.default,
        }
    }
}
