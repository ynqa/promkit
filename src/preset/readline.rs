use crate::{
    crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers},
    error::Result,
    preset::theme::readline::Theme,
    validate::Validator,
    view::{
        Mode, State, Suggest, TextEditorViewer, TextEditorViewerBuilder, TextViewer,
        TextViewerBuilder, Viewable,
    },
    Prompt,
};

pub struct Readline {
    title_builder: TextViewerBuilder,
    text_editor_builder: TextEditorViewerBuilder,
    validator: Option<Validator<str>>,
    error_message_builder: TextViewerBuilder,
}

impl Default for Readline {
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

impl Readline {
    pub fn theme(mut self, theme: Theme) -> Self {
        self.title_builder = self.title_builder.style(theme.title_style);
        self.text_editor_builder = self
            .text_editor_builder
            .prefix(theme.prefix)
            .prefix_style(theme.prefix_style)
            .style(theme.text_style)
            .cursor_style(theme.cursor_style);
        self.error_message_builder = self.error_message_builder.style(theme.error_message_style);
        self
    }

    pub fn title<T: AsRef<str>>(mut self, text: T) -> Self {
        self.title_builder = self.title_builder.text(text);
        self
    }

    pub fn edit_mode(mut self, mode: Mode) -> Self {
        self.text_editor_builder = self.text_editor_builder.edit_mode(mode);
        self
    }

    pub fn lines(mut self, lines: usize) -> Self {
        self.text_editor_builder = self.text_editor_builder.lines(lines);
        self
    }

    pub fn suggest(mut self, suggest: Suggest) -> Self {
        self.text_editor_builder = self.text_editor_builder.suggest(suggest);
        self
    }

    pub fn enable_history(mut self) -> Self {
        self.text_editor_builder = self.text_editor_builder.enable_history();
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
                    .text
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
                    .text
                    .content_without_cursor())
            },
        )
    }
}
