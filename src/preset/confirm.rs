use anyhow::{Ok, Result};

use crate::{
    components::{TextEditorBuilder, Widget},
    crossterm::event::Event,
    Prompt, PromptBuilder, validate::Validator,
};

use super::ReadlineBuilder;

pub struct Confirm {
    readline: ReadlineBuilder,
}

impl Confirm {
    pub fn new<T: AsRef<str>>(text: T) -> Self {
        Self {
            readline: ReadlineBuilder::default(),
            text_editor: TextEditorBuilder::default()
                .label(format!("{} (y/n) ", text.as_ref().to_string())),
            validator: Validator::new(
                |text| -> bool {
                    vec!["yes", "no", "y", "n", "Y", "N"].iter().any(|yn| yn == text)
                },
                |_, builder| {
                    builder.text("Please type 'y' or 'n' as an answer")
                }
            )
        }
    }

    pub fn prompt(self) -> Result<Prompt> {
        
    }
}
