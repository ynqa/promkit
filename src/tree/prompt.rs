use std::cell::RefCell;

use std::rc::Rc;

use crate::{keybind::KeyBind, tree::State, Handler};

#[derive(Clone)]
pub struct Builder {
    _handler: Rc<RefCell<dyn Handler<State>>>,
    _folded_label: char,
    _unfolded_label: char,
}

impl Default for Builder {
    fn default() -> Self {
        Self {
            _handler: Rc::new(RefCell::new(KeyBind::default())),
            _folded_label: '▸',
            _unfolded_label: '▾',
        }
    }
}

impl Builder {
    pub fn folded_label(mut self, label: char) -> Self {
        self._folded_label = label;
        self
    }

    pub fn unfolded_label(mut self, label: char) -> Self {
        self._unfolded_label = label;
        self
    }
}
