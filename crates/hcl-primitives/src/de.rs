use core::fmt;
use core::marker::PhantomData;
use serde::de;

pub(crate) struct FromStrVisitor<T> {
    expecting: &'static str,
    marker: PhantomData<T>,
}

impl<T> FromStrVisitor<T> {
    pub(crate) fn new(expecting: &'static str) -> FromStrVisitor<T> {
        FromStrVisitor {
            expecting,
            marker: PhantomData,
        }
    }
}

impl<T> de::Visitor<'_> for FromStrVisitor<T>
where
    T: std::str::FromStr,
    T::Err: fmt::Display,
{
    type Value = T;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str(self.expecting)
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        T::from_str(value).map_err(de::Error::custom)
    }
}
