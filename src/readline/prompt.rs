use std::cell::RefCell;
use std::io;
use std::rc::Rc;

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
    termutil, Handler, Prompt, Result,
};

#[derive(Clone)]
pub struct Builder {
    _handler: Rc<RefCell<dyn Handler<State>>>,
    _title: Option<Graphemes>,
    _title_color: Option<style::Color>,
    _label: Graphemes,
    _label_color: style::Color,
    _mask: Option<Grapheme>,
    _edit_mode: Mode,
    _num_lines: Option<usize>,
    _min_len_to_search: usize,
    _limit_history_size: Option<usize>,
    _suggest: Option<Box<Suggest>>,
}

impl Default for Builder {
    fn default() -> Self {
        Self {
            _handler: Rc::new(RefCell::new(KeyBind::default())),
            _title: None,
            _title_color: None,
            _label: Graphemes::from("❯❯ "),
            _label_color: style::Color::DarkRed,
            _mask: None,
            _edit_mode: Mode::Insert,
            _num_lines: None,
            _min_len_to_search: 1,
            _limit_history_size: None,
            _suggest: None,
        }
    }
}

impl build::Builder<State> for Builder {
    fn build(self) -> Result<Prompt<State>> {
        Ok(Prompt::<State> {
            out: io::stdout(),
            handler: self.clone()._handler,
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
                |out: &mut io::Stdout, state: &mut State| -> Result<()> { state.pre_render(out) },
            )),
            finalize: Some(Box::new(
                |out: &mut io::Stdout, state: &mut State| -> Result<()> {
                    termutil::move_right(out, state.editor.width_from_position() as u16)?;
                    termutil::move_down(out)?;
                    termutil::move_head(out)?;
                    if let Some(hstr) = &mut state.hstr {
                        hstr.register(state.editor.data.clone());
                        // Oldest one of history is removed
                        // when the history is filled.
                        if let Some(limit) = state.limit_history_size {
                            // Plus 1 considers the current input.
                            if limit + 1 < hstr.data.len() {
                                hstr.data.remove(0);
                            }
                        }
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
                title_color: self._title_color,
                label: self._label,
                label_color: self._label_color,
                mask: self._mask,
                edit_mode: self._edit_mode,
                num_lines: self._num_lines,
                hstr: Some(Box::new(History::default())),
                min_len_to_search: self._min_len_to_search,
                limit_history_size: self._limit_history_size,
                suggest: self._suggest,
            },
        })
    }
}

impl Builder {
    pub fn handler<H: 'static + Handler<State>>(mut self, handler: H) -> Self {
        self._handler = Rc::new(RefCell::new(handler));
        self
    }

    pub fn title<T: Into<String>>(mut self, title: T) -> Self {
        self._title = Some(Graphemes::from(title.into()));
        self
    }

    pub fn title_color(mut self, color: style::Color) -> Self {
        self._title_color = Some(color);
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

    pub fn min_len_to_search(mut self, len: usize) -> Self {
        self._min_len_to_search = len;
        self
    }

    pub fn limit_history_size(mut self, size: usize) -> Self {
        self._limit_history_size = Some(size);
        self
    }

    pub fn suggest(mut self, suggest: Box<Suggest>) -> Self {
        self._suggest = Some(suggest);
        self
    }
}
