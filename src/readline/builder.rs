use core::fmt;
use std::io;

use crate::{
    build,
    crossterm::style,
    grapheme::{Grapheme, Graphemes},
    grid::Grid,
    internal::buffer::Buffer,
    internal::selector::history::History,
    keybind::KeyBind,
    readline::{EventHandler, Mode, Renderer, State, Store},
    suggest::Suggest,
    text, Prompt, Result,
};

pub struct Builder {
    _keybind: KeyBind<State>,
    _title_store: Option<text::Store>,
    _label: Graphemes,
    _label_color: style::Color,
    _mask: Option<Grapheme>,
    _edit_mode: Mode,
    _num_lines: Option<u16>,
    _suggest: Option<Suggest>,
}

impl Default for Builder {
    fn default() -> Self {
        Self {
            _keybind: KeyBind::default(),
            _title_store: None,
            _label: Graphemes::from("❯❯ "),
            _label_color: style::Color::Reset,
            _mask: None,
            _edit_mode: Mode::Insert,
            _num_lines: None,
            _suggest: None,
        }
    }
}

impl build::Builder for Builder {
    fn build(self) -> Result<Prompt> {
        let mut g = Grid(vec![Box::new(Store {
            readline: State {
                editor: Buffer::default(),
                prev: Buffer::default(),
                next: Buffer::default(),
                label: self._label,
                label_color: self._label_color,
                mask: self._mask,
                edit_mode: self._edit_mode,
                num_lines: self._num_lines,
                hstr: Some(History::default()),
                suggest: self._suggest,
            },
            handler: EventHandler {
                keybind: self._keybind,
            },
            renderer: Renderer {},
        })]);
        if let Some(title_store) = self._title_store {
            g.insert(0, Box::new(title_store));
        }
        Ok(Prompt {
            out: io::stdout(),
            grid: g,
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

    pub fn label<T: Into<String>>(mut self, label: T) -> Self {
        self._label = Graphemes::from(label.into());
        self
    }

    pub fn label_color(mut self, color: style::Color) -> Self {
        self._label_color = color;
        self
    }

    pub fn mask(mut self, mask: char) -> Self {
        self._mask = Some(Grapheme::from(mask));
        self
    }

    pub fn edit_mode(mut self, mode: Mode) -> Self {
        self._edit_mode = mode;
        self
    }

    pub fn num_lines(mut self, lines: u16) -> Self {
        self._num_lines = Some(lines);
        self
    }

    pub fn suggest(mut self, suggest: Suggest) -> Self {
        self._suggest = Some(suggest);
        self
    }
}
