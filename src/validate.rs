/// A generic structure for validating inputs of any type.
///
/// This structure allows for the definition of custom validation logic and error message generation
/// for inputs of a specified type. It encapsulates a validator function and an error message builder
/// function, both of which operate on references to the input.
pub struct Validator<T: ?Sized> {
    /// A boxed function that takes a reference to an input of type `T` and returns a boolean
    /// indicating whether the input passes the validation.
    validator: Box<dyn Fn(&T) -> bool>,
    /// A boxed function that takes a reference to an input of type `T` and returns a `String`
    /// that describes the validation error.
    error_message_builder: Box<dyn Fn(&T) -> String>,
}

impl<T: ?Sized> Validator<T> {
    /// Constructs a new `Validator` instance with the specified validator and error message builder functions.
    ///
    /// # Arguments
    ///
    /// * `validator` - A function that takes a reference to an input of type `T` and returns a boolean
    /// indicating whether the input passes the validation.
    /// * `error_message_builder` - A function that takes a reference to an input of type `T` and returns a `String`
    /// that describes the validation error.
    ///
    /// # Returns
    ///
    /// Returns a new instance of `Validator<T>`.
    pub fn new<V, S>(validator: V, error_message_builder: S) -> Self
    where
        V: Fn(&T) -> bool + 'static,
        S: Fn(&T) -> String + 'static,
    {
        Self {
            validator: Box::new(validator),
            error_message_builder: Box::new(error_message_builder),
        }
    }

    /// Validates the given input using the encapsulated validator function.
    ///
    /// # Arguments
    ///
    /// * `input` - A reference to the input of type `T` to be validated.
    ///
    /// # Returns
    ///
    /// Returns `true` if the input passes the validation, otherwise `false`.
    pub fn validate(&self, input: &T) -> bool {
        (self.validator)(input)
    }

    /// Generates an error message for the given input using the encapsulated error message builder function.
    ///
    /// # Arguments
    ///
    /// * `input` - A reference to the input of type `T` for which to generate an error message.
    ///
    /// # Returns
    ///
    /// Returns a `String` that describes the validation error.
    pub fn error_message(&self, input: &T) -> String {
        (self.error_message_builder)(input)
    }
}
