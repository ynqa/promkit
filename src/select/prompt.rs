use std::cell::RefCell;
use std::io;
use std::rc::Rc;

use crate::{
    build,
    crossterm::style,
    grapheme::Graphemes,
    keybind::KeyBind,
    select::{state::With, State},
    selectbox::SelectBox,
    state::{self, Render},
    termutil, Handler, Prompt, Result,
};

#[derive(Clone)]
pub struct Builder {
    _handler: Rc<RefCell<dyn Handler<State>>>,
    _selectbox: Box<SelectBox>,
    _title: Option<Graphemes>,
    _title_color: Option<style::Color>,
    _label: Graphemes,
    _label_color: style::Color,
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
            _label: Graphemes::from("❯ "),
            _label_color: style::Color::Cyan,
            _init_move_down_lines: 0,
            _window: None,
            _suffix_after_trim: Graphemes::from("…"),
        }
    }
}

impl build::Builder<SelectBox, With> for Builder {
    fn state(self) -> Result<Box<State>> {
        Ok(Box::new(state::State(
            state::Inherited {
                editor: self._selectbox.clone(),
                prev: self._selectbox.clone(),
                next: self._selectbox.clone(),
            },
            With {
                title: self._title,
                title_color: self._title_color,
                selected_cursor_position: 0,
                label: self._label,
                label_color: self._label_color,
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
                |out: &mut io::Stdout, state: &mut State| -> Result<()> {
                    state.render(out)?;
                    state.0.prev = state.0.editor.clone();
                    Ok(())
                },
            )),
            post_run: Some(Box::new(
                |_: &mut io::Stdout, state: &mut State| -> Result<()> {
                    state.0.next = state.0.editor.clone();
                    Ok(())
                },
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
