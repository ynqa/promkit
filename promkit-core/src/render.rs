use std::sync::Arc;

use crossbeam_skiplist::SkipMap;
use tokio::sync::Mutex;

use crate::{Pane, terminal::Terminal};

/// OrderedIndex is used to represent the index of pane in a fraction format.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct OrderedIndex(pub usize, pub usize); // numerator, denominator

impl std::fmt::Display for OrderedIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({},{})", self.0, self.1)
    }
}

impl PartialOrd for OrderedIndex {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for OrderedIndex {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Comparing fractions: To compare a/b and c/d, compare ad and bc
        let left = (self.0 as u64) * (other.1 as u64);
        let right = (self.1 as u64) * (other.0 as u64);
        left.cmp(&right)
    }
}

impl OrderedIndex {
    pub fn mediant(a: &OrderedIndex, b: &OrderedIndex) -> Self {
        // TODO: gcd to reduce the fraction
        Self(a.0 + b.0, a.1 + b.1)
    }
}

/// SharedRenderer is a type alias for an Arc-wrapped Renderer, allowing for shared ownership and concurrency.
pub type SharedRenderer<K> = Arc<Renderer<K>>;

/// Content represents the content of a pane, including its visibility status.
pub struct Content {
    pane: Pane,
    visible: bool,
}

/// Renderer is responsible for managing and rendering multiple panes in a terminal.
pub struct Renderer<K: Ord + Send + 'static> {
    terminal: Mutex<Terminal>,
    contents: SkipMap<K, Content>,
}

impl<K: Ord + Send + 'static> Renderer<K> {
    pub fn try_new() -> anyhow::Result<Self> {
        Ok(Self {
            terminal: Mutex::new(Terminal {
                position: crossterm::cursor::position()?,
            }),
            contents: SkipMap::new(),
        })
    }

    pub fn update<I>(&mut self, items: I) -> &mut Self
    where
        I: IntoIterator<Item = (K, Content)>,
    {
        items.into_iter().for_each(|(index, content)| {
            self.contents.insert(index, content);
        });
        self
    }

    pub fn remove<I>(&mut self, items: I) -> &mut Self
    where
        I: IntoIterator<Item = K>,
    {
        items.into_iter().for_each(|index| {
            self.contents.remove(&index);
        });
        self
    }

    pub async fn render(&mut self) -> anyhow::Result<()> {
        let panes: Vec<Pane> = self
            .contents
            .iter()
            .filter(|entry| entry.value().visible)
            .map(|entry| entry.value().pane.clone())
            .collect();
        let mut terminal = self.terminal.lock().await;
        terminal.draw(&panes)
    }
}
