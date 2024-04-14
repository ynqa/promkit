#[macro_export]
macro_rules! impl_as_any {
    ($type:ty) => {
        impl $crate::AsAny for $type {
            fn as_any(&self) -> &dyn std::any::Any {
                self
            }

            fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
                self
            }
        }
    };
}
