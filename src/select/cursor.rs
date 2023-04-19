use std::cell::Cell;

use crate::Result;

#[derive(Default)]
pub struct Cursor {
    pub position: Cell<u16>,
}

impl Cursor {
    pub fn prev(&self) {
        if 0 < self.position.get() {
            self.position.set(self.position.get() - 1);
        }
    }

    pub fn next(&self, screen_size: u16) -> Result<()> {
        if screen_size > 0 {
            let limit = screen_size - 1;
            if self.position.get() >= limit {
                self.position.set(limit);
            } else {
                self.position.set(self.position.get() + 1);
            }
        }
        Ok(())
    }

    pub fn to_head(&self) {
        self.position.set(0);
    }

    pub fn to_tail(&self, screen_size: u16) {
        self.position.set(screen_size - 1);
    }
}
