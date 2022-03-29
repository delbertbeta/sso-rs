#[macro_export]
macro_rules! impl_from {
    ($from_struct: ident, $for_struct: ident, $for_field: ident) => {
        impl From<$from_struct> for $for_struct {
            fn from(inner: $from_struct) -> Self {
                Self::$for_field(inner)
            }
        }
    };
}
