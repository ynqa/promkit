use std::any::{type_name, Any};

use crate::{
    keymap::KeymapManager, listbox, pane::Pane, snapshot::Snapshot, text, text_editor, AsAny,
    Error, Result,
};

/// Represents a renderer for the query selector.
/// This struct manages the rendering process of different components within the query selector,
/// including key mappings, title, text editor, and listbox.
pub struct Renderer {
    /// Manages key mappings specific to this renderer.
    pub keymap: KeymapManager<Self>,
    /// Snapshot of the title renderer.
    pub title_snapshot: Snapshot<text::Renderer>,
    /// Snapshot of the text editor renderer.
    pub text_editor_snapshot: Snapshot<text_editor::Renderer>,
    /// Snapshot of the listbox renderer.
    pub listbox_snapshot: Snapshot<listbox::Renderer>,
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
        panes.extend(self.text_editor_snapshot.create_panes(width));
        panes.extend(self.listbox_snapshot.create_panes(width));
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
