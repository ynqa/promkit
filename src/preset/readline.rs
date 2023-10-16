use anyhow::Result;

use crate::{
    crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers},
    validate::Validator,
    widgets::{State, Text, TextBuilder, TextEditor, TextEditorBuilder, Widget},
    Prompt, PromptBuilder,
};

#[derive(Default)]
pub struct ReadlineBuilder {
    title: TextBuilder,
    text_editor: TextEditorBuilder,
    validator: Option<Validator<String>>,
}

impl ReadlineBuilder {
    pub fn title<F: Fn(TextBuilder) -> TextBuilder>(mut self, configure: F) -> Self {
        self.title = configure(self.title);
        self
    }

    pub fn text_editor<F: Fn(TextEditorBuilder) -> TextEditorBuilder>(
        mut self,
        configure: F,
    ) -> Self {
        self.text_editor = configure(self.text_editor);
        self
    }

    pub fn validator<V, F>(mut self, validator: V, error_message_configure: F) -> Self
    where
        V: Fn(&String) -> bool + 'static,
        F: Fn(&String, TextBuilder) -> TextBuilder + 'static,
    {
        self.validator = Some(Validator::new(validator, error_message_configure));
        self
    }

    pub fn build(self) -> Result<Prompt> {
        let validator = self.validator;

        PromptBuilder::new(vec![
            self.title.build_state()?,
            self.text_editor.build_state()?,
            TextBuilder::default().build_state()?,
        ])
        .evaluate(
            move |event: &Event, widgets: &Vec<Box<dyn Widget + 'static>>| -> Result<bool> {
                let text: String = widgets[1]
                    .as_any()
                    .downcast_ref::<State<TextEditor>>()
                    .unwrap()
                    .after
                    .borrow()
                    .textbuffer
                    .content_without_cursor();

                let hint_state = widgets[2].as_any().downcast_ref::<State<Text>>().unwrap();

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
                                let builder =
                                    validator.error_message_builder(&text, TextBuilder::default());
                                *hint_state.after.borrow_mut() = builder.build()?;
                            }
                            ret
                        }
                        None => true,
                    },
                    _ => true,
                };
                let hint_state = widgets[2].as_any().downcast_ref::<State<Text>>().unwrap();
                if ret {
                    *hint_state.after.borrow_mut() = hint_state.init.clone();
                }
                Ok(ret)
            },
        )
        .build()
    }
}
