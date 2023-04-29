//! An HCL parser which keeps track of whitespace, comments and span information.

mod error;
mod expr;
mod number;
mod repr;
mod state;
mod string;
mod structure;
mod template;
#[cfg(test)]
mod tests;
mod trivia;

pub use self::error::{Error, Location};
use self::expr::expr;
use self::structure::body;
use self::template::template;
use crate::expr::Expression;
use crate::structure::Body;
use crate::template::Template;
use winnow::stream::UpdateSlice;

mod prelude {
    pub(super) use super::ParserExt;

    pub(super) use winnow::error::{ContextError, StrContext, StrContextValue};
    pub(super) use winnow::stream::Stream;
    pub(super) use winnow::{dispatch, PResult, Parser};

    pub(super) type Input<'a> = winnow::stream::Located<&'a [u8]>;
}

use self::prelude::*;

#[doc(hidden)]
pub trait ParserExt<I, O, E>: Parser<I, O, E> {
    /// Produce the consumed input as produced value.
    ///
    /// The produced value is of the same type as the input `Stream`. If you're looking for an
    /// alternative that returns `Stream::Slice`, use [`recognize`](Parser::recognize) instead.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use winnow::{error::{ErrMode, InputError, ErrorKind}, IResult, Parser};
    /// use hcl_edit::parser::ParserExt;
    /// use winnow::ascii::alpha1;
    /// use winnow::combinator::separated_pair;
    /// use winnow::stream::BStr;
    ///
    /// let mut parser = separated_pair(alpha1, ',', alpha1).recognize_stream();
    ///
    /// assert_eq!(
    ///     parser.parse_peek(BStr::new("abcd,efgh")),
    ///     Ok((BStr::new(""), BStr::new("abcd,efgh"))),
    /// );
    /// assert_eq!(
    ///     parser.parse_peek(BStr::new("abcd;")),
    ///     Err(ErrMode::Backtrack(InputError::new(BStr::new(";"), ErrorKind::Verify))),
    /// );
    /// ```
    fn recognize_stream(self) -> RecognizeStream<Self, I, O, E>
    where
        Self: core::marker::Sized,
        I: UpdateSlice + Clone,
    {
        RecognizeStream::new(self)
    }

    /// Produce the consumed input with the output.
    ///
    /// Functions similarly to [`recognize_stream`][Parser::recognize_stream] except it returns the
    /// parser output as well.
    ///
    /// This can be useful especially in cases where the output is not the same type as the input.
    ///
    /// The consumed input's value is of the same type as the input `Stream`. If you're looking for
    /// an alternative that returns `Stream::Slice`, use
    /// [`with_recognized`](Parser::with_recognized) instead.
    ///
    /// Returned tuple is of the format `(produced output, consumed input)`.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use winnow::{error::{ErrMode, InputError, ErrorKind}, PResult, Parser};
    /// use hcl_edit::parser::ParserExt;
    /// use winnow::ascii::alpha1;
    /// use winnow::combinator::separated_pair;
    /// use winnow::stream::BStr;
    ///
    /// fn inner_parser(input: &mut &BStr) -> PResult<bool> {
    ///     "1234".value(true).parse_next(input)
    /// }
    ///
    /// let mut consumed_parser = separated_pair(alpha1, ',', alpha1)
    ///     .value(true)
    ///     .with_recognized_stream();
    ///
    /// assert_eq!(
    ///     consumed_parser.parse_peek(BStr::new("abcd,efgh1")),
    ///     Ok((BStr::new("1"), (true, BStr::new("abcd,efgh")))),
    /// );
    /// assert_eq!(
    ///     consumed_parser.parse_peek(BStr::new("abcd;")),
    ///     Err(ErrMode::Backtrack(InputError::new(BStr::new(";"), ErrorKind::Verify))),
    /// );
    ///
    /// // The second output (representing the consumed input) should be the same as that of the
    /// // `recognize_stream` parser.
    /// let mut recognize_stream_parser = inner_parser.recognize_stream();
    /// let mut consumed_parser = inner_parser.with_recognized_stream()
    ///     .map(|(_output, recognized_stream)| recognized_stream);
    ///
    /// assert_eq!(
    ///     recognize_stream_parser.parse_peek(BStr::new("1234")),
    ///     consumed_parser.parse_peek(BStr::new("1234")),
    /// );
    /// assert_eq!(
    ///     recognize_stream_parser.parse_peek(BStr::new("abcd")),
    ///     consumed_parser.parse_peek(BStr::new("abcd")),
    /// );
    /// ```
    fn with_recognized_stream(self) -> WithRecognizedStream<Self, I, O, E>
    where
        Self: core::marker::Sized,
        I: UpdateSlice + Clone,
    {
        WithRecognizedStream::new(self)
    }
}

impl<I, O, E, T> ParserExt<I, O, E> for T where T: Parser<I, O, E> {}

/// Implementation of [`Parser::recognize_stream`]
#[cfg_attr(nightly, warn(rustdoc::missing_doc_code_examples))]
pub struct RecognizeStream<F, I, O, E>
where
    F: Parser<I, O, E>,
    I: UpdateSlice + Clone,
{
    parser: F,
    i: core::marker::PhantomData<I>,
    o: core::marker::PhantomData<O>,
    e: core::marker::PhantomData<E>,
}

impl<F, I, O, E> RecognizeStream<F, I, O, E>
where
    F: Parser<I, O, E>,
    I: UpdateSlice + Clone,
{
    pub(crate) fn new(parser: F) -> Self {
        Self {
            parser,
            i: Default::default(),
            o: Default::default(),
            e: Default::default(),
        }
    }
}

impl<I, O, E, F> Parser<I, I, E> for RecognizeStream<F, I, O, E>
where
    F: Parser<I, O, E>,
    I: UpdateSlice + Clone,
{
    fn parse_next(&mut self, input: &mut I) -> winnow::PResult<I, E> {
        let initial = input.clone();
        let checkpoint = input.checkpoint();
        match self.parser.parse_next(input) {
            Ok(_) => {
                let offset = input.offset_from(&checkpoint);
                input.reset(checkpoint);
                let slice = input.next_slice(offset);
                Ok(initial.update_slice(slice))
            }
            Err(e) => Err(e),
        }
    }
}

/// Implementation of [`Parser::with_recognized_stream`]
#[cfg_attr(nightly, warn(rustdoc::missing_doc_code_examples))]
pub struct WithRecognizedStream<F, I, O, E>
where
    F: Parser<I, O, E>,
    I: UpdateSlice + Clone,
{
    parser: F,
    i: core::marker::PhantomData<I>,
    o: core::marker::PhantomData<O>,
    e: core::marker::PhantomData<E>,
}

impl<F, I, O, E> WithRecognizedStream<F, I, O, E>
where
    F: Parser<I, O, E>,
    I: UpdateSlice + Clone,
{
    pub(crate) fn new(parser: F) -> Self {
        Self {
            parser,
            i: Default::default(),
            o: Default::default(),
            e: Default::default(),
        }
    }
}

impl<F, I, O, E> Parser<I, (O, I), E> for WithRecognizedStream<F, I, O, E>
where
    F: Parser<I, O, E>,
    I: UpdateSlice + Clone,
{
    fn parse_next(&mut self, input: &mut I) -> winnow::PResult<(O, I), E> {
        let initial = input.clone();
        let checkpoint = input.checkpoint();
        match self.parser.parse_next(input) {
            Ok(output) => {
                let offset = input.offset_from(&checkpoint);
                input.reset(checkpoint);
                let slice = input.next_slice(offset);
                Ok((output, initial.update_slice(slice)))
            }
            Err(e) => Err(e),
        }
    }
}

/// Parse an input into a [`Body`](crate::structure::Body).
///
/// # Errors
///
/// Returns an error if the input does not resemble a valid HCL body.
pub fn parse_body(input: &str) -> Result<Body, Error> {
    let mut body = parse_complete(input, body)?;
    body.despan(input);
    Ok(body)
}

/// Parse an input into an [`Expression`](crate::expr::Expression).
///
/// # Errors
///
/// Returns an error if the input does not resemble a valid HCL expression.
pub fn parse_expr(input: &str) -> Result<Expression, Error> {
    let mut expr = parse_complete(input, expr)?;
    expr.despan(input);
    Ok(expr)
}

/// Parse an input into a [`Template`](crate::template::Template).
///
/// # Errors
///
/// Returns an error if the input does not resemble a valid HCL template.
pub fn parse_template(input: &str) -> Result<Template, Error> {
    let mut template = parse_complete(input, template)?;
    template.despan(input);
    Ok(template)
}

fn parse_complete<'a, P, O>(input: &'a str, mut parser: P) -> Result<O, Error>
where
    P: Parser<Input<'a>, O, ContextError>,
{
    let input = Input::new(input.as_bytes());

    parser
        .parse(input)
        .map_err(|err| Error::from_parse_error(&err))
}
