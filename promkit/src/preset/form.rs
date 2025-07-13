//! Provides multiple readline input options.

use crate::{
    core::{
        crossterm::{
            self,
            event::Event,
            style::{Attribute, Attributes, ContentStyle},
        },
        render::{Renderer, SharedRenderer},
        PaneFactory,
    },
    widgets::{cursor::Cursor, text_editor},
    Signal,
};

mod evaluate;

/// Represents the indices of various components in the form preset.
pub type Evaluator = fn(event: &Event, ctx: &mut Form) -> anyhow::Result<Signal>;

/// Represents the visual styles for different states of text editor components.
pub struct Style {
    /// Style for the prefix of the text editor.
    pub prefix_style: ContentStyle,
    /// Style for the character that is currently active (e.g., where the cursor is).
    pub active_char_style: ContentStyle,
    /// Style for characters that are not currently active.
    pub inactive_char_style: ContentStyle,
}

/// `Form` struct provides functionality for managing multiple text input fields.
pub struct Form {
    /// Shared renderer for the prompt, allowing for rendering of UI components.
    pub renderer: Option<SharedRenderer<usize>>,
    /// Function to evaluate the input events and update the state of the prompt.
    pub evaluator_fn: Evaluator,
    /// State for the multiple text editor components.
    pub readlines: Cursor<Vec<text_editor::State>>,
    /// Default styles applied to text editors.
    pub focus_styles: Vec<Style>,
    /// Styles applied to text editors when they are unselected.
    pub unfocus_styles: Vec<Style>,
}

#[async_trait::async_trait]
impl crate::Prompt for Form {
    async fn initialize(&mut self) -> anyhow::Result<()> {
        // Update styles based on the current position.
        self.overwrite_styles();

        let size = crossterm::terminal::size()?;
        self.renderer = Some(SharedRenderer::new(
            Renderer::try_new_with_panes(
                self.readlines
                    .contents()
                    .iter()
                    .enumerate()
                    .map(|(i, state)| (i, state.create_pane(size.0, size.1))),
                true,
            )
            .await?,
        ));
        Ok(())
    }

    async fn evaluate(&mut self, event: &Event) -> anyhow::Result<Signal> {
        let ret = (self.evaluator_fn)(event, self);

        // Update the styles based on the current position.
        self.overwrite_styles();

        let size = crossterm::terminal::size()?;
        self.renderer
            .as_ref()
            .unwrap()
            .update(
                self.readlines
                    .contents()
                    .iter()
                    .enumerate()
                    .map(|(i, state)| (i, state.create_pane(size.0, size.1))),
            )
            .render()
            .await?;
        ret
    }

    type Return = Vec<String>;

    fn finalize(&mut self) -> anyhow::Result<Self::Return> {
        Ok(self
            .readlines
            .contents()
            .iter()
            .map(|state| state.texteditor.text_without_cursor().to_string())
            .collect())
    }
}

impl Form {
    pub fn new<I: IntoIterator<Item = text_editor::State>>(states: I) -> Self {
        let (readlines, focus_styles, unfocus_styles): (Vec<_>, Vec<_>, Vec<_>) =
            states.into_iter().fold(
                (Vec::new(), Vec::new(), Vec::new()),
                |(mut readlines, mut focus_styles, mut unfocus_styles), state| {
                    let focus_style = Style {
                        prefix_style: state.prefix_style,
                        active_char_style: state.active_char_style,
                        inactive_char_style: state.inactive_char_style,
                    };

                    let unfocus_style = Style {
                        prefix_style: ContentStyle {
                            attributes: Attributes::from(Attribute::Dim),
                            ..state.prefix_style
                        },
                        active_char_style: ContentStyle {
                            attributes: Attributes::from(Attribute::Dim),
                            ..Default::default()
                        },
                        inactive_char_style: ContentStyle {
                            attributes: Attributes::from(Attribute::Dim),
                            ..state.inactive_char_style
                        },
                    };

                    // Push the state and styles into the respective vectors.
                    readlines.push(state);
                    focus_styles.push(focus_style);
                    unfocus_styles.push(unfocus_style);

                    (readlines, focus_styles, unfocus_styles)
                },
            );

        Self {
            renderer: None,
            evaluator_fn: evaluate::default,
            readlines: Cursor::new(readlines, 0, false),
            focus_styles,
            unfocus_styles,
        }
    }

    /// Updates the styles of text editor states based on their active or inactive status.
    fn overwrite_styles(&mut self) {
        let current_position = self.readlines.position();
        self.readlines
            .contents_mut()
            .iter_mut()
            .enumerate()
            .for_each(|(i, state)| {
                if i == current_position {
                    state.prefix_style = self.focus_styles[i].prefix_style;
                    state.inactive_char_style = self.focus_styles[i].inactive_char_style;
                    state.active_char_style = self.focus_styles[i].active_char_style;
                } else {
                    state.prefix_style = self.unfocus_styles[i].prefix_style;
                    state.inactive_char_style = self.unfocus_styles[i].inactive_char_style;
                    state.active_char_style = self.unfocus_styles[i].active_char_style;
                }
            });
    }
}
