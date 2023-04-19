use crate::Result;

#[derive(Default)]
pub struct Cursor {
    pub position: u16,
}

impl Cursor {
    pub fn prev(&mut self) -> Result<()> {
        if self.position == 0 {
            self.position = 0;
        } else {
            self.position -= 1;
        }
        Ok(())
    }

    pub fn next(&mut self, screen_size: u16) -> Result<()> {
        if screen_size > 0 {
            let limit = screen_size - 1;
            if self.position >= limit {
                self.position = limit;
            } else {
                self.position += 1;
            }
        }
        Ok(())
    }

    pub fn to_head(&mut self) -> Result<()> {
        self.position = 0;
        Ok(())
    }

    pub fn to_tail(&mut self, screen_size: u16) -> Result<()> {
        self.position = screen_size - 1;
        Ok(())
    }
}
