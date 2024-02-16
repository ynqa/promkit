use crate::{error::Result, Prompt};

use super::Readline;

pub struct Confirm(Readline);

impl Confirm {
    pub fn new<T: AsRef<str>>(text: T) -> Self {
        Self(
            Readline::default()
                .prefix_string(format!("{} (y/n) ", text.as_ref()))
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

    pub fn prompt(self) -> Result<Prompt<String>> {
        self.0.prompt()
    }
}
