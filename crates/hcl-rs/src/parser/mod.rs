use crate::edit;
use crate::structure::Body;
use crate::template::Template;
use crate::Result;

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
    let body: edit::structure::Body = input.parse()?;
    Ok(body.into())
}

pub fn parse_template(input: &str) -> Result<Template> {
    let template: edit::template::Template = input.parse()?;
    Ok(template.into())
}
