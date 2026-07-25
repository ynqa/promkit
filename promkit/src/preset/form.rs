//! Provides multiple readline input options.

use crate::{
    core::{
        crossterm::{
            event::Event,
            style::{Attribute, Attributes, ContentStyle},
        },
        render::{Renderer, SharedRenderer},
        Widget,
    },
    preset::Evaluator,
    widgets::text_editor,
    Signal,
};

mod evaluate;

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
    pub evaluator: Evaluator<Self>,
    /// State for the multiple text editor components.
    pub readlines: Vec<text_editor::State>,
    /// Index of the focused text editor, or `None` when the form is empty.
    active: Option<usize>,
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

        self.renderer = Some(SharedRenderer::new(
            Renderer::try_new_with_graphemes(
                self.readlines
                    .iter()
                    .enumerate()
                    .map(|(i, state)| (i, state.create_graphemes())),
                true,
            )
            .await?,
        ));
        Ok(())
    }

    async fn evaluate(&mut self, event: &Event) -> anyhow::Result<Signal> {
        let ret = (self.evaluator)(event, self).await;

        // Update the styles based on the current position.
        self.overwrite_styles();

        self.render().await?;
        ret
    }

    type Return = Vec<String>;

    fn finalize(&mut self) -> anyhow::Result<Self::Return> {
        Ok(self
            .readlines
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
                        prefix_style: state.config.prefix_style,
                        active_char_style: state.config.active_char_style,
                        inactive_char_style: state.config.inactive_char_style,
                    };

                    let unfocus_style = Style {
                        prefix_style: ContentStyle {
                            attributes: Attributes::from(Attribute::Dim),
                            ..state.config.prefix_style
                        },
                        active_char_style: ContentStyle {
                            attributes: Attributes::from(Attribute::Dim),
                            ..Default::default()
                        },
                        inactive_char_style: ContentStyle {
                            attributes: Attributes::from(Attribute::Dim),
                            ..state.config.inactive_char_style
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
            evaluator: |event, ctx| Box::pin(evaluate::default(event, ctx)),
            active: (!readlines.is_empty()).then_some(0),
            readlines,
            focus_styles,
            unfocus_styles,
        }
    }

    /// Returns the focused text editor index, or `None` when the form is empty.
    pub fn active(&self) -> Option<usize> {
        self.active
            .filter(|position| *position < self.readlines.len())
    }

    /// Moves focus to the previous text editor, if possible.
    pub fn focus_previous(&mut self) -> bool {
        let Some(position) = self.active().filter(|position| *position > 0) else {
            return false;
        };
        self.active = Some(position - 1);
        true
    }

    /// Moves focus to the next text editor, if possible.
    pub fn focus_next(&mut self) -> bool {
        let Some(position) = self
            .active()
            .filter(|position| position.saturating_add(1) < self.readlines.len())
        else {
            return false;
        };
        self.active = Some(position + 1);
        true
    }

    /// Moves focus to a text editor by index.
    pub fn focus(&mut self, position: usize) -> bool {
        if position < self.readlines.len() {
            self.active = Some(position);
            true
        } else {
            false
        }
    }

    /// Render the prompt with the specified width and height.
    async fn render(&mut self) -> anyhow::Result<()> {
        match self.renderer.as_ref() {
            Some(renderer) => {
                renderer
                    .update(
                        self.readlines
                            .iter()
                            .enumerate()
                            .map(|(i, state)| (i, state.create_graphemes())),
                    )
                    .render()
                    .await
            }
            None => Err(anyhow::anyhow!("Renderer not initialized")),
        }
    }

    /// Updates the styles of text editor states based on their active or inactive status.
    fn overwrite_styles(&mut self) {
        let current_position = self.active();
        self.readlines
            .iter_mut()
            .enumerate()
            .for_each(|(i, state)| {
                if Some(i) == current_position {
                    state.config.prefix_style = self.focus_styles[i].prefix_style;
                    state.config.inactive_char_style = self.focus_styles[i].inactive_char_style;
                    state.config.active_char_style = self.focus_styles[i].active_char_style;
                } else {
                    state.config.prefix_style = self.unfocus_styles[i].prefix_style;
                    state.config.inactive_char_style = self.unfocus_styles[i].inactive_char_style;
                    state.config.active_char_style = self.unfocus_styles[i].active_char_style;
                }
            });
    }
}

#[cfg(test)]
mod tests {
    use super::Form;
    use crate::widgets::text_editor;

    #[test]
    fn empty_form_has_no_active_field() {
        let mut form = Form::new(Vec::<text_editor::State>::new());

        assert_eq!(form.active(), None);
        assert!(!form.focus_previous());
        assert!(!form.focus_next());
        assert!(!form.focus(0));
    }

    #[test]
    fn focus_navigation_stops_at_form_boundaries() {
        let mut form = Form::new([text_editor::State::default(), text_editor::State::default()]);

        assert_eq!(form.active(), Some(0));
        assert!(!form.focus_previous());
        assert!(form.focus_next());
        assert_eq!(form.active(), Some(1));
        assert!(!form.focus_next());
        assert!(!form.focus(2));
        assert_eq!(form.active(), Some(1));
        assert!(form.focus(0));
        assert_eq!(form.active(), Some(0));
    }
}
