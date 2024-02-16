use crate::{
    crossterm::{
        event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers},
        style::{Attribute, Attributes, Color, ContentStyle},
    },
    error::Result,
    render::{Renderable, State},
    style::Style,
    text,
    text_editor::{self, History, Mode, Suggest},
    validate::Validator,
    Prompt,
};

mod confirm;
pub use confirm::Confirm;
mod password;
pub use password::Password;

pub struct Readline {
    title_renderer: text::Renderer,
    text_editor_renderer: text_editor::Renderer,
    error_message_renderer: text::Renderer,
    validator: Option<Validator<str>>,
}

impl Default for Readline {
    fn default() -> Self {
        Self {
            title_renderer: text::Renderer {
                text: Default::default(),
                style: Style::new()
                    .attrs(Attributes::from(Attribute::Bold))
                    .build(),
            },
            text_editor_renderer: text_editor::Renderer {
                texteditor: Default::default(),
                history: Default::default(),
                suggest: Default::default(),
                ps: String::from("❯❯ "),
                mask: Default::default(),
                ps_style: Style::new().fgc(Color::DarkGreen).build(),
                active_char_style: Style::new().bgc(Color::DarkCyan).build(),
                inactive_char_style: Style::new().build(),
                edit_mode: Default::default(),
                screen_lines: Default::default(),
            },
            error_message_renderer: text::Renderer {
                text: Default::default(),
                style: Style::new()
                    .fgc(Color::DarkRed)
                    .attrs(Attributes::from(Attribute::Bold))
                    .build(),
            },
            validator: Default::default(),
        }
    }
}

impl Readline {
    pub fn title<T: AsRef<str>>(mut self, text: T) -> Self {
        self.title_renderer.text = text.as_ref().to_string();
        self
    }

    pub fn title_style(mut self, style: ContentStyle) -> Self {
        self.title_renderer.style = style;
        self
    }

    pub fn enable_suggest(mut self, suggest: Suggest) -> Self {
        self.text_editor_renderer.suggest = suggest;
        self
    }

    pub fn enable_history(mut self) -> Self {
        self.text_editor_renderer.history = Some(History::default());
        self
    }

    pub fn prefix_string<T: AsRef<str>>(mut self, ps: T) -> Self {
        self.text_editor_renderer.ps = ps.as_ref().to_string();
        self
    }

    pub fn mask(mut self, mask: char) -> Self {
        self.text_editor_renderer.mask = Some(mask);
        self
    }

    pub fn prefix_string_style(mut self, style: ContentStyle) -> Self {
        self.text_editor_renderer.ps_style = style;
        self
    }

    pub fn active_char_style(mut self, style: ContentStyle) -> Self {
        self.text_editor_renderer.active_char_style = style;
        self
    }

    pub fn inactive_char_style(mut self, style: ContentStyle) -> Self {
        self.text_editor_renderer.inactive_char_style = style;
        self
    }

    pub fn edit_mode(mut self, mode: Mode) -> Self {
        self.text_editor_renderer.edit_mode = mode;
        self
    }

    pub fn screen_lines(mut self, screen_lines: usize) -> Self {
        self.text_editor_renderer.screen_lines = Some(screen_lines);
        self
    }

    pub fn validator<V, F>(mut self, validator: V, error_message_configure: F) -> Self
    where
        V: Fn(&str) -> bool + 'static,
        F: Fn(&str) -> String + 'static,
    {
        self.validator = Some(Validator::new(validator, error_message_configure));
        self
    }

    pub fn prompt(self) -> Result<Prompt<String>> {
        let validator = self.validator;

        Prompt::try_new(
            vec![
                Box::new(State::<text::Renderer>::new(self.title_renderer)),
                Box::new(State::<text_editor::Renderer>::new(
                    self.text_editor_renderer,
                )),
                Box::new(State::<text::Renderer>::new(self.error_message_renderer)),
            ],
            move |event: &Event,
                  renderables: &Vec<Box<dyn Renderable + 'static>>|
                  -> Result<bool> {
                let text: String = renderables[1]
                    .as_any()
                    .downcast_ref::<State<text_editor::Renderer>>()
                    .unwrap()
                    .after
                    .borrow()
                    .texteditor
                    .text_without_cursor();

                let error_message_state = renderables[2]
                    .as_any()
                    .downcast_ref::<State<text::Renderer>>()
                    .unwrap();

                let ret = match event {
                    Event::Key(KeyEvent {
                        code: KeyCode::Enter,
                        modifiers: KeyModifiers::NONE,
                        kind: KeyEventKind::Press,
                        state: KeyEventState::NONE,
                    }) => match &validator {
                        Some(validator) => {
                            let ret = validator.validate(&text);
                            if !validator.validate(&text) {
                                error_message_state.after.borrow_mut().text =
                                    validator.error_message(&text);
                            }
                            ret
                        }
                        None => true,
                    },
                    _ => true,
                };
                if ret {
                    *error_message_state.after.borrow_mut() = error_message_state.init.clone();
                }
                Ok(ret)
            },
            |renderables: &Vec<Box<dyn Renderable + 'static>>| -> Result<String> {
                Ok(renderables[1]
                    .as_any()
                    .downcast_ref::<State<text_editor::Renderer>>()
                    .unwrap()
                    .after
                    .borrow()
                    .texteditor
                    .text_without_cursor())
            },
        )
    }
}
