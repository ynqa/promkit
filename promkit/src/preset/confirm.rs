//! Contains a simple yes/no confirmation prompt.

use crate::Prompt;

use crate::preset::readline::Readline;

/// A wrapper around `Readline` for creating simple yes/no confirmation prompts.
pub struct Confirm(Readline);

impl Confirm {
    /// Creates a new `Confirm` instance with a default prompt.
    pub async fn try_default<T: AsRef<str>>(text: T) -> anyhow::Result<Self> {
        Ok(Self(
            Readline::try_default()
                .await?
                .prefix(format!("{} (y/n) ", text.as_ref()))
                .validator(
                    |text| -> bool {
                        ["yes", "no", "y", "n", "Y", "N"]
                            .iter()
                            .any(|yn| *yn == text)
                    },
                    |_| String::from("Please type 'y' or 'n' as an answer"),
                ),
        ))
    }

    /// Sets the title text displayed above the confirmation prompt.
    pub async fn run(&mut self) -> anyhow::Result<String> {
        self.0.run().await
    }
}
