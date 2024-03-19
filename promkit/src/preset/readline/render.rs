use crate::{
    impl_as_any, impl_cast, keymap::KeymapManager, listbox, pane::Pane, snapshot::Snapshot,
    suggest::Suggest, text, text_editor, validate::ValidatorManager,
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

impl_as_any!(Renderer);
impl_cast!(Renderer);

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
