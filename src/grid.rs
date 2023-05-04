use std::io;
use std::ops::{Deref, DerefMut};

use crate::{
    crossterm::{event::Event, terminal},
    termutil, Controller, Result, UpstreamContext,
};

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
            upstream_used_rows += d.used_rows(&UpstreamContext {
                unused_rows: terminal::size()?.1 - upstream_used_rows,
            })?;
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
        for d in self.iter() {
            d.render_static(out)?;
        }
        Ok(())
    }

    fn handle_resize(&mut self, out: &mut io::Stdout) -> Result<()> {
        termutil::clear(out)?;
        for d in self.iter_mut() {
            d.run_on_resize()?;
        }
        self.can_render()?;
        self.render_static(out)
    }

    pub fn handle_event(&mut self, ev: &Event, out: &mut io::Stdout) -> Result<Option<String>> {
        if let Event::Resize(_, _) = ev {
            self.handle_resize(out)?;
        } else {
            let mut upstream_used_rows = 0;
            for d in self.iter_mut() {
                let context = UpstreamContext {
                    unused_rows: terminal::size()?.1 - upstream_used_rows,
                };
                if let Some(ret) = d.handle_event(ev, out, &context)? {
                    return Ok(Some(ret));
                }
                upstream_used_rows += d.used_rows(&context)?;
            }
        }
        Ok(None)
    }

    pub fn render(&mut self, out: &mut io::Stdout) -> Result<()> {
        let mut upstream_used_rows = 0;
        for d in self.iter_mut() {
            let context = UpstreamContext {
                unused_rows: terminal::size()?.1 - upstream_used_rows,
            };
            d.render(out, &context)?;
            upstream_used_rows += d.used_rows(&context)?;
        }
        Ok(())
    }

    pub fn finalize(&mut self, out: &mut io::Stdout) -> Result<()> {
        for d in self.iter_mut() {
            d.finalize(out)?;
        }
        Ok(())
    }
}
