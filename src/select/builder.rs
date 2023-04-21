use std::fmt;
use std::io;

use crate::{
    build,
    crossterm::style,
    grapheme::Graphemes,
    internal::selector::Selector,
    keybind::KeyBind,
    register::Register,
    select::cursor::Cursor,
    select::{handler::Handler, State},
    termutil, text, Prompt, Result,
};

pub struct Builder {
    _keybind: KeyBind<State>,
    _selector: Selector,
    _title: Option<text::State>,
    _label: Graphemes,
    _label_color: style::Color,
    _init_move_down_lines: u16,
    _window: Option<u16>,
    _suffix_after_trim: Graphemes,
}

impl Builder {
    pub fn new<I: fmt::Display, U: IntoIterator<Item = I>>(items: U) -> Self {
        let mut res = Self {
            _keybind: KeyBind::default(),
            _selector: Selector::default(),
            _title: None,
            _label: Graphemes::from("❯ "),
            _label_color: style::Color::Reset,
            _init_move_down_lines: 0,
            _window: None,
            _suffix_after_trim: Graphemes::from("…"),
        };
        res._selector.register_all(items);
        res
    }
}

impl build::Builder<State, Handler<State>, Handler<State>> for Builder {
    fn build(self) -> Result<Prompt<State, Handler<State>, Handler<State>>> {
        Ok(Prompt::<State, Handler<State>, Handler<State>> {
            out: io::stdout(),
            keybind: self._keybind,
            input_handler: Handler::default(),
            resize_handler: Handler::default(),
            pre_run: Some(Box::new(
                |out: &mut io::Stdout, state: &mut State| -> Result<()> {
                    state.render(out)?;
                    state.prev = state.editor.clone();
                    Ok(())
                },
            )),
            post_run: Some(Box::new(
                |_: &mut io::Stdout, state: &mut State| -> Result<()> {
                    state.next = state.editor.clone();
                    Ok(())
                },
            )),
            initialize: Some(Box::new(
                |out: &mut io::Stdout, state: &mut State| -> Result<()> {
                    termutil::hide_cursor(out)?;
                    state.render_static(out)
                },
            )),
            finalize: Some(Box::new(
                |out: &mut io::Stdout, state: &mut State| -> Result<()> {
                    termutil::show_cursor(out)?;
                    termutil::move_down(
                        out,
                        termutil::num_lines(
                            &state.title.as_ref().unwrap_or(&text::State::default()).text,
                        )?,
                    )
                },
            )),
            state: State {
                editor: self._selector.clone(),
                prev: self._selector.clone(),
                next: self._selector.clone(),
                title: self._title,
                cursor: Cursor::default(),
                label: self._label,
                label_color: self._label_color,
                init_move_down_lines: self._init_move_down_lines,
                window: self._window,
                suffix_after_trim: self._suffix_after_trim,
            },
        })
    }
}

impl Builder {
    pub fn keybind(mut self, keybind: KeyBind<State>) -> Self {
        self._keybind = keybind;
        self
    }

    pub fn title<T: fmt::Display>(mut self, title: T) -> Self {
        self._title = Some(text::State {
            text: Graphemes::from(format!("{}", title)),
            ..Default::default()
        });
        self
    }

    pub fn title_color(mut self, color: style::Color) -> Self {
        self._title.as_mut().map(|mut t| {
            t.text_color = color;
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

    pub fn init_move_down_lines(mut self, lines: u16) -> Self {
        self._init_move_down_lines = lines;
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
