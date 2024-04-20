use std::time::Duration;

use promkit::{
    crossterm::style::{Color, ContentStyle},
    style::StyleBuilder,
    switch::ActiveKeySwitcher,
    text_editor::{self, Mode},
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
    /// Sets the prefix string displayed before the input text.
    pub fn prefix<T: AsRef<str>>(mut self, prefix: T) -> Self {
        self.text_editor_state.prefix = prefix.as_ref().to_string();
        self
    }

    /// Sets the style for the prefix string.
    pub fn prefix_style(mut self, style: ContentStyle) -> Self {
        self.text_editor_state.prefix_style = style;
        self
    }

    /// Sets the style for the currently active character in the input field.
    pub fn active_char_style(mut self, style: ContentStyle) -> Self {
        self.text_editor_state.active_char_style = style;
        self
    }

    /// Sets the style for characters that are not currently active in the input field.
    pub fn inactive_char_style(mut self, style: ContentStyle) -> Self {
        self.text_editor_state.inactive_char_style = style;
        self
    }

    /// Sets the edit mode for the text editor, either insert or overwrite.
    pub fn edit_mode(mut self, mode: Mode) -> Self {
        self.text_editor_state.edit_mode = mode;
        self
    }

    /// Sets the number of lines available for rendering the text editor.
    pub fn text_editor_lines(mut self, lines: usize) -> Self {
        self.text_editor_state.lines = Some(lines);
        self
    }

    pub async fn run(self) -> anyhow::Result<String> {
        let (fin_sender, fin_receiver) = mpsc::channel(1);
        let (pane_sender, pane_receiver) = mpsc::channel(1);

        let renderer = render::Renderer::new(
            self.keymap,
            self.text_editor_state.clone(),
            self.text_editor_state.clone(),
            fin_sender,
            pane_sender,
        )?;

        let mut prompt = Prompt { renderer };

        prompt
            .run(
                Duration::from_millis(10),
                Duration::from_millis(10),
                Duration::from_millis(10),
                fin_receiver,
                pane_receiver,
            )
            .await
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("result: {:?}", Lazy::default().run().await?);
    Ok(())
}
