use crate::{
    crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers},
    error::Result,
    theme::password::Theme,
    validate::Validator,
    viewer::{State, Text, TextBuilder, TextEditor, TextEditorBuilder, Viewable},
    Prompt,
};

pub struct Password {
    title: TextBuilder,
    text_editor: TextEditorBuilder,
    validator: Option<Validator<str>>,
    error_message: TextBuilder,
}

impl Default for Password {
    fn default() -> Self {
        Self {
            title: Default::default(),
            text_editor: Default::default(),
            validator: Default::default(),
            error_message: Default::default(),
        }
        .theme(Theme::default())
    }
}

impl Password {
    pub fn theme(mut self, theme: Theme) -> Self {
        self.title = self.title.style(theme.title_style);
        self.text_editor = self
            .text_editor
            .prefix(theme.prefix)
            .prefix_style(theme.prefix_style)
            .style(theme.text_style)
            .cursor_style(theme.cursor_style)
            .mask(theme.mask);
        self.error_message = self.error_message.style(theme.error_message_style);
        self
    }

    pub fn title<T: AsRef<str>>(mut self, text: T) -> Self {
        self.title = self.title.text(text);
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
                self.title.build_state()?,
                self.text_editor.build_state()?,
                self.error_message.build_state()?,
            ],
            move |event: &Event, viewables: &Vec<Box<dyn Viewable + 'static>>| -> Result<bool> {
                let text: String = viewables[1]
                    .as_any()
                    .downcast_ref::<State<TextEditor>>()
                    .unwrap()
                    .after
                    .borrow()
                    .textbuffer
                    .content_without_cursor();

                let error_message_state =
                    viewables[2].as_any().downcast_ref::<State<Text>>().unwrap();

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
            |viewables: &Vec<Box<dyn Viewable + 'static>>| -> Result<String> {
                Ok(viewables[1]
                    .as_any()
                    .downcast_ref::<State<TextEditor>>()
                    .unwrap()
                    .after
                    .borrow()
                    .textbuffer
                    .content_without_cursor())
            },
        )
    }
}
