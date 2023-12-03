use crate::{
    crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers},
    error::Result,
    preset::theme::confirm::Theme,
    text::Text,
    validate::Validator,
    view::{
        State, TextEditorViewer, TextEditorViewerBuilder, TextViewer, TextViewerBuilder, Viewable,
    },
    Prompt,
};

pub struct Confirm {
    text_editor_builder: TextEditorViewerBuilder,
    error_message_builder: TextViewerBuilder,
}

impl Confirm {
    pub fn new<T: AsRef<str>>(text: T) -> Self {
        Self {
            text_editor_builder: TextEditorViewerBuilder::default()
                .prefix(format!("{} (y/n) ", text.as_ref())),
            error_message_builder: Default::default(),
        }
        .theme(Theme::default())
    }

    pub fn theme(mut self, theme: Theme) -> Self {
        self.text_editor_builder = self
            .text_editor_builder
            .prefix_style(theme.prefix_style)
            .style(theme.text_style)
            .cursor_style(theme.cursor_style);
        self.error_message_builder = self.error_message_builder.style(theme.error_message_style);
        self
    }

    pub fn prompt(self) -> Result<Prompt<String>> {
        let validator = Validator::new(
            |text| -> bool {
                ["yes", "no", "y", "n", "Y", "N"]
                    .iter()
                    .any(|yn| yn == text)
            },
            |_| String::from("Please type 'y' or 'n' as an answer"),
        );

        Prompt::try_new(
            vec![
                self.text_editor_builder.build_state()?,
                self.error_message_builder.build_state()?,
            ],
            move |event: &Event, viewables: &Vec<Box<dyn Viewable + 'static>>| -> Result<bool> {
                let text_editor_state = viewables[0]
                    .as_any()
                    .downcast_ref::<State<TextEditorViewer>>()
                    .unwrap();

                let text = text_editor_state
                    .after
                    .borrow()
                    .text
                    .content_without_cursor();

                let error_message_state = viewables[1]
                    .as_any()
                    .downcast_ref::<State<TextViewer>>()
                    .unwrap();

                let ret = match event {
                    Event::Key(KeyEvent {
                        code: KeyCode::Enter,
                        modifiers: KeyModifiers::NONE,
                        kind: KeyEventKind::Press,
                        state: KeyEventState::NONE,
                    }) => {
                        let ret = validator.validate(&text);
                        if !ret {
                            error_message_state.after.borrow_mut().text =
                                validator.error_message(&text);
                            text_editor_state.after.borrow_mut().text = Text::default();
                        }
                        ret
                    }
                    _ => true,
                };
                if ret {
                    *error_message_state.after.borrow_mut() = error_message_state.init.clone();
                }
                Ok(ret)
            },
            |viewables: &Vec<Box<dyn Viewable + 'static>>| -> Result<String> {
                Ok(viewables[0]
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
