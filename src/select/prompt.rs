use std::cell::RefCell;
use std::io;
use std::rc::Rc;

use crossterm::style;

use crate::{
    build,
    edit::SelectBox,
    grapheme::Graphemes,
    keybind::KeyBind,
    select::{state::With, State},
    state::{self, Render},
    termutil, Handler, Prompt, Result,
};

#[derive(Clone)]
pub struct Builder {
    _handler: Rc<RefCell<dyn Handler<State>>>,
    _selectbox: Box<SelectBox>,
    _title: Option<Graphemes>,
    _title_color: Option<style::Color>,
    _selected_color: style::Color,
    _selected_item_prefix: Graphemes,
    _init_move_down_lines: u16,
    _window: Option<u16>,
    _suffix_after_trim: Graphemes,
}

impl Default for Builder {
    fn default() -> Self {
        Self {
            _handler: Rc::new(RefCell::new(KeyBind::default())),
            _selectbox: Box::new(SelectBox::default()),
            _title: None,
            _title_color: None,
            _selected_color: style::Color::Cyan,
            _selected_item_prefix: Graphemes::from("❯ "),
            _init_move_down_lines: 0,
            _window: None,
            _suffix_after_trim: Graphemes::from("…"),
        }
    }
}

impl build::Builder<SelectBox, With> for Builder {
    fn state(self) -> Result<Box<State>> {
        let tup = (self._selectbox.clone(), self._selectbox.clone());
        Ok(Box::new(state::State(
            state::Inherited {
                editor: self._selectbox,
                input_stream: vec![tup],
            },
            With {
                title: self._title,
                title_color: self._title_color,
                selected_cursor_pos: 0,
                selected_color: self._selected_color,
                selected_item_prefix: self._selected_item_prefix,
                init_move_down_lines: self._init_move_down_lines,
                window: self._window,
                suffix_after_trim: self._suffix_after_trim,
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
                |out: &mut io::Stdout, state: &mut State| -> Result<()> {
                    termutil::hide_cursor(out)?;
                    state.pre_render(out)
                },
            )),
            finalize: Some(Box::new(
                |out: &mut io::Stdout, _: &mut State| -> Result<()> {
                    termutil::show_cursor(out)?;
                    termutil::clear(out)
                },
            )),
            state: self.state()?,
        })
    }
}

impl Builder {
    pub fn selectbox(mut self, items: Box<SelectBox>) -> Self {
        self._selectbox = items;
        self
    }

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

    pub fn selected_color(mut self, color: style::Color) -> Self {
        self._selected_color = color;
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
