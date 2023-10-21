use crate::{
    components::{Component, State, Text, TextBuilder, TextEditor, TextEditorBuilder},
    crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers},
    error::Result,
    theme::confirm::Theme,
    validate::Validator,
    Prompt,
};

pub struct Confirm {
    text_editor: TextEditorBuilder,
    error_message: TextBuilder,
}

impl Confirm {
    pub fn new<T: AsRef<str>>(text: T) -> Self {
        Self {
            text_editor: TextEditorBuilder::default().prefix(format!("{} (y/n) ", text.as_ref())),
            error_message: Default::default(),
        }
        .theme(Theme::default())
    }

    pub fn theme(mut self, theme: Theme) -> Self {
        self.text_editor = self
            .text_editor
            .prefix_style(theme.prefix_style)
            .style(theme.text_style)
            .cursor_style(theme.cursor_style);
        self.error_message = self.error_message.style(theme.error_message_style);
        self
    }

    pub fn prompt(self) -> Result<Prompt<String>> {
        let validator = Validator::new(
            |text| -> bool {
                vec!["yes", "no", "y", "n", "Y", "N"]
                    .iter()
                    .any(|yn| yn == text)
            },
            |_| String::from("Please type 'y' or 'n' as an answer"),
        );

        Prompt::try_new(
            vec![
                self.text_editor.build_state()?,
                self.error_message.build_state()?,
            ],
            move |event: &Event, components: &Vec<Box<dyn Component + 'static>>| -> Result<bool> {
                let text: String = components[0]
                    .as_any()
                    .downcast_ref::<State<TextEditor>>()
                    .unwrap()
                    .after
                    .borrow()
                    .textbuffer
                    .content_without_cursor();

                let hint_state = components[1]
                    .as_any()
                    .downcast_ref::<State<Text>>()
                    .unwrap();

                let ret = match event {
                    Event::Key(KeyEvent {
                        code: KeyCode::Enter,
                        modifiers: KeyModifiers::NONE,
                        kind: KeyEventKind::Press,
                        state: KeyEventState::NONE,
                    }) => {
                        let ret = validator.validate(&text);
                        if !validator.validate(&text) {
                            hint_state.after.borrow_mut().text = validator.error_message(&text);
                        }
                        ret
                    }
                    _ => true,
                };
                if ret {
                    *hint_state.after.borrow_mut() = hint_state.init.clone();
                }
                Ok(ret)
            },
            |components: &Vec<Box<dyn Component + 'static>>| -> Result<String> {
                Ok(components[0]
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
