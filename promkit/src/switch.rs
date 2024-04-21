use std::collections::HashMap;

#[derive(Clone)]
pub struct ActiveKeySwitcher<S> {
    mapping: HashMap<String, S>,
    active_key: String,
}

impl<S> ActiveKeySwitcher<S> {
    pub fn new<K: AsRef<str>>(key: K, handler: S) -> Self {
        let key = key.as_ref().to_string();
        Self {
            mapping: HashMap::new(),
            active_key: key.clone(),
        }
        .register(key, handler)
    }

    pub fn register<K: AsRef<str>>(mut self, key: K, handler: S) -> Self {
        self.mapping.insert(key.as_ref().to_string(), handler);
        self
    }

    pub fn switch<K: AsRef<str>>(&mut self, key: K) {
        let key = key.as_ref().to_string();
        if self.mapping.contains_key(&key) {
            self.active_key = key;
        }
    }

    pub fn active_key(&self) -> &str {
        &self.active_key
    }

    pub fn get(&self) -> &S {
        self.mapping.get(&self.active_key).unwrap()
    }
}
