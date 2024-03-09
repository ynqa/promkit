use std::collections::HashMap;

use crate::{
    crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers},
    Error, EventAction, EventHandler, Result,
};

/// Defines a minimal keymap for handling basic events such as quitting the application
/// or interrupting with ctrl+c. It matches specific key events and returns an appropriate
/// `EventAction` or an `Error`.
///
/// # Arguments
///
/// * `_`: A mutable reference to a generic state `S`, not used in this function.
/// * `event`: A reference to the `Event` to be handled.
///
/// # Returns
///
/// This function returns a `Result<EventAction>` indicating the action to be taken in response
/// to the event, or an `Error` if the event indicates an interruption.
fn minimum_keymap<S>(_: &mut S, event: &Event) -> Result<EventAction> {
    match event {
        Event::Key(KeyEvent {
            code: KeyCode::Enter,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => Ok(EventAction::Quit),
        Event::Key(KeyEvent {
            code: KeyCode::Char('c'),
            modifiers: KeyModifiers::CONTROL,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => Err(Error::Interrupted("ctrl+c".into())),
        _ => Ok(EventAction::Continue),
    }
}

/// `KeymapManager` manages a collection of key-event handlers, allowing for dynamic switching
/// and retrieval of event handlers based on a string key. It supports registering new key-handler
/// pairs, switching the current active handler, and retrieving the current handler for event processing.
/// A default handler can be specified for use when no specific handler is available for the current key.
///
/// # Type Parameters
///
/// * `S`: The type of the state that the event handlers will operate on.
///
/// # Fields
///
/// * `mapping`: A `HashMap` that associates string keys with their corresponding `EventHandler`.
/// * `active_key`: A `String` representing the key of the currently active event handler.
/// * `default`: An `EventHandler` that is used as a fallback when no specific handler is found for the current key.
#[derive(Clone)]
pub struct KeymapManager<S> {
    mapping: HashMap<String, EventHandler<S>>,
    active_key: String,
    default: EventHandler<S>,
}

impl<S> KeymapManager<S> {
    /// Creates a new `KeymapManager` with a specified initial key and handler.
    ///
    /// # Arguments
    ///
    /// * `key`: A string slice that references the initial key to be used.
    /// * `handler`: An `EventHandler` associated with the initial key.
    ///
    /// # Returns
    ///
    /// Returns an instance of `KeymapManager`.
    pub fn new<K: AsRef<str>>(key: K, handler: EventHandler<S>) -> Self {
        let key = key.as_ref().to_string();
        Self {
            mapping: HashMap::new(),
            active_key: key.clone(),
            default: minimum_keymap,
        }
        .register(key, handler)
    }

    /// Registers a new key and its associated handler to the keymap.
    ///
    /// # Arguments
    ///
    /// * `key`: A string slice that references the key to be registered.
    /// * `handler`: An `EventHandler` to be associated with the key.
    ///
    /// # Returns
    ///
    /// Returns the `KeymapManager` instance to allow for method chaining.
    pub fn register<K: AsRef<str>>(mut self, key: K, handler: EventHandler<S>) -> Self {
        self.mapping.insert(key.as_ref().to_string(), handler);
        self
    }

    /// Switches the current key to a new key if it exists in the keymap.
    ///
    /// # Arguments
    ///
    /// * `key`: A string slice that references the new key to switch to.
    pub fn switch<K: AsRef<str>>(&mut self, key: K) {
        let key = key.as_ref().to_string();
        if self.mapping.contains_key(&key) {
            self.active_key = key;
        }
    }

    /// Returns a reference to the string representing the key of the currently active event handler.
    ///
    /// This method allows for querying which event handler is currently active by returning the key
    /// associated with it. This can be useful for debugging or for logic that needs to know which
    /// handler is currently in use.
    pub fn active_key(&self) -> &str {
        &self.active_key
    }

    /// Retrieves the current `EventHandler` based on the current key.
    ///
    /// # Returns
    ///
    /// Returns a reference to the current `EventHandler`. If the current key does not have an
    /// associated handler, the default handler is returned.
    pub fn get(&self) -> &EventHandler<S> {
        match self.mapping.get(&self.active_key) {
            Some(handler) => handler,
            None => &self.default,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod register {
        use super::*;

        #[test]
        fn test() {
            let mut manager =
                KeymapManager::<()>::new("default", |_: &mut (), _| Ok(EventAction::Continue))
                    .register("key2", |_: &mut (), _| Ok(EventAction::Continue));
            manager.switch("key2");
            assert_eq!("key2", manager.active_key)
        }
    }
}
