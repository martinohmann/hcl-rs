#[cfg(feature = "nom")]
pub(crate) mod nom;
#[cfg(feature = "pest")]
pub(crate) mod pest;

#[cfg(feature = "nom")]
use self::nom::parse as parse_impl;
#[cfg(all(feature = "nom", not(feature = "pest")))]
use self::nom::parse_template as parse_template_impl;

#[cfg(all(feature = "pest", not(feature = "nom")))]
use self::pest::parse as parse_impl;
#[cfg(all(feature = "pest"))]
use self::pest::parse_template as parse_template_impl;

use crate::{structure::Body, template::Template, Result};

/// Parse a `hcl::Body` from a `&str`.
///
/// If deserialization into a different type is preferred consider using [`hcl::from_str`][from_str].
///
/// [from_str]: ./de/fn.from_str.html
///
/// # Example
///
/// ```
/// use hcl::{Attribute, Block, Body};
/// # use std::error::Error;
/// #
/// # fn main() -> Result<(), Box<dyn Error>> {
/// let input = r#"
///     some_attr = "foo"
///
///     some_block "some_block_label" {
///       attr = "value"
///     }
/// "#;
///
/// let expected = Body::builder()
///     .add_attribute(("some_attr", "foo"))
///     .add_block(
///         Block::builder("some_block")
///             .add_label("some_block_label")
///             .add_attribute(("attr", "value"))
///             .build()
///     )
///     .build();
///
/// let body = hcl::parse(input)?;
///
/// assert_eq!(body, expected);
/// #   Ok(())
/// # }
/// ```
///
/// # Errors
///
/// This function fails with an error if the `input` cannot be parsed as HCL.
pub fn parse(input: &str) -> Result<Body> {
    parse_impl(input)
}

pub fn parse_template(input: &str) -> Result<Template> {
    parse_template_impl(input)
}
