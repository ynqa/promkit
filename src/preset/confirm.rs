use crate::{
    crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers},
    error::Result,
    preset::theme::confirm::Theme,
    render::{Renderable, State},
    text::{Builder as TextRendererBuilder, Renderer as TextRenderer},
    text_editor::{
        Builder as TextEditorRendererBuilder, Renderer as TextEditorRenderer, TextEditor,
    },
    validate::Validator,
    Prompt,
};

pub struct Confirm {
    text_editor_builder: TextEditorRendererBuilder,
    error_message_builder: TextRendererBuilder,
}

impl Confirm {
    pub fn new<T: AsRef<str>>(text: T) -> Self {
        Self {
            text_editor_builder: TextEditorRendererBuilder::default()
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
            move |event: &Event,
                  renderables: &Vec<Box<dyn Renderable + 'static>>|
                  -> Result<bool> {
                let text_editor_state = renderables[0]
                    .as_any()
                    .downcast_ref::<State<TextEditorRenderer>>()
                    .unwrap();

                let text = text_editor_state
                    .after
                    .borrow()
                    .texteditor
                    .text_without_cursor();

                let error_message_state = renderables[1]
                    .as_any()
                    .downcast_ref::<State<TextRenderer>>()
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
                            text_editor_state.after.borrow_mut().texteditor = TextEditor::default();
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
            |renderables: &Vec<Box<dyn Renderable + 'static>>| -> Result<String> {
                Ok(renderables[0]
                    .as_any()
                    .downcast_ref::<State<TextEditorRenderer>>()
                    .unwrap()
                    .after
                    .borrow()
                    .texteditor
                    .text_without_cursor())
            },
        )
    }
}
