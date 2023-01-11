mod combinators;
mod comment;
mod expr;
mod primitives;
mod structure;
mod template;
#[cfg(test)]
mod tests;

use self::structure::body;
use self::template::template;

use crate::structure::Body;
use crate::template::Template;
use crate::{Error, Result};
use nom::{
    combinator::all_consuming,
    error::{convert_error, VerboseError},
    Finish, Parser,
};

pub fn parse<'a>(input: &'a str) -> Result<Body> {
    all_consuming(|input| body::<VerboseError<&'a str>>(input))
        .parse(input)
        .finish()
        .map(|(_, body)| body)
        .map_err(|err| Error::new(convert_error(input, err)))
}

pub fn parse_template<'a>(input: &'a str) -> Result<Template> {
    all_consuming(|input| template::<VerboseError<&'a str>>(input))
        .parse(input)
        .finish()
        .map(|(_, template)| template)
        .map_err(|err| Error::new(convert_error(input, err)))
}
