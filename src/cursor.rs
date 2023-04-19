use crate::Result;

#[derive(Default)]
pub struct Vertical {
    pub position: u16,
}

impl Vertical {
    pub fn move_up(&mut self) -> Result<()> {
        if self.position == 0 {
            self.position = 0;
        } else {
            self.position -= 1;
        }
        Ok(())
    }

    pub fn move_down(&mut self, screen_size: u16) -> Result<()> {
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

    pub fn move_head(&mut self) -> Result<()> {
        self.position = 0;
        Ok(())
    }

    pub fn move_tail(&mut self, screen_size: u16) -> Result<()> {
        self.position = screen_size - 1;
        Ok(())
    }
}
