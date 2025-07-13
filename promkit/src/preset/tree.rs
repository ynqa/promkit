//! Supports creating and interacting with a tree structure for hierarchical data.

use crate::{
    core::{
        crossterm::{
            self,
            event::Event,
            style::{Attribute, Attributes, Color, ContentStyle},
        },
        render::{Renderer, SharedRenderer},
        PaneFactory,
    },
    preset::Evaluator,
    widgets::{
        text::{self, Text},
        tree::{self, node::Node},
    },
    Signal,
};

pub mod evaluate;

/// Represents the indices of various components in the tree preset.
#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub enum Index {
    Title = 0,
    Tree = 1,
}

/// Represents a tree component for creating
/// and managing a hierarchical list of options.
pub struct Tree {
    /// Shared renderer for the prompt, allowing for rendering of UI components.
    pub renderer: Option<SharedRenderer<Index>>,
    /// Function to evaluate the input events and update the state of the prompt.
    pub evaluator: Evaluator<Self>,
    /// State for the title displayed above the tree.
    pub title: text::State,
    /// State for the tree itself.
    pub tree: tree::State,
}

#[async_trait::async_trait]
impl crate::Prompt for Tree {
    async fn initialize(&mut self) -> anyhow::Result<()> {
        let size = crossterm::terminal::size()?;
        self.renderer = Some(SharedRenderer::new(
            Renderer::try_new_with_panes(
                [
                    (Index::Title, self.title.create_pane(size.0, size.1)),
                    (Index::Tree, self.tree.create_pane(size.0, size.1)),
                ],
                true,
            )
            .await?,
        ));
        Ok(())
    }

    async fn evaluate(&mut self, event: &Event) -> anyhow::Result<Signal> {
        let ret = (self.evaluator)(event, self).await;
        let size = crossterm::terminal::size()?;
        self.renderer
            .as_ref()
            .unwrap()
            .update([
                (Index::Title, self.title.create_pane(size.0, size.1)),
                (Index::Tree, self.tree.create_pane(size.0, size.1)),
            ])
            .render()
            .await?;
        ret
    }

    type Return = Vec<String>;

    fn finalize(&mut self) -> anyhow::Result<Self::Return> {
        Ok(self.tree.tree.get())
    }
}

impl Tree {
    /// Creates a new `Tree` instance with the specified root node.
    pub fn new(root: Node) -> Self {
        Self {
            renderer: None,
            evaluator: |event, ctx| Box::pin(evaluate::default(event, ctx)),
            title: text::State {
                style: ContentStyle {
                    attributes: Attributes::from(Attribute::Bold),
                    ..Default::default()
                },
                ..Default::default()
            },
            tree: tree::State {
                tree: tree::Tree::new(root),
                folded_symbol: String::from("▶︎ "),
                unfolded_symbol: String::from("▼ "),
                active_item_style: ContentStyle {
                    foreground_color: Some(Color::DarkCyan),
                    ..Default::default()
                },
                inactive_item_style: ContentStyle::default(),
                lines: Default::default(),
                indent: 2,
            },
        }
    }

    /// Sets the title text displayed above the tree.
    pub fn title<T: AsRef<str>>(mut self, text: T) -> Self {
        self.title.text = Text::from(text);
        self
    }

    /// Sets the style for the title text.
    pub fn title_style(mut self, style: ContentStyle) -> Self {
        self.title.style = style;
        self
    }

    /// Sets the symbol used to indicate a folded (collapsed) node.
    pub fn folded_symbol<T: AsRef<str>>(mut self, symbol: T) -> Self {
        self.tree.folded_symbol = symbol.as_ref().to_string();
        self
    }

    /// Sets the symbol used to indicate an unfolded (expanded) node.
    pub fn unfolded_symbol<T: AsRef<str>>(mut self, symbol: T) -> Self {
        self.tree.unfolded_symbol = symbol.as_ref().to_string();
        self
    }

    /// Sets the style for active (currently selected) items.
    pub fn active_item_style(mut self, style: ContentStyle) -> Self {
        self.tree.active_item_style = style;
        self
    }

    /// Sets the style for inactive (not currently selected) items.
    pub fn inactive_item_style(mut self, style: ContentStyle) -> Self {
        self.tree.inactive_item_style = style;
        self
    }

    /// Sets the number of lines to be used for displaying the tree.
    pub fn tree_lines(mut self, lines: usize) -> Self {
        self.tree.lines = Some(lines);
        self
    }

    /// Sets the indentation level for rendering the tree data.
    pub fn indent(mut self, indent: usize) -> Self {
        self.tree.indent = indent;
        self
    }

    /// Sets the evaluator function for processing events in the tree.
    pub fn evaluator(mut self, evaluator: Evaluator<Self>) -> Self {
        self.evaluator = evaluator;
        self
    }
}
