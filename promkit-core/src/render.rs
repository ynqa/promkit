use std::sync::Arc;

use crossbeam_skiplist::SkipMap;
use tokio::sync::Mutex;

use crate::{grapheme::StyledGraphemes, terminal::Terminal};

/// SharedRenderer is a type alias for an Arc-wrapped Renderer, allowing for shared ownership and concurrency.
pub type SharedRenderer<K> = Arc<Renderer<K>>;

/// Renderer is responsible for managing and rendering multiple grapheme chunks in a terminal.
pub struct Renderer<K: Ord + Send + 'static> {
    terminal: Mutex<Terminal>,
    graphemes: SkipMap<K, StyledGraphemes>,
}

impl<K: Ord + Send + 'static> Renderer<K> {
    pub fn try_new() -> anyhow::Result<Self> {
        Ok(Self {
            terminal: Mutex::new(Terminal {
                position: crossterm::cursor::position()?,
            }),
            graphemes: SkipMap::new(),
        })
    }

    pub async fn try_new_with_graphemes<I>(init: I, draw: bool) -> anyhow::Result<Self>
    where
        I: IntoIterator<Item = (K, StyledGraphemes)>,
    {
        let renderer = Self::try_new()?;
        renderer.update(init);
        if draw {
            renderer.render().await?;
        }
        Ok(renderer)
    }

    pub fn update<I>(&self, items: I) -> &Self
    where
        I: IntoIterator<Item = (K, StyledGraphemes)>,
    {
        items.into_iter().for_each(|(index, graphemes)| {
            self.graphemes.insert(index, graphemes);
        });
        self
    }

    pub fn remove<I>(&self, items: I) -> &Self
    where
        I: IntoIterator<Item = K>,
    {
        items.into_iter().for_each(|index| {
            self.graphemes.remove(&index);
        });
        self
    }

    // TODO: Implement diff rendering
    pub async fn render(&self) -> anyhow::Result<()> {
        let graphemes: Vec<StyledGraphemes> = self
            .graphemes
            .iter()
            .map(|entry| entry.value().clone())
            .collect();
        let mut terminal = self.terminal.lock().await;
        terminal.draw(&graphemes)
    }
}
