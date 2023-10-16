use crate::widgets::TextBuilder;

type ErrorMessageBuilder<T> = dyn Fn(&T, TextBuilder) -> TextBuilder;

pub struct Validator<T> {
    validator: Box<dyn Fn(&T) -> bool>,
    error_message_builder: Box<ErrorMessageBuilder<T>>,
}

impl<T> Validator<T> {
    pub fn new<V, S>(validator: V, error_message_builder: S) -> Self
    where
        V: Fn(&T) -> bool + 'static,
        S: Fn(&T, TextBuilder) -> TextBuilder + 'static,
    {
        Self {
            validator: Box::new(validator),
            error_message_builder: Box::new(error_message_builder),
        }
    }

    pub fn validate(&self, input: &T) -> bool {
        (self.validator)(input)
    }

    pub fn error_message_builder(&self, input: &T, text_builder: TextBuilder) -> TextBuilder {
        (self.error_message_builder)(input, text_builder)
    }
}
