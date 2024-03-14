use super::Len;

#[derive(Clone)]
pub struct CompositeCursor<C> {
    bundle: Vec<C>,
    cross_contents_position: usize,
}

impl<C: Len> CompositeCursor<C> {
    pub fn new<I: IntoIterator<Item = C>>(bundle_iter: I, position: usize) -> Self {
        let bundle: Vec<C> = bundle_iter.into_iter().collect();
        let total_len: usize = bundle.iter().map(|c| c.len()).sum();
        let adjusted_position = if position >= total_len {
            total_len.saturating_sub(1)
        } else {
            position
        };

        Self {
            bundle,
            cross_contents_position: adjusted_position,
        }
    }

    pub fn bundle(&self) -> &Vec<C> {
        &self.bundle
    }

    pub fn cross_contents_position(&self) -> usize {
        self.cross_contents_position
    }

    pub fn current_bundle_index_and_inner_position(&self) -> (usize, usize) {
        let mut accumulated_len = 0;
        for (bundle_index, c) in self.bundle.iter().enumerate() {
            let c_len = c.len();
            if accumulated_len + c_len > self.cross_contents_position {
                return (bundle_index, self.cross_contents_position - accumulated_len);
            }
            accumulated_len += c_len;
        }
        (self.bundle.len() - 1, self.bundle.last().unwrap().len() - 1)
    }

    pub fn forward(&mut self) -> bool {
        let total_len: usize = self.bundle.iter().map(|c| c.len()).sum();
        if self.cross_contents_position < total_len.saturating_sub(1) {
            self.cross_contents_position += 1;
            true
        } else {
            false
        }
    }

    pub fn backward(&mut self) -> bool {
        if self.cross_contents_position > 0 {
            self.cross_contents_position = self.cross_contents_position.saturating_sub(1);
            true
        } else {
            false
        }
    }

    /// Moves the cursor to the head (start) of the bundle.
    pub fn move_to_head(&mut self) {
        self.cross_contents_position = 0
    }

    /// Moves the cursor to the tail (end) of the bundle.
    pub fn move_to_tail(&mut self) {
        let total_len: usize = self.bundle.iter().map(|c| c.len()).sum();
        if total_len == 0 {
            self.cross_contents_position = 0
        } else {
            self.cross_contents_position = total_len.saturating_sub(1);
        }
    }
}
