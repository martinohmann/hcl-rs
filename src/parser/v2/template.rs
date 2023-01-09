use crate::template::Template;
use nom::{
    error::{ContextError, ParseError},
    IResult,
};

pub fn template<'a, E>(input: &'a str) -> IResult<&'a str, Template, E>
where
    E: ParseError<&'a str> + ContextError<&'a str> + 'a,
{
    let _input = input;
    unimplemented!()
}
