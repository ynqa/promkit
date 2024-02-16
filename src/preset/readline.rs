use crate::{
    crossterm::{
        event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers},
        style::{Attribute, Attributes, Color, ContentStyle},
    },
    error::Result,
    render::{Renderable, State},
    style::Style,
    text,
    text_editor::{self, History, Mode, Suggest, TextEditor},
    validate::Validator,
    Prompt,
};

mod password;
pub use password::Password;

pub struct Theme {
    /// Style for title (enabled if you set title).
    pub title_style: ContentStyle,
    /// Style for error message (enabled if you set error message).
    pub error_message_style: ContentStyle,

    /// Prompt string.
    pub ps: String,
    /// Style for prompt string.
    pub ps_style: ContentStyle,
    /// Style for selected character.
    pub active_char_style: ContentStyle,
    /// Style for un-selected character.
    pub inactive_item_style: ContentStyle,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            title_style: Style::new()
                .attrs(Attributes::from(Attribute::Bold))
                .build(),
            ps: String::from("❯❯ "),
            ps_style: Style::new().fgc(Color::DarkGreen).build(),
            inactive_item_style: Style::new().build(),
            active_char_style: Style::new().bgc(Color::DarkCyan).build(),
            error_message_style: Style::new()
                .fgc(Color::DarkRed)
                .attrs(Attributes::from(Attribute::Bold))
                .build(),
        }
    }
}

#[derive(Default)]
pub struct Readline {
    title: String,
    texteditor: TextEditor,
    error_message: String,
    validator: Option<Validator<str>>,
    theme: Theme,
    history: Option<History>,
    suggest: Suggest,
    mode: Mode,
    mask: Option<char>,
    window_size: Option<usize>,
}

impl Readline {
    pub fn title<T: AsRef<str>>(mut self, text: T) -> Self {
        self.title = text.as_ref().to_string();
        self
    }

    pub fn theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    pub fn enable_suggest(mut self, suggest: Suggest) -> Self {
        self.suggest = suggest;
        self
    }

    pub fn enable_history(mut self) -> Self {
        self.history = Some(History::default());
        self
    }

    pub fn edit_mode(mut self, mode: Mode) -> Self {
        self.mode = mode;
        self
    }

    pub fn mask(mut self, mask: char) -> Self {
        self.mask = Some(mask);
        self
    }

    pub fn window_size(mut self, window_size: usize) -> Self {
        self.window_size = Some(window_size);
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
                State::<text::Renderer>::try_new(self.title, self.theme.title_style)?,
                State::<text_editor::Renderer>::try_new(
                    self.texteditor,
                    self.history,
                    self.suggest,
                    self.theme.ps,
                    self.theme.ps_style,
                    self.theme.active_char_style,
                    self.theme.inactive_item_style,
                    self.mode,
                    self.mask,
                    self.window_size,
                )?,
                State::<text::Renderer>::try_new(
                    self.error_message,
                    self.theme.error_message_style,
                )?,
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
