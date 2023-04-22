use core::fmt;
use std::io;

use crate::{
    build,
    crossterm::style,
    grapheme::{Grapheme, Graphemes},
    internal::buffer::Buffer,
    internal::selector::history::History,
    keybind::KeyBind,
    readline::{state::State, Mode},
    register::Register,
    suggest::Suggest,
    termutil, text, Output, Prompt, Result,
};

pub struct Builder {
    _keybind: KeyBind<State>,
    _title: Option<text::State>,
    _label: Graphemes,
    _label_color: style::Color,
    _mask: Option<Grapheme>,
    _edit_mode: Mode,
    _num_lines: Option<usize>,
    _suggest: Option<Suggest>,
}

impl Default for Builder {
    fn default() -> Self {
        Self {
            _keybind: KeyBind::default(),
            _title: None,
            _label: Graphemes::from("❯❯ "),
            _label_color: style::Color::Reset,
            _mask: None,
            _edit_mode: Mode::Insert,
            _num_lines: None,
            _suggest: None,
        }
    }
}

impl build::Builder<State> for Builder {
    fn build(self) -> Result<Prompt<State>> {
        Ok(Prompt::<State> {
            out: io::stdout(),
            keybind: self._keybind,
            input_handler: Some(Box::new(
                |ch: char,
                 _out: &mut io::Stdout,
                 state: &mut State|
                 -> Result<Option<<State as Output>::Output>> {
                    if let Some(limit) = state.buffer_limit()? {
                        if limit <= state.editor.data.width() {
                            return Ok(None);
                        }
                    }
                    match state.edit_mode {
                        Mode::Insert => state.editor.insert(Grapheme::from(ch)),
                        Mode::Overwrite => state.editor.overwrite(Grapheme::from(ch)),
                    }
                    Ok(None)
                },
            )),
            resize_handler: Some(Box::new(
                |_: (u16, u16),
                 out: &mut io::Stdout,
                 state: &mut State|
                 -> Result<Option<<State as Output>::Output>> {
                    termutil::clear(out)?;
                    state.render_static(out)?;
                    // Overwrite the prev as default.
                    state.prev = Buffer::default();
                    Ok(None)
                },
            )),
            pre_run: Some(Box::new(
                |out: &mut io::Stdout, state: &mut State| -> Result<()> {
                    state.can_render()?;
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
                    state.render_static(out)
                },
            )),
            finalize: Some(Box::new(
                |out: &mut io::Stdout, state: &mut State| -> Result<()> {
                    termutil::move_right(out, state.editor.width_from_position() as u16)?;
                    termutil::move_down(out, 1)?;
                    termutil::move_head(out)?;
                    if let Some(hstr) = &mut state.hstr {
                        hstr.register(state.editor.data.clone());
                    }
                    state.editor = Buffer::default();
                    state.prev = Buffer::default();
                    state.next = Buffer::default();
                    Ok(())
                },
            )),
            state: State {
                editor: Buffer::default(),
                prev: Buffer::default(),
                next: Buffer::default(),
                title: self._title,
                label: self._label,
                label_color: self._label_color,
                mask: self._mask,
                edit_mode: self._edit_mode,
                num_lines: self._num_lines,
                hstr: Some(History::default()),
                suggest: self._suggest,
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

    pub fn mask(mut self, mask: char) -> Self {
        self._mask = Some(Grapheme::from(mask));
        self
    }

    pub fn edit_mode(mut self, mode: Mode) -> Self {
        self._edit_mode = mode;
        self
    }

    pub fn num_lines(mut self, lines: usize) -> Self {
        self._num_lines = Some(lines);
        self
    }

    pub fn suggest(mut self, suggest: Suggest) -> Self {
        self._suggest = Some(suggest);
        self
    }
}
