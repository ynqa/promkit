use std::any::{type_name, Any};

use crate::{
    checkbox, keymap::KeymapManager, pane::Pane, snapshot::Snapshot, text, AsAny, Error, Result,
};

pub struct Renderer {
    pub title_snapshot: Snapshot<text::Renderer>,
    pub checkbox_snapshot: Snapshot<checkbox::Renderer>,
    pub keymap: KeymapManager<Self>,
}

impl Renderer {
    pub fn cast_mut(renderer: &mut dyn crate::Renderer) -> Result<&mut Self> {
        let snapshot = renderer
            .as_any_mut()
            .downcast_mut::<Self>()
            .ok_or_else(|| Error::TypeCastError(type_name::<Self>().to_string()))?;
        Ok(snapshot)
    }

    pub fn cast(renderer: &dyn crate::Renderer) -> Result<&Self> {
        let snapshot = renderer
            .as_any()
            .downcast_ref::<Self>()
            .ok_or_else(|| Error::TypeCastError(type_name::<Self>().to_string()))?;
        Ok(snapshot)
    }
}

impl crate::Renderer for Renderer {
    fn create_panes(&self, width: u16) -> Vec<Pane> {
        let mut panes = Vec::new();
        panes.extend(self.title_snapshot.create_panes(width));
        panes.extend(self.checkbox_snapshot.create_panes(width));
        panes
    }

    fn postrun(&mut self) {}
}

impl AsAny for Renderer {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
