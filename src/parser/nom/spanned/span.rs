use nom::{
    error::{ErrorKind, ParseError},
    AsBytes, Compare, CompareResult, Err, ExtendInto, FindSubstring, FindToken, IResult, InputIter,
    InputLength, InputTake, InputTakeAtPosition, Offset, ParseTo, Slice,
};
use std::fmt;
use std::hash::{Hash, Hasher};
use std::ops::{Deref, RangeFrom, RangeTo};
use std::str::FromStr;

#[derive(Debug, Clone, Copy)]
pub struct LocatedSpan<T> {
    offset: usize,
    fragment: T,
}

impl<T> Deref for LocatedSpan<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.fragment
    }
}

impl<T, U> AsRef<U> for LocatedSpan<&T>
where
    T: ?Sized + AsRef<U>,
    U: ?Sized,
{
    fn as_ref(&self) -> &U {
        self.fragment.as_ref()
    }
}

impl<T> LocatedSpan<T> {
    pub fn new(program: T) -> LocatedSpan<T> {
        LocatedSpan {
            offset: 0,
            fragment: program,
        }
    }
    pub fn location_offset(&self) -> usize {
        self.offset
    }

    pub fn fragment(&self) -> &T {
        &self.fragment
    }
}

impl<T: Hash> Hash for LocatedSpan<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.offset.hash(state);
        self.fragment.hash(state);
    }
}

impl<T: AsBytes> From<T> for LocatedSpan<T> {
    fn from(i: T) -> Self {
        LocatedSpan::new(i)
    }
}

impl<T: AsBytes + PartialEq> PartialEq for LocatedSpan<T> {
    fn eq(&self, other: &Self) -> bool {
        self.offset == other.offset && self.fragment == other.fragment
    }
}

impl<T: AsBytes + Eq> Eq for LocatedSpan<T> {}

impl<T: AsBytes> AsBytes for LocatedSpan<T> {
    fn as_bytes(&self) -> &[u8] {
        self.fragment.as_bytes()
    }
}

impl<T: InputLength> InputLength for LocatedSpan<T> {
    fn input_len(&self) -> usize {
        self.fragment.input_len()
    }
}

impl<T> InputTake for LocatedSpan<T>
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

impl<T> InputTakeAtPosition for LocatedSpan<T>
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
        match self.fragment.position(predicate) {
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
        match self.fragment.position(predicate) {
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
        match self.fragment.position(predicate) {
            Some(0) => Err(Err::Error(E::from_error_kind(self.clone(), e))),
            Some(n) => Ok(self.take_split(n)),
            None => {
                if self.fragment.input_len() == 0 {
                    Err(Err::Error(E::from_error_kind(self.clone(), e)))
                } else {
                    Ok(self.take_split(self.input_len()))
                }
            }
        }
    }
}

impl<'a, T> InputIter for LocatedSpan<T>
where
    T: InputIter,
{
    type Item = T::Item;
    type Iter = T::Iter;
    type IterElem = T::IterElem;

    fn iter_indices(&self) -> Self::Iter {
        self.fragment.iter_indices()
    }

    fn iter_elements(&self) -> Self::IterElem {
        self.fragment.iter_elements()
    }

    fn position<P>(&self, predicate: P) -> Option<usize>
    where
        P: Fn(Self::Item) -> bool,
    {
        self.fragment.position(predicate)
    }

    fn slice_index(&self, count: usize) -> Result<usize, nom::Needed> {
        self.fragment.slice_index(count)
    }
}

impl<A: Compare<B>, B: Into<LocatedSpan<B>>> Compare<B> for LocatedSpan<A> {
    fn compare(&self, t: B) -> CompareResult {
        self.fragment.compare(t.into().fragment)
    }

    fn compare_no_case(&self, t: B) -> CompareResult {
        self.fragment.compare_no_case(t.into().fragment)
    }
}

impl<'a, T, R> Slice<R> for LocatedSpan<T>
where
    T: Slice<R> + Offset + AsBytes + Slice<RangeTo<usize>>,
{
    fn slice(&self, range: R) -> Self {
        let next_fragment = self.fragment.slice(range);
        let consumed_len = self.fragment.offset(&next_fragment);
        let next_offset = self.offset + consumed_len;

        LocatedSpan {
            offset: next_offset,
            fragment: next_fragment,
        }
    }
}

impl<Fragment: FindToken<Token>, Token> FindToken<Token> for LocatedSpan<Fragment> {
    fn find_token(&self, token: Token) -> bool {
        self.fragment.find_token(token)
    }
}

impl<T, U> FindSubstring<U> for LocatedSpan<T>
where
    T: FindSubstring<U>,
{
    fn find_substring(&self, substr: U) -> Option<usize> {
        self.fragment.find_substring(substr)
    }
}

impl<R: FromStr, T> ParseTo<R> for LocatedSpan<T>
where
    T: ParseTo<R>,
{
    fn parse_to(&self) -> Option<R> {
        self.fragment.parse_to()
    }
}

impl<T> Offset for LocatedSpan<T> {
    fn offset(&self, second: &Self) -> usize {
        let fst = self.offset;
        let snd = second.offset;

        snd - fst
    }
}

impl<T: ToString> fmt::Display for LocatedSpan<T> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str(&self.fragment.to_string())
    }
}

impl<'a, T> ExtendInto for LocatedSpan<T>
where
    T: ExtendInto,
{
    type Item = T::Item;
    type Extender = T::Extender;

    fn new_builder(&self) -> Self::Extender {
        self.fragment.new_builder()
    }

    fn extend_into(&self, acc: &mut Self::Extender) {
        self.fragment.extend_into(acc)
    }
}

/// Capture the position of the current fragment
pub fn position<T, E>(s: T) -> IResult<T, T, E>
where
    E: ParseError<T>,
    T: InputIter + InputTake,
{
    nom::bytes::complete::take(0usize)(s)
}
