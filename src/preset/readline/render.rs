use std::any::{type_name, Any};

use crate::{
    keymap::KeymapManager, listbox, pane::Pane, snapshot::Snapshot, suggest::Suggest, text,
    text_editor, validate::ValidatorManager, AsAny, Error, Result,
};

pub struct Renderer {
    pub keymap: KeymapManager<Self>,
    pub title_snapshot: Snapshot<text::Renderer>,
    pub text_editor_snapshot: Snapshot<text_editor::Renderer>,
    pub suggest: Option<Suggest>,
    pub suggest_snapshot: Snapshot<listbox::Renderer>,
    pub validator: Option<ValidatorManager<str>>,
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
