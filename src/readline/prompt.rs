use std::cell::RefCell;
use std::io;
use std::rc::Rc;

use crossterm::style;

use crate::{
    build,
    edit::{Buffer, History, Register, Suggest},
    grapheme::{Grapheme, Graphemes},
    keybind::KeyBind,
    readline::{state::With, Mode, State},
    state::{self, Render},
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
            _suggest: None,
        }
    }
}

impl build::Builder<Buffer, With> for Builder {
    fn state(self) -> Result<Box<State>> {
        Ok(Box::new(state::State(
            state::Inherited {
                editor: Box::new(Buffer::default()),
                input_stream: vec![(Box::new(Buffer::default()), Box::new(Buffer::default()))],
            },
            With {
                title: self._title,
                title_color: self._title_color,
                label: self._label,
                label_color: self._label_color,
                mask: self._mask,
                edit_mode: self._edit_mode,
                num_lines: self._num_lines,
                hstr: Some(Box::new(History::default())),
                suggest: self._suggest,
            },
        )))
    }

    fn build(self) -> Result<Prompt<State>> {
        Ok(Prompt::<State> {
            out: io::stdout(),
            handler: self.clone()._handler,
            pre_run: Some(Box::new(
                |out: &mut io::Stdout, state: &mut State| -> Result<()> { state.render(out) },
            )),
            initialize: Some(Box::new(
                |out: &mut io::Stdout, state: &mut State| -> Result<()> { state.pre_render(out) },
            )),
            finalize: Some(Box::new(
                |out: &mut io::Stdout, state: &mut State| -> Result<()> {
                    termutil::move_right(out, state.0.editor.width_from_pos() as u16)?;
                    termutil::move_down(out)?;
                    termutil::move_head(out)?;
                    if let Some(hstr) = &mut state.1.hstr {
                        hstr.register(state.0.editor.data.clone());
                    }
                    state.0.editor = Box::new(Buffer::default());
                    Ok(())
                },
            )),
            state: self.state()?,
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

    pub fn suggest(mut self, suggest: Box<Suggest>) -> Self {
        self._suggest = Some(suggest);
        self
    }
}
