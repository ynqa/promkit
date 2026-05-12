/// A generic structure for validating inputs of any type.
///
/// This structure allows for the definition of custom validation logic
/// and error message generation for inputs of a specified type.
/// It encapsulates a validator function and an error message generator
/// function, both of which operate on references to the input.
pub struct ValidatorManager<T: ?Sized> {
    /// A function that takes a reference
    /// to an input of type `T` and returns a boolean
    /// indicating whether the input passes the validation.
    validator: Box<dyn Fn(&T) -> bool + Send + Sync>,
    /// A function that takes a reference
    /// to an input of type `T` and returns a `String`
    /// that describes the validation error.
    error_message_generator: Box<dyn Fn(&T) -> String + Send + Sync>,
}

impl<T: ?Sized> ValidatorManager<T> {
    /// Constructs a new `ValidatorManager` instance
    /// with the specified validator and error message generator functions.
    ///
    /// # Arguments
    ///
    /// * `validator` - A function or closure that takes a reference
    ///   to an input of type `T` and returns a boolean
    ///   indicating whether the input passes the validation.
    /// * `error_message_generator` - A function or closure that takes a reference
    ///   to an input of type `T` and returns a `String`
    ///   that describes the validation error.
    ///
    /// # Returns
    ///
    /// Returns a new instance of `ValidatorManager<T>`.
    pub fn new(
        validator: impl Fn(&T) -> bool + Send + Sync + 'static,
        error_message_generator: impl Fn(&T) -> String + Send + Sync + 'static,
    ) -> Self {
        Self {
            validator: Box::new(validator),
            error_message_generator: Box::new(error_message_generator),
        }
    }

    /// Validates the given input
    /// using the encapsulated validator function.
    ///
    /// # Arguments
    ///
    /// * `input` - A reference
    ///   to the input of type `T` to be validated.
    ///
    /// # Returns
    ///
    /// Returns `true` if the input passes the validation,
    /// otherwise `false`.
    pub fn validate(&self, input: &T) -> bool {
        (self.validator)(input)
    }

    /// Generates an error message for the given input
    /// using the encapsulated error message generator function.
    ///
    /// # Arguments
    ///
    /// * `input` - A reference to the input of type `T`
    ///   for which to generate an error message.
    ///
    /// # Returns
    ///
    /// Returns a `String` that describes the validation error.
    pub fn generate_error_message(&self, input: &T) -> String {
        (self.error_message_generator)(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[test]
    fn function_pointer_validator() {
        let vm = ValidatorManager::new(
            |text: &str| text.len() > 3,
            |text: &str| format!("Too short: {}", text.len()),
        );
        assert!(vm.validate("hello"));
        assert!(!vm.validate("hi"));
        assert_eq!(vm.generate_error_message("hi"), "Too short: 2");
    }

    #[test]
    fn closure_captures_owned_data() {
        let forbidden: Vec<String> = vec!["admin".into(), "root".into()];
        let vm = ValidatorManager::new(
            move |text: &str| !forbidden.contains(&text.to_string()),
            |text: &str| format!("'{}' is not allowed", text),
        );
        assert!(!vm.validate("admin"));
        assert!(vm.validate("user"));
    }

    #[test]
    fn closure_captures_shared_state() {
        let counter = Arc::new(Mutex::new(0u32));
        let counter_clone = Arc::clone(&counter);
        let vm = ValidatorManager::new(
            move |_text: &str| {
                let mut c = counter_clone.lock().unwrap();
                *c += 1;
                true
            },
            |_text: &str| String::new(),
        );
        vm.validate("a");
        vm.validate("b");
        assert_eq!(*counter.lock().unwrap(), 2);
    }
}
