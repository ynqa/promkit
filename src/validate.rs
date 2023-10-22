pub struct Validator<T: ?Sized> {
    validator: Box<dyn Fn(&T) -> bool>,
    error_message_builder: Box<dyn Fn(&T) -> String>,
}

impl<T: ?Sized> Validator<T> {
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

    pub fn validate(&self, input: &T) -> bool {
        (self.validator)(input)
    }

    pub fn error_message(&self, input: &T) -> String {
        (self.error_message_builder)(input)
    }
}
