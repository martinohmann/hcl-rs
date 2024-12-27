#![allow(dead_code)]

use hcl::eval::{Context, Evaluate};
use hcl::format::{Format, FormatterBuilder};
use pretty_assertions::assert_eq;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[track_caller]
pub fn assert_serialize<T>(value: T, expected: &str)
where
    T: Serialize,
{
    assert_eq!(hcl::to_string(&value).unwrap(), expected);
}

#[track_caller]
pub fn assert_deserialize<'de, T>(input: &'de str, expected: T)
where
    T: Deserialize<'de> + Debug + PartialEq,
{
    assert_eq!(hcl::from_str::<T>(input).unwrap(), expected);
}

#[track_caller]
pub fn assert_format<T>(value: T, expected: &str)
where
    T: Format,
{
    assert_eq!(hcl::format::to_string(&value).unwrap(), expected);
}

#[track_caller]
pub fn assert_format_builder<T>(builder: FormatterBuilder<'_>, value: T, expected: &str)
where
    T: Format,
{
    let mut formatter = builder.build_vec();
    assert_eq!(value.format_string(&mut formatter).unwrap(), expected);
}

#[track_caller]
pub fn assert_eval<T, U>(value: T, expected: U)
where
    T: Evaluate<Output = U> + Debug + PartialEq,
    U: Debug + PartialEq,
{
    let ctx = Context::new();
    assert_eq!(value.evaluate(&ctx).unwrap(), expected);
}

#[track_caller]
pub fn assert_eval_ctx<T, U>(ctx: &Context, value: T, expected: U)
where
    T: Evaluate<Output = U> + Debug + PartialEq,
    U: Debug + PartialEq,
{
    assert_eq!(value.evaluate(ctx).unwrap(), expected);
}

#[track_caller]
pub fn assert_eval_error<T, E>(value: T, expected: E)
where
    T: Evaluate + Debug + PartialEq,
    <T as Evaluate>::Output: Debug,
    E: Into<hcl::eval::Error>,
{
    let ctx = Context::new();
    let err = value.evaluate(&ctx).unwrap_err();
    let expected = expected.into();
    assert_eq!(err.kind(), expected.kind());
    assert_eq!(err.expr(), expected.expr());
}
