use promkit::pane::Pane;

pub struct PaneMerger {
    pub panes: Vec<Pane>,
}

impl PaneMerger {
    pub fn new(init: Vec<Pane>) -> Self {
        PaneMerger { panes: init }
    }

    pub fn merge(&mut self, index: usize, pane: Pane) -> &Vec<Pane> {
        if index < self.panes.len() {
            self.panes[index] = pane;
        } else {
            self.panes.push(pane);
        }
        &self.panes
    }
}
