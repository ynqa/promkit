use std::any::{type_name, Any};

use crate::{
    keymap::KeymapManager, listbox, pane::Pane, snapshot::Snapshot, suggest::Suggest, text,
    text_editor, validate::ValidatorManager, AsAny, Error, Result,
};

/// A `Renderer` for the readline preset, responsible for managing the rendering process.
/// It holds references to various components and their states, facilitating the rendering of the readline interface.
pub struct Renderer {
    /// Manages key bindings and their associated actions within the readline interface.
    pub keymap: KeymapManager<Self>,
    /// Holds a snapshot of the title's renderer state, used for rendering the title section.
    pub title_snapshot: Snapshot<text::Renderer>,
    /// Holds a snapshot of the text editor's renderer state, used for rendering the text input area.
    pub text_editor_snapshot: Snapshot<text_editor::Renderer>,
    /// Optional suggest component for autocomplete functionality.
    pub suggest: Option<Suggest>,
    /// Holds a snapshot of the suggest box's renderer state, used when rendering suggestions for autocomplete.
    pub suggest_snapshot: Snapshot<listbox::Renderer>,
    /// Optional validator manager for input validation.
    pub validator: Option<ValidatorManager<str>>,
    /// Holds a snapshot of the error message's renderer state, used for rendering error messages.
    pub error_message_snapshot: Snapshot<text::Renderer>,
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
        panes.extend(self.error_message_snapshot.create_panes(width));
        panes.extend(self.text_editor_snapshot.create_panes(width));
        panes.extend(self.suggest_snapshot.create_panes(width));
        panes
    }
}

impl AsAny for Renderer {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
