use nom::{
    error::{ErrorKind, ParseError},
    AsBytes, Compare, CompareResult, Err, ExtendInto, FindSubstring, FindToken, IResult, InputIter,
    InputLength, InputTake, InputTakeAtPosition, Offset, ParseTo, Slice,
};
use std::fmt;
use std::hash::{Hash, Hasher};
use std::ops::{Deref, RangeFrom, RangeTo};
use std::str::FromStr;

pub type Input<'a> = Located<&'a [u8]>;

pub trait Location {
    fn location(&self) -> usize;
}

#[derive(Debug, Clone, Copy)]
pub struct Located<T> {
    initial: T,
    input: T,
}

impl<T> Deref for Located<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.input
    }
}

impl<T> AsRef<T> for Located<T>
where
    T: AsRef<T>,
{
    fn as_ref(&self) -> &T {
        self.input.as_ref()
    }
}

impl<T> Located<T>
where
    T: Clone + Offset,
{
    pub fn new(input: T) -> Located<T> {
        let initial = input.clone();
        Located { input, initial }
    }
}

impl<T> Location for Located<T>
where
    T: Offset,
{
    fn location(&self) -> usize {
        self.initial.offset(&self.input)
    }
}

impl<T: Hash> Hash for Located<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.initial.hash(state);
        self.input.hash(state);
    }
}

impl<T: AsBytes + Clone + Offset> From<T> for Located<T> {
    fn from(i: T) -> Self {
        Located::new(i)
    }
}

impl<T: AsBytes + PartialEq> PartialEq for Located<T> {
    fn eq(&self, other: &Self) -> bool {
        self.initial == other.initial && self.input == other.input
    }
}

impl<T: AsBytes + Eq> Eq for Located<T> {}

impl<T: AsBytes> AsBytes for Located<T> {
    fn as_bytes(&self) -> &[u8] {
        self.input.as_bytes()
    }
}

impl<T: InputLength> InputLength for Located<T> {
    fn input_len(&self) -> usize {
        self.input.input_len()
    }
}

impl<T> InputTake for Located<T>
where
    Self: Slice<RangeFrom<usize>> + Slice<RangeTo<usize>>,
{
    fn take(&self, count: usize) -> Self {
        self.slice(..count)
    }

    fn take_split(&self, count: usize) -> (Self, Self) {
        (self.slice(count..), self.slice(..count))
    }
}

impl<T> InputTakeAtPosition for Located<T>
where
    T: InputTakeAtPosition + InputLength + InputIter,
    Self: Slice<RangeFrom<usize>> + Slice<RangeTo<usize>> + Clone,
{
    type Item = <T as InputIter>::Item;

    fn split_at_position_complete<P, E: ParseError<Self>>(
        &self,
        predicate: P,
    ) -> IResult<Self, Self, E>
    where
        P: Fn(Self::Item) -> bool,
    {
        match self.split_at_position(predicate) {
            Err(Err::Incomplete(_)) => Ok(self.take_split(self.input_len())),
            res => res,
        }
    }

    fn split_at_position<P, E: ParseError<Self>>(&self, predicate: P) -> IResult<Self, Self, E>
    where
        P: Fn(Self::Item) -> bool,
    {
        match self.input.position(predicate) {
            Some(n) => Ok(self.take_split(n)),
            None => Err(Err::Incomplete(nom::Needed::new(1))),
        }
    }

    fn split_at_position1<P, E: ParseError<Self>>(
        &self,
        predicate: P,
        e: ErrorKind,
    ) -> IResult<Self, Self, E>
    where
        P: Fn(Self::Item) -> bool,
    {
        match self.input.position(predicate) {
            Some(0) => Err(Err::Error(E::from_error_kind(self.clone(), e))),
            Some(n) => Ok(self.take_split(n)),
            None => Err(Err::Incomplete(nom::Needed::new(1))),
        }
    }

    fn split_at_position1_complete<P, E: ParseError<Self>>(
        &self,
        predicate: P,
        e: ErrorKind,
    ) -> IResult<Self, Self, E>
    where
        P: Fn(Self::Item) -> bool,
    {
        match self.input.position(predicate) {
            Some(0) => Err(Err::Error(E::from_error_kind(self.clone(), e))),
            Some(n) => Ok(self.take_split(n)),
            None => {
                if self.input.input_len() == 0 {
                    Err(Err::Error(E::from_error_kind(self.clone(), e)))
                } else {
                    Ok(self.take_split(self.input_len()))
                }
            }
        }
    }
}

impl<'a, T> InputIter for Located<T>
where
    T: InputIter,
{
    type Item = T::Item;
    type Iter = T::Iter;
    type IterElem = T::IterElem;

    fn iter_indices(&self) -> Self::Iter {
        self.input.iter_indices()
    }

    fn iter_elements(&self) -> Self::IterElem {
        self.input.iter_elements()
    }

    fn position<P>(&self, predicate: P) -> Option<usize>
    where
        P: Fn(Self::Item) -> bool,
    {
        self.input.position(predicate)
    }

    fn slice_index(&self, count: usize) -> Result<usize, nom::Needed> {
        self.input.slice_index(count)
    }
}

impl<A: Compare<B>, B: Into<Located<B>>> Compare<B> for Located<A> {
    fn compare(&self, t: B) -> CompareResult {
        self.input.compare(t.into().input)
    }

    fn compare_no_case(&self, t: B) -> CompareResult {
        self.input.compare_no_case(t.into().input)
    }
}

impl<'a, T, R> Slice<R> for Located<T>
where
    T: Slice<R> + Offset + Clone,
{
    fn slice(&self, range: R) -> Self {
        Located {
            initial: self.initial.clone(),
            input: self.input.slice(range),
        }
    }
}

impl<T: FindToken<Token>, Token> FindToken<Token> for Located<T> {
    fn find_token(&self, token: Token) -> bool {
        self.input.find_token(token)
    }
}

impl<T, U> FindSubstring<U> for Located<T>
where
    T: FindSubstring<U>,
{
    fn find_substring(&self, substr: U) -> Option<usize> {
        self.input.find_substring(substr)
    }
}

impl<R: FromStr, T> ParseTo<R> for Located<T>
where
    T: ParseTo<R>,
{
    fn parse_to(&self) -> Option<R> {
        self.input.parse_to()
    }
}

impl<T: Offset> Offset for Located<T> {
    fn offset(&self, second: &Self) -> usize {
        self.input.offset(&second.input)
    }
}

impl<T: ToString> fmt::Display for Located<T> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str(&self.input.to_string())
    }
}

impl<'a, T> ExtendInto for Located<T>
where
    T: ExtendInto,
{
    type Item = T::Item;
    type Extender = T::Extender;

    fn new_builder(&self) -> Self::Extender {
        self.input.new_builder()
    }

    fn extend_into(&self, acc: &mut Self::Extender) {
        self.input.extend_into(acc)
    }
}