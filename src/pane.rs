use crate::grapheme::Graphemes;

pub struct Pane {
    pub layout: Vec<Graphemes>,
    pub offset: usize,
}

impl Pane {
    pub fn extract(&self, viewport_height: usize) -> Vec<Graphemes> {
        if self.layout.len() <= viewport_height {
            return self.layout.clone();
        }
        let mut start = self.offset;
        let end = self.offset + viewport_height;
        if end > self.layout.len() {
            start = self.layout.len().saturating_sub(viewport_height);
        }

        return self
            .layout
            .iter()
            .enumerate()
            .filter(|(i, _)| start <= *i && *i < end)
            .map(|(_, row)| row.clone())
            .collect::<Vec<_>>();
    }
}
