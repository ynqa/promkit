use std::collections::BTreeMap;

use crate::{Pane, terminal::Terminal};

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
