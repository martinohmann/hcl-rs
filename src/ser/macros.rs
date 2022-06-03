#[doc(hidden)]
#[macro_export(local_inner_macros)]
macro_rules! serialize_unsupported {
    ($($func:ident)*) => {
        $(serialize_unsupported_helper!{$func})*
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! serialize_unsupported_method {
    ($func:ident<T>($($arg:ident : $ty:ty),*)) => {
        #[inline]
        fn $func<T>(self, $($arg: $ty,)*) -> $crate::Result<Self::Ok, Self::Error>
        where
            T: ?Sized + serde::ser::Serialize,
        {
            $(
                let _ = $arg;
            )*
            Err(serde::ser::Error::custom(format!("`{}` not supported", stringify!($func))))
        }
    };
    ($func:ident($($arg:ident : $ty:ty),*) -> Result<$rty:ident>) => {
        #[inline]
        fn $func(self, $($arg: $ty,)*) -> $crate::Result<Self::$rty, Self::Error> {
            $(
                let _ = $arg;
            )*
            Err(serde::ser::Error::custom(format!("`{}` not supported", stringify!($func))))
        }
    };
    ($func:ident($($arg:ident : $ty:ty),*)) => {
        #[inline]
        fn $func(self, $($arg: $ty,)*) -> $crate::Result<Self::Ok, Self::Error> {
            $(
                let _ = $arg;
            )*
            Err(serde::ser::Error::custom(format!("`{}` not supported", stringify!($func))))
        }
    };
}

#[doc(hidden)]
#[macro_export(local_inner_macros)]
macro_rules! serialize_unsupported_helper {
    (bool) => {
        serialize_unsupported_method!{serialize_bool(v: bool)}
    };
    (i8) => {
        serialize_unsupported_method!{serialize_i8(v: i8)}
    };
    (i16) => {
        serialize_unsupported_method!{serialize_i16(v: i16)}
    };
    (i32) => {
        serialize_unsupported_method!{serialize_i32(v: i32)}
    };
    (i64) => {
        serialize_unsupported_method!{serialize_i64(v: i64)}
    };
    (i128) => {
        serde_if_integer128! {
            serialize_unsupported_method!{serialize_i128(v: i128)}
        }
    };
    (u8) => {
        serialize_unsupported_method!{serialize_u8(v: u8)}
    };
    (u16) => {
        serialize_unsupported_method!{serialize_u16(v: u16)}
    };
    (u32) => {
        serialize_unsupported_method!{serialize_u32(v: u32)}
    };
    (u64) => {
        serialize_unsupported_method!{serialize_u64(v: u64)}
    };
    (u128) => {
        serde_if_integer128! {
            serialize_unsupported_method!{serialize_u128(v: u128)}
        }
    };
    (f32) => {
        serialize_unsupported_method!{serialize_f32(v: f32)}
    };
    (f64) => {
        serialize_unsupported_method!{serialize_f64(v: f64)}
    };
    (char) => {
        serialize_unsupported_method!{serialize_char(v: char)}
    };
    (str) => {
        serialize_unsupported_method!{serialize_str(v: &str)}
    };
    (bytes) => {
        serialize_unsupported_method!{serialize_bytes(v: &[u8])}
    };
    (some) => {
        serialize_unsupported_method!{serialize_some<T>(value: &T)}
    };
    (none) => {
        serialize_unsupported_method!{serialize_none()}
    };
    (unit) => {
        serialize_unsupported_method!{serialize_unit()}
    };
    (unit_struct) => {
        serialize_unsupported_method!{serialize_unit_struct(name: &'static str)}
    };
    (unit_variant) => {
        serialize_unsupported_method!{serialize_unit_variant(name: &'static str, variant_index: u32, variant: &'static str)}
    };
    (newtype_struct) => {
        serialize_unsupported_method!{serialize_newtype_struct<T>(name: &'static str, value: &T)}
    };
    (newtype_variant) => {
        serialize_unsupported_method!{serialize_newtype_variant<T>(name: &'static str, variant_index: u32, variant: &'static str, value: &T)}
    };
    (seq) => {
        serialize_unsupported_method!{serialize_seq(len: Option<usize>) -> Result<SerializeSeq>}
    };
    (tuple) => {
        serialize_unsupported_method!{serialize_tuple(len: usize) -> Result<SerializeTuple>}
    };
    (tuple_struct) => {
        serialize_unsupported_method!{serialize_tuple_struct(name: &'static str, len: usize) -> Result<SerializeTupleStruct>}
    };
    (tuple_variant) => {
        serialize_unsupported_method!{serialize_tuple_variant(name: &'static str, variant_index: u32, variant: &'static str, len: usize) -> Result<SerializeTupleVariant>}
    };
    (map) => {
        serialize_unsupported_method!{serialize_map(len: Option<usize>) -> Result<SerializeMap>}
    };
    (struct) => {
        serialize_unsupported_method!{serialize_struct(name: &'static str, len: usize) -> Result<SerializeStruct>}
    };
    (struct_variant) => {
        serialize_unsupported_method!{serialize_struct_variant(name: &'static str, variant_index: u32, variant: &'static str, len: usize) -> Result<SerializeStructVariant>}
    };
}
