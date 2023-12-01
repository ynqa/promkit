use crate::{
    crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers},
    error::Result,
    theme::password::Theme,
    validate::Validator,
    view::{
        State, TextEditorViewer, TextEditorViewerBuilder, TextViewer, TextViewerBuilder, Viewable,
    },
    Prompt,
};

pub struct Password {
    title_builder: TextViewerBuilder,
    text_editor_builder: TextEditorViewerBuilder,
    validator: Option<Validator<str>>,
    error_message_builder: TextViewerBuilder,
}

impl Default for Password {
    fn default() -> Self {
        Self {
            title_builder: Default::default(),
            text_editor_builder: Default::default(),
            validator: Default::default(),
            error_message_builder: Default::default(),
        }
        .theme(Theme::default())
    }
}

impl Password {
    pub fn theme(mut self, theme: Theme) -> Self {
        self.title_builder = self.title_builder.style(theme.title_style);
        self.text_editor_builder = self
            .text_editor_builder
            .prefix(theme.prefix)
            .prefix_style(theme.prefix_style)
            .style(theme.text_style)
            .cursor_style(theme.cursor_style)
            .mask(theme.mask);
        self.error_message_builder = self.error_message_builder.style(theme.error_message_style);
        self
    }

    pub fn title<T: AsRef<str>>(mut self, text: T) -> Self {
        self.title_builder = self.title_builder.text(text);
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
                self.title_builder.build_state()?,
                self.text_editor_builder.build_state()?,
                self.error_message_builder.build_state()?,
            ],
            move |event: &Event, viewables: &Vec<Box<dyn Viewable + 'static>>| -> Result<bool> {
                let text: String = viewables[1]
                    .as_any()
                    .downcast_ref::<State<TextEditorViewer>>()
                    .unwrap()
                    .after
                    .borrow()
                    .textbuffer
                    .content_without_cursor();

                let error_message_state = viewables[2]
                    .as_any()
                    .downcast_ref::<State<TextViewer>>()
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
            |viewables: &Vec<Box<dyn Viewable + 'static>>| -> Result<String> {
                Ok(viewables[1]
                    .as_any()
                    .downcast_ref::<State<TextEditorViewer>>()
                    .unwrap()
                    .after
                    .borrow()
                    .textbuffer
                    .content_without_cursor())
            },
        )
    }
}
