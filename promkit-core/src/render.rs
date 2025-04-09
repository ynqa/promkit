use std::collections::BTreeMap;

use crate::{Pane, terminal::Terminal};

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

pub struct Renderer<K: Ord> {
    terminal: Terminal,
    panes: BTreeMap<K, Pane>,
}

impl<K: Ord> Renderer<K> {
    pub fn try_new() -> anyhow::Result<Self> {
        Ok(Self {
            terminal: Terminal {
                position: crossterm::cursor::position()?,
            },
            panes: BTreeMap::new(),
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

    pub fn render(&mut self) -> anyhow::Result<()> {
        self.terminal
            .draw(&self.panes.values().cloned().collect::<Vec<Pane>>())
    }
}
