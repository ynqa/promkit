use std::cell::Cell;

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

    pub fn to_head(&self) {
        self.position.set(0);
    }

    pub fn move_to(&self, n: u16) {
        self.position.set(n);
    }
}
