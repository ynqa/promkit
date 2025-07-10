//! Contains a simple yes/no confirmation prompt.

use crate::Prompt;

use crate::preset::readline::{render, Readline};

/// A wrapper around `Readline` for creating simple yes/no confirmation prompts.
pub struct Confirm(Readline);

impl Confirm {
    /// Creates a new `Confirm` instance with a specified prompt text.
    /// The prompt text is formatted
    /// to include "(y/n)" to indicate the expected input.
    ///
    /// # Arguments
    ///
    /// * `text` - The text to display as part of the confirmation prompt.
    pub fn new<T: AsRef<str>>(text: T) -> Self {
        Self(
            Readline::default()
                .prefix(format!("{} (y/n) ", text.as_ref()))
                .validator(
                    |text| -> bool {
                        ["yes", "no", "y", "n", "Y", "N"]
                            .iter()
                            .any(|yn| *yn == text)
                    },
                    |_| String::from("Please type 'y' or 'n' as an answer"),
                ),
        )
    }

    /// Displays the confirmation prompt and waits for user input.
    /// Returns a `Result` containing the `Prompt` result,
    /// which is the user's input.
    pub fn prompt(self) -> anyhow::Result<Prompt<render::Renderer>> {
        self.0.prompt()
    }
}
