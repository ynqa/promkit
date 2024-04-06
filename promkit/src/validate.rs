pub type Validator<T> = fn(&T) -> bool;
pub type ErrorMessageGenerator<T> = fn(&T) -> String;

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
    validator: Validator<T>,
    /// A function that takes a reference
    /// to an input of type `T` and returns a `String`
    /// that describes the validation error.
    error_message_generator: ErrorMessageGenerator<T>,
}

impl<T: ?Sized> ValidatorManager<T> {
    /// Constructs a new `Validator` instance
    /// with the specified validator and error message generator functions.
    ///
    /// # Arguments
    ///
    /// * `validator` - A function that takes a reference
    /// to an input of type `T` and returns a boolean
    /// indicating whether the input passes the validation.
    /// * `error_message_generator` - A function that takes a reference
    /// to an input of type `T` and returns a `String`
    /// that describes the validation error.
    ///
    /// # Returns
    ///
    /// Returns a new instance of `Validator<T>`.
    pub fn new(validator: Validator<T>, error_message_generator: ErrorMessageGenerator<T>) -> Self {
        Self {
            validator,
            error_message_generator,
        }
    }

    /// Validates the given input
    /// using the encapsulated validator function.
    ///
    /// # Arguments
    ///
    /// * `input` - A reference
    /// to the input of type `T` to be validated.
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
    /// for which to generate an error message.
    ///
    /// # Returns
    ///
    /// Returns a `String` that describes the validation error.
    pub fn generate_error_message(&self, input: &T) -> String {
        (self.error_message_generator)(input)
    }
}
