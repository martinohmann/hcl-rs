#[doc(hidden)]
#[macro_export(local_inner_macros)]
macro_rules! serialize_unsupported {
    ($err_fn:ident $($func:ident)*) => {
        $(serialize_unsupported_helper!{$err_fn $func})*
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! serialize_unsupported_method {
    ($err_fn:ident $func:ident<T>($($arg:ident : $ty:ty),*)) => {
        #[inline]
        fn $func<T>(self, $($arg: $ty,)*) -> $crate::Result<Self::Ok, Self::Error>
        where
            T: ?Sized + Serialize,
        {
            $(
                let _ = $arg;
            )*
            Err($err_fn())
        }
    };
    ($err_fn:ident $func:ident($($arg:ident : $ty:ty),*) -> Result<$rty:ident>) => {
        #[inline]
        fn $func(self, $($arg: $ty,)*) -> $crate::Result<Self::$rty, Self::Error> {
            $(
                let _ = $arg;
            )*
            Err($err_fn())
        }
    };
    ($err_fn:ident $func:ident($($arg:ident : $ty:ty),*)) => {
        #[inline]
        fn $func(self, $($arg: $ty,)*) -> $crate::Result<Self::Ok, Self::Error> {
            $(
                let _ = $arg;
            )*
            Err($err_fn())
        }
    };
}

#[doc(hidden)]
#[macro_export(local_inner_macros)]
macro_rules! serialize_unsupported_helper {
    ($err_fn:ident bool) => {
        serialize_unsupported_method!{$err_fn serialize_bool(v: bool)}
    };
    ($err_fn:ident i8) => {
        serialize_unsupported_method!{$err_fn serialize_i8(v: i8)}
    };
    ($err_fn:ident i16) => {
        serialize_unsupported_method!{$err_fn serialize_i16(v: i16)}
    };
    ($err_fn:ident i32) => {
        serialize_unsupported_method!{$err_fn serialize_i32(v: i32)}
    };
    ($err_fn:ident i64) => {
        serialize_unsupported_method!{$err_fn serialize_i64(v: i64)}
    };
    ($err_fn:ident i128) => {
        serde_if_integer128! {
            serialize_unsupported_method!{$err_fn serialize_i128(v: i128)}
        }
    };
    ($err_fn:ident u8) => {
        serialize_unsupported_method!{$err_fn serialize_u8(v: u8)}
    };
    ($err_fn:ident u16) => {
        serialize_unsupported_method!{$err_fn serialize_u16(v: u16)}
    };
    ($err_fn:ident u32) => {
        serialize_unsupported_method!{$err_fn serialize_u32(v: u32)}
    };
    ($err_fn:ident u64) => {
        serialize_unsupported_method!{$err_fn serialize_u64(v: u64)}
    };
    ($err_fn:ident u128) => {
        serde_if_integer128! {
            serialize_unsupported_method!{$err_fn serialize_u128(v: u128)}
        }
    };
    ($err_fn:ident f32) => {
        serialize_unsupported_method!{$err_fn serialize_f32(v: f32)}
    };
    ($err_fn:ident f64) => {
        serialize_unsupported_method!{$err_fn serialize_f64(v: f64)}
    };
    ($err_fn:ident char) => {
        serialize_unsupported_method!{$err_fn serialize_char(v: char)}
    };
    ($err_fn:ident str) => {
        serialize_unsupported_method!{$err_fn serialize_str(v: &str)}
    };
    ($err_fn:ident bytes) => {
        serialize_unsupported_method!{$err_fn serialize_bytes(v: &[u8])}
    };
    ($err_fn:ident some) => {
        serialize_unsupported_method!{$err_fn serialize_some<T>(value: &T)}
    };
    ($err_fn:ident none) => {
        serialize_unsupported_method!{$err_fn serialize_none()}
    };
    ($err_fn:ident unit) => {
        serialize_unsupported_method!{$err_fn serialize_unit()}
    };
    ($err_fn:ident unit_struct) => {
        serialize_unsupported_method!{$err_fn serialize_unit_struct(name: &'static str)}
    };
    ($err_fn:ident unit_variant) => {
        serialize_unsupported_method!{$err_fn serialize_unit_variant(name: &'static str, variant_index: u32, variant: &'static str)}
    };
    ($err_fn:ident newtype_struct) => {
        serialize_unsupported_method!{$err_fn serialize_newtype_struct<T>(name: &'static str, value: &T)}
    };
    ($err_fn:ident newtype_variant) => {
        serialize_unsupported_method!{$err_fn serialize_newtype_variant<T>(name: &'static str, variant_index: u32, variant: &'static str, value: &T)}
    };
    ($err_fn:ident seq) => {
        serialize_unsupported_method!{$err_fn serialize_seq(len: Option<usize>) -> Result<SerializeSeq>}
    };
    ($err_fn:ident tuple) => {
        serialize_unsupported_method!{$err_fn serialize_tuple(len: usize) -> Result<SerializeTuple>}
    };
    ($err_fn:ident tuple_struct) => {
        serialize_unsupported_method!{$err_fn serialize_tuple_struct(name: &'static str, len: usize) -> Result<SerializeTupleStruct>}
    };
    ($err_fn:ident tuple_variant) => {
        serialize_unsupported_method!{$err_fn serialize_tuple_variant(name: &'static str, variant_index: u32, variant: &'static str, len: usize) -> Result<SerializeTupleVariant>}
    };
    ($err_fn:ident map) => {
        serialize_unsupported_method!{$err_fn serialize_map(len: Option<usize>) -> Result<SerializeMap>}
    };
    ($err_fn:ident struct) => {
        serialize_unsupported_method!{$err_fn serialize_struct(name: &'static str, len: usize) -> Result<SerializeStruct>}
    };
    ($err_fn:ident struct_variant) => {
        serialize_unsupported_method!{$err_fn serialize_struct_variant(name: &'static str, variant_index: u32, variant: &'static str, len: usize) -> Result<SerializeStructVariant>}
    };
}
