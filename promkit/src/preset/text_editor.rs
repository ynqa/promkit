//! Offers functionality for editing multiline text.

use std::collections::HashSet;

use crate::{
    core::{
        crossterm::{
            event::Event,
            style::{Attribute, Attributes, Color, ContentStyle},
        },
        render::{Renderer, SharedRenderer},
        Widget,
    },
    preset::Evaluator,
    validate::{ErrorMessageGenerator, Validator, ValidatorManager},
    widgets::{
        text::{self, Text},
        text_editor::{self as text_editor_widget},
    },
    Signal,
};

pub mod evaluate;

/// Represents the indices of the multiline text editor components.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Index {
    Title = 0,
    Editor = 1,
    ErrorMessage = 2,
}

/// A prompt for editing newline-delimited text.
///
/// The default evaluator inserts a newline with <kbd>Enter</kbd> and submits
/// the complete buffer with <kbd>Ctrl+D</kbd>.
pub struct TextEditor {
    /// Shared renderer for the prompt.
    pub renderer: Option<SharedRenderer<Index>>,
    /// Function used to evaluate terminal events.
    pub evaluator: Evaluator<Self>,
    /// Title displayed above the editor.
    pub title: text::State,
    /// Multiline text editor state.
    pub editor: text_editor_widget::State,
    /// Cursor style restored whenever the prompt is reused.
    pub active_char_style: ContentStyle,
    /// Optional validator applied when the buffer is submitted.
    pub validator: Option<ValidatorManager<str>>,
    /// Validation error displayed below the editor.
    pub error_message: text::State,
}

impl Default for TextEditor {
    fn default() -> Self {
        let active_char_style = ContentStyle {
            background_color: Some(Color::DarkCyan),
            ..Default::default()
        };

        Self {
            renderer: None,
            evaluator: |event, ctx| Box::pin(evaluate::default(event, ctx)),
            title: text::State {
                config: text::config::Config {
                    style: Some(ContentStyle {
                        attributes: Attributes::from(Attribute::Bold),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                ..Default::default()
            },
            editor: text_editor_widget::State {
                texteditor: Default::default(),
                history: Default::default(),
                config: text_editor_widget::config::Config {
                    prefix: String::from("❯❯ "),
                    prefix_style: ContentStyle {
                        foreground_color: Some(Color::DarkGreen),
                        ..Default::default()
                    },
                    active_char_style,
                    inactive_char_style: ContentStyle::default(),
                    edit_mode: Default::default(),
                    word_break_chars: HashSet::from([' ', '\n']),
                    lines: Some(5),
                    ..Default::default()
                },
            },
            active_char_style,
            validator: None,
            error_message: text::State {
                text: Default::default(),
                config: text::config::Config {
                    style: Some(ContentStyle {
                        foreground_color: Some(Color::DarkRed),
                        attributes: Attributes::from(Attribute::Bold),
                        ..Default::default()
                    }),
                    lines: None,
                },
            },
        }
    }
}

#[async_trait::async_trait]
impl crate::Prompt for TextEditor {
    async fn initialize(&mut self) -> anyhow::Result<()> {
        self.renderer = Some(SharedRenderer::new(
            Renderer::try_new_with_graphemes(
                [
                    (Index::Title, self.title.create_graphemes()),
                    (Index::Editor, self.editor.create_graphemes()),
                    (Index::ErrorMessage, self.error_message.create_graphemes()),
                ],
                true,
            )
            .await?,
        ));
        Ok(())
    }

    async fn evaluate(&mut self, event: &Event) -> anyhow::Result<Signal> {
        let signal = (self.evaluator)(event, self).await;
        self.render().await?;
        signal
    }

    type Return = String;

    fn finalize(&mut self) -> anyhow::Result<Self::Return> {
        let text = self.editor.texteditor.text_without_cursor().to_string();
        self.editor.texteditor.erase_all();
        self.editor.config.active_char_style = self.active_char_style;
        Ok(text)
    }
}

impl TextEditor {
    /// Sets the title displayed above the editor.
    pub fn title<T: AsRef<str>>(mut self, text: T) -> Self {
        self.title.text = Text::from(text);
        self
    }

    /// Sets the title style.
    pub fn title_style(mut self, style: ContentStyle) -> Self {
        self.title.config.style = Some(style);
        self
    }

    /// Sets the prefix displayed before the first logical row.
    pub fn prefix<T: AsRef<str>>(mut self, prefix: T) -> Self {
        self.editor.config.prefix = prefix.as_ref().to_string();
        self
    }

    /// Sets the prefix style.
    pub fn prefix_style(mut self, style: ContentStyle) -> Self {
        self.editor.config.prefix_style = style;
        self
    }

    /// Sets the style of the grapheme at the cursor.
    pub fn active_char_style(mut self, style: ContentStyle) -> Self {
        self.editor.config.active_char_style = style;
        self.active_char_style = style;
        self
    }

    /// Sets the style of text outside the cursor.
    pub fn inactive_char_style(mut self, style: ContentStyle) -> Self {
        self.editor.config.inactive_char_style = style;
        self
    }

    /// Sets insert or overwrite editing mode.
    pub fn edit_mode(mut self, mode: text_editor_widget::Mode) -> Self {
        self.editor.config.edit_mode = mode;
        self
    }

    /// Sets characters used as word movement and deletion boundaries.
    pub fn word_break_chars(mut self, characters: HashSet<char>) -> Self {
        self.editor.config.word_break_chars = characters;
        self
    }

    /// Sets the maximum number of visible editor rows.
    pub fn lines(mut self, lines: usize) -> Self {
        self.editor.config.lines = Some(lines);
        self
    }

    /// Replaces the default event evaluator.
    pub fn evaluator(mut self, evaluator: Evaluator<Self>) -> Self {
        self.evaluator = evaluator;
        self
    }

    /// Configures validation performed when the buffer is submitted.
    pub fn validator(
        mut self,
        validator: Validator<str>,
        error_message_generator: ErrorMessageGenerator<str>,
    ) -> Self {
        self.validator = Some(ValidatorManager::new(validator, error_message_generator));
        self
    }

    async fn render(&mut self) -> anyhow::Result<()> {
        match self.renderer.as_ref() {
            Some(renderer) => {
                renderer
                    .update([
                        (Index::Title, self.title.create_graphemes()),
                        (Index::Editor, self.editor.create_graphemes()),
                        (Index::ErrorMessage, self.error_message.create_graphemes()),
                    ])
                    .render()
                    .await
            }
            None => Err(anyhow::anyhow!("Renderer not initialized")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{IndentContext, TextEditor};
    use crate::Prompt;

    fn lua_indent(context: IndentContext<'_>) -> String {
        if context.text.ends_with("then") {
            "    ".to_string()
        } else {
            String::new()
        }
    }

    #[test]
    fn builder_configures_the_editor() {
        let editor = TextEditor::default()
            .prefix("lua> ")
            .continuation_prefix("... ")
            .indenter(lua_indent)
            .lines(8);

        assert_eq!(editor.editor.config.prefix, "lua> ");
        assert_eq!(editor.editor.config.continuation_prefix, "... ");
        assert!(editor.indenter.is_some());
        assert_eq!(editor.editor.config.lines, Some(8));
    }

    #[test]
    fn finalize_returns_and_clears_the_buffer() {
        let mut editor = TextEditor::default();
        let active_char_style = editor.active_char_style;
        editor.editor.texteditor.replace("first\nsecond");
        editor.editor.config.active_char_style = Default::default();

        assert_eq!(editor.finalize().unwrap(), "first\nsecond");
        assert_eq!(
            editor.editor.texteditor.text_without_cursor().to_string(),
            ""
        );
        assert_eq!(editor.editor.config.active_char_style, active_char_style);
    }
}
