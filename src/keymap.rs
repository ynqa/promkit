use std::collections::HashMap;

use crate::EventHandler;

/// Represents the mode of operation for key mappings.
///
/// Modes allow for different sets of keybindings based on the current context or state.
/// - `Default`: The default mode with a basic set of keybindings.
/// - `Type(String)`: A custom mode identified by a string, allowing for specialized keybindings.
#[derive(Clone, Hash, Eq, PartialEq)]
pub enum Mode {
    Default,
    Type(String),
}

/// Manages key mappings for different modes of operation.
///
/// This manager allows registering event handlers for specific modes and switching between these modes.
/// Each mode can have its own set of keybindings through associated event handlers.
#[derive(Clone)]
pub struct KeymapManager<S> {
    mapping: HashMap<Mode, EventHandler<S>>,
    current: Mode,
}

impl<S> KeymapManager<S> {
    /// Creates a new `KeymapManager` with a default event handler.
    ///
    /// The default handler is associated with the `Mode::Default`.
    ///
    /// # Arguments
    ///
    /// * `default` - The default event handler to be used.
    pub fn new(default: EventHandler<S>) -> Self {
        let mut mapping = HashMap::new();
        mapping.insert(Mode::Default, default);
        Self {
            mapping,
            current: Mode::Default,
        }
    }

    /// Registers an event handler for a specific mode.
    ///
    /// If the mode already exists, its handler is updated.
    ///
    /// # Arguments
    ///
    /// * `mode` - The mode for which the handler is being registered.
    /// * `handler` - The event handler to be associated with the mode.
    pub fn register(&mut self, mode: Mode, handler: EventHandler<S>) {
        self.mapping.insert(mode, handler);
    }

    /// Switches the current mode to the specified mode if it exists.
    ///
    /// If the specified mode does not exist, no action is taken.
    ///
    /// # Arguments
    ///
    /// * `mode` - The mode to switch to.
    pub fn switch(&mut self, mode: Mode) {
        if self.mapping.contains_key(&mode) {
            self.current = mode;
        }
    }

    /// Retrieves the current event handler based on the current mode.
    ///
    /// If the current mode has no associated handler, the default handler is returned.
    pub fn get(&self) -> &EventHandler<S> {
        self.mapping
            .get(&self.current)
            .unwrap_or(&self.mapping[&Mode::Default])
    }
}
