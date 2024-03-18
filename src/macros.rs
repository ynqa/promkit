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

#[macro_export]
macro_rules! impl_cast {
    ($type:ty) => {
        impl $type {
            pub fn cast_mut(renderer: &mut dyn $crate::Renderer) -> $crate::Result<&mut Self> {
                let snapshot = renderer
                    .as_any_mut()
                    .downcast_mut::<Self>()
                    .ok_or_else(|| {
                        $crate::Error::DowncastError(std::any::type_name::<Self>().to_string())
                    })?;
                Ok(snapshot)
            }

            pub fn cast(renderer: &dyn $crate::Renderer) -> $crate::Result<&Self> {
                let snapshot = renderer.as_any().downcast_ref::<Self>().ok_or_else(|| {
                    $crate::Error::DowncastError(std::any::type_name::<Self>().to_string())
                })?;
                Ok(snapshot)
            }
        }
    };
}
