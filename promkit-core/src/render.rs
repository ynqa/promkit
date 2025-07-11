use std::sync::Arc;

use crossbeam_skiplist::SkipMap;
use tokio::sync::Mutex;

use crate::{Pane, terminal::Terminal};

/// SharedRenderer is a type alias for an Arc-wrapped Renderer, allowing for shared ownership and concurrency.
pub type SharedRenderer<K> = Arc<Renderer<K>>;

/// Renderer is responsible for managing and rendering multiple panes in a terminal.
pub struct Renderer<K: Ord + Send + 'static> {
    terminal: Mutex<Terminal>,
    panes: SkipMap<K, Pane>,
}

impl<K: Ord + Send + 'static> Renderer<K> {
    pub fn try_new() -> anyhow::Result<Self> {
        Ok(Self {
            terminal: Mutex::new(Terminal {
                position: crossterm::cursor::position()?,
            }),
            panes: SkipMap::new(),
        })
    }

    pub fn update<I>(&mut self, items: I) -> &mut Self
    where
        I: IntoIterator<Item = (K, Pane)>,
    {
        items.into_iter().for_each(|(index, pane)| {
            self.panes.insert(index, pane);
        });
        self
    }

    pub fn remove<I>(&mut self, items: I) -> &mut Self
    where
        I: IntoIterator<Item = K>,
    {
        items.into_iter().for_each(|index| {
            self.panes.remove(&index);
        });
        self
    }

    pub async fn render(&mut self) -> anyhow::Result<()> {
        let panes: Vec<Pane> = self
            .panes
            .iter()
            .map(|entry| entry.value().clone())
            .collect();
        let mut terminal = self.terminal.lock().await;
        terminal.draw(&panes)
    }
}
