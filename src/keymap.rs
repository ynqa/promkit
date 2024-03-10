use std::collections::HashMap;

use crate::EventHandler;

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
}

impl<S> KeymapManager<S> {
    pub fn new<K: AsRef<str>>(key: K, handler: EventHandler<S>) -> Self {
        let key = key.as_ref().to_string();
        Self {
            mapping: HashMap::new(),
            active_key: key.clone(),
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

    pub fn get(&self) -> Option<&EventHandler<S>> {
        self.mapping.get(&self.active_key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod register {
        use super::*;
        use crate::PromptSignal;

        #[test]
        fn test() {
            let mut manager =
                KeymapManager::<()>::new("default", |_: &mut (), _| Ok(PromptSignal::Continue))
                    .register("key2", |_: &mut (), _| Ok(PromptSignal::Continue));
            manager.switch("key2");
            assert_eq!("key2", manager.active_key)
        }
    }
}
