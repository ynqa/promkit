use std::collections::HashMap;

use crate::EventHandler;

#[derive(Clone, Hash, Eq, PartialEq)]
pub enum Mode {
    Default,
    Type(String),
}

#[derive(Clone)]
pub struct KeymapManager<S> {
    mapping: HashMap<Mode, EventHandler<S>>,
    current: Mode,
}

impl<S> KeymapManager<S> {
    pub fn new(default: EventHandler<S>) -> Self {
        let mut mapping = HashMap::new();
        mapping.insert(Mode::Default, default);
        Self {
            mapping,
            current: Mode::Default,
        }
    }

    pub fn register(&mut self, mode: Mode, handler: EventHandler<S>) {
        self.mapping.insert(mode, handler);
    }

    pub fn switch(&mut self, mode: Mode) {
        if self.mapping.contains_key(&mode) {
            self.current = mode;
        }
    }

    pub fn get(&self) -> &EventHandler<S> {
        self.mapping
            .get(&self.current)
            .unwrap_or(&self.mapping[&Mode::Default])
    }
}
