use std::io;
use std::ops::{Deref, DerefMut};

use crate::{crossterm::terminal, Controller, Result};

pub struct Grid(pub Vec<Box<dyn Controller>>);

impl Deref for Grid {
    type Target = Vec<Box<dyn Controller>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Grid {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Grid {
    pub fn can_render(&self) -> Result<()> {
        let mut upstream_used_rows = 0;
        self.iter().try_for_each(|d| {
            upstream_used_rows += d.used_rows(upstream_used_rows)?;
            if terminal::size()?.1 < upstream_used_rows {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "Terminal does not leave the space to render.",
                ));
            }
            Ok(())
        })
    }

    pub fn render_static(&self, out: &mut io::Stdout) -> Result<()> {
        let mut upstream_used_rows = 0;
        for d in self.iter() {
            d.render_static(out, terminal::size()?.1 - upstream_used_rows)?;
            upstream_used_rows += d.used_rows(upstream_used_rows)?;
        }
        Ok(())
    }

    pub fn handle_event(
        &mut self,
        ev: &crossterm::event::Event,
        out: &mut io::Stdout,
    ) -> Result<Option<String>> {
        let mut upstream_used_rows = 0;
        for d in self.iter_mut() {
            if let Some(ret) = d.handle_event(ev, out, terminal::size()?.1 - upstream_used_rows)? {
                return Ok(Some(ret));
            }
            upstream_used_rows += d.used_rows(upstream_used_rows)?;
        }
        Ok(None)
    }

    pub fn render(&mut self, out: &mut io::Stdout) -> Result<()> {
        let mut upstream_used_rows = 0;
        for d in self.iter_mut() {
            d.render(out, terminal::size()?.1 - upstream_used_rows)?;
            upstream_used_rows += d.used_rows(upstream_used_rows)?;
        }
        Ok(())
    }

    pub fn finalize(&mut self, out: &mut io::Stdout) -> Result<()> {
        let mut upstream_used_rows = 0;
        for d in self.iter_mut() {
            d.finalize(out)?;
            upstream_used_rows += d.used_rows(upstream_used_rows)?;
        }
        Ok(())
    }
}
