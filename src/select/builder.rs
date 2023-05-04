use std::fmt;
use std::io;

use crate::{
    build,
    crossterm::style,
    grapheme::Graphemes,
    internal::selector::Selector,
    keybind::KeyBind,
    readline,
    register::Register,
    select::{EventHandler, Renderer, State, Store},
    text, Prompt, Result,
};

pub struct Builder {
    _keybind: KeyBind<State>,
    _selector: Selector,
    _title_store: Option<text::Store>,
    _query_builder: Option<readline::Builder>,
    _label: Graphemes,
    _label_color: style::Color,
    _window: Option<u16>,
    _suffix_after_trim: Graphemes,
}

impl Builder {
    pub fn new<I: fmt::Display, U: IntoIterator<Item = I>>(items: U) -> Self {
        let mut res = Self {
            _keybind: KeyBind::default(),
            _selector: Selector::default(),
            _title_store: None,
            _query_builder: None,
            _label: Graphemes::from("❯ "),
            _label_color: style::Color::Reset,
            _window: None,
            _suffix_after_trim: Graphemes::from("…"),
        };
        res._selector.register_all(items);
        res
    }
}

impl build::Builder for Builder {
    fn build(self) -> Result<Prompt> {
        Ok(Prompt {
            out: io::stdout(),
            ctr: Box::new(Store {
                state: State {
                    editor: self._selector.clone(),
                    screen_position: 0,
                    label: self._label,
                    label_color: self._label_color,
                    window: self._window,
                    suffix_after_trim: self._suffix_after_trim,
                },
                handler: EventHandler {
                    keybind: self._keybind,
                },
                renderer: Renderer {},
                title_store: self._title_store,
                query_store: self._query_builder.map(|b| b.store()),
            }),
        })
    }
}

impl Builder {
    pub fn keybind(mut self, keybind: KeyBind<State>) -> Self {
        self._keybind = keybind;
        self
    }

    pub fn title<T: fmt::Display>(mut self, title: T) -> Self {
        self._title_store = Some(text::Store {
            state: text::State {
                text: Graphemes::from(format!("{}", title)),
                ..Default::default()
            },
            renderer: text::Renderer {},
        });
        self
    }

    pub fn title_color(mut self, color: style::Color) -> Self {
        self._title_store.as_mut().map(|mut t| {
            t.state.text_color = color;
            t
        });
        self
    }

    pub fn query(mut self) -> Self {
        self._query_builder = Some(readline::Builder::default().num_lines(1));
        self
    }

    pub fn label<T: Into<String>>(mut self, label: T) -> Self {
        self._label = Graphemes::from(label.into());
        self
    }

    pub fn label_color(mut self, color: style::Color) -> Self {
        self._label_color = color;
        self
    }

    pub fn window(mut self, size: u16) -> Self {
        self._window = Some(size);
        self
    }

    pub fn suffix_after_trim<S: Into<String>>(mut self, suffix: S) -> Self {
        self._suffix_after_trim = Graphemes::from(suffix.into());
        self
    }
}
