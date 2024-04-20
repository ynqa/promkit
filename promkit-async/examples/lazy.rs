use std::time::Duration;

use promkit::{
    crossterm::style::Color,
    style::StyleBuilder,
    switch::ActiveKeySwitcher,
    text_editor::{self},
};
use promkit_async::Prompt;

use tokio::sync::mpsc;

mod lazyutil;
use lazyutil::{keymap, render};

pub struct Lazy {
    keymap: ActiveKeySwitcher<keymap::Handler>,
    text_editor_state: text_editor::State,
}

impl Default for Lazy {
    fn default() -> Self {
        Self {
            keymap: ActiveKeySwitcher::new("default", self::keymap::default),
            text_editor_state: text_editor::State {
                texteditor: Default::default(),
                history: Default::default(),
                prefix: String::from("❯❯ "),
                mask: Default::default(),
                prefix_style: StyleBuilder::new().fgc(Color::DarkGreen).build(),
                active_char_style: StyleBuilder::new().bgc(Color::DarkCyan).build(),
                inactive_char_style: StyleBuilder::new().build(),
                edit_mode: Default::default(),
                word_break_chars: Default::default(),
                lines: Default::default(),
            },
        }
    }
}

impl Lazy {
    pub async fn run(self) -> anyhow::Result<String> {
        let (fin_sender, fin_receiver) = mpsc::channel(1);
        let (versioned_each_pane_sender, versioned_each_pane_receiver) = mpsc::channel(1);
        let (versioned_loading_indicator_sender, versioned_loading_indicator_receiver) =
            mpsc::channel(1);

        let renderer = render::Renderer::new(
            self.keymap,
            self.text_editor_state.clone(),
            self.text_editor_state.clone(),
            fin_sender,
            versioned_each_pane_sender,
            versioned_loading_indicator_sender,
        )?;

        let mut prompt = Prompt { renderer };

        prompt
            .run(
                Duration::from_millis(10),
                Duration::from_millis(10),
                Duration::from_millis(10),
                fin_receiver,
                versioned_each_pane_receiver,
                versioned_loading_indicator_receiver,
            )
            .await
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("result: {:?}", Lazy::default().run().await?);
    Ok(())
}
