/// Construct an `hcl::Body` from HCL blocks and attributes.
///
/// The macro supports a subset of the HCL syntax. If you need more flexibility, use the
/// [`BlockBuilder`][BlockBuilder] instead.
///
/// Unsupported syntax:
///
/// - Raw HCL expressions in attribute values and object keys
/// - A mix of identifier and string block labels
///
/// [BlockBuilder]: ./struct.BlockBuilder.html
///
/// ```
/// let body = hcl::body!({
///     resource "aws_sns_topic" "topic" {
///         name = "my-topic"
///     }
/// });
///
/// let expected = hcl::Body::builder()
///     .add_block(
///         hcl::Block::builder("resource")
///             .add_label("aws_sns_topic")
///             .add_label("topic")
///             .add_attribute(("name", "my-topic"))
///             .build()
///     )
///     .build();
///
/// assert_eq!(body, expected);
/// ```
///
/// Attribute keys and block identifiers can be expressions:
///
/// ```
/// use hcl::{Block, Body};
///
/// let block_identifier = "resource";
/// let attribute_key = "name";
///
/// let body = hcl::body!({
///     (block_identifier) "aws_sns_topic" "topic" {
///         (attribute_key) = "my-topic"
///     }
/// });
///
/// let expected = Body::builder()
///     .add_block(
///         Block::builder(block_identifier)
///             .add_label("aws_sns_topic")
///             .add_label("topic")
///             .add_attribute((attribute_key, "my-topic"))
///             .build()
///     )
///     .build();
///
/// assert_eq!(body, expected);
/// ```
#[macro_export]
macro_rules! body {
    // Hide distracting implementation details from the generated rustdoc.
    ($($body:tt)*) => {
        $crate::body_internal!($($body)*)
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! body_internal {
    //////////////////////////////////////////////////////////////////////////
    // TT muncher for parsing zero or more HCL structures (attributes and blocks).
    //
    // Produces a vec![...] of the elements.
    //
    // Must be invoked as: body_internal!(@structures [] $($tt)*)
    //////////////////////////////////////////////////////////////////////////

    // No tokens left, done.
    (@structures [$(($elems:expr))*]) => {
        std::vec![$($elems),*]
    };

    // Next element is an attribute, munch into elems and prceed with next structure.
    (@structures [$(($elems:expr))*] $key:tt = $expr:tt $($rest:tt)*) => {
        $crate::body_internal!(@structures [$(($elems))* ($crate::body_internal!(@attribute $key = $expr))] $($rest)*)
    };

    // Next element must be a block, invoke block muncher.
    (@structures [$(($elems:expr))*] $ident:tt $($rest:tt)+) => {
        $crate::body_internal!(@block [$(($elems))*] $ident $($rest)+)
    };

    //////////////////////////////////////////////////////////////////////////
    // TT muncher for parsing an HCL block.
    //
    // Must be invoked as: body_internal!(@block [$(($elems))*] $ident $($rest)*)
    //////////////////////////////////////////////////////////////////////////

    // Found block body, munch block into elems and proceed with next structure.
    (@block [$(($elems:expr))*] ($ident:expr) [$(($labels:expr))*] { $($body:tt)* } $($rest:tt)*) => {
        $crate::body_internal!(@structures [$(($elems))* ($crate::body_internal!(@block ($ident) [$(($labels))*] { $($body)* }))] $($rest)*)
    };

    // Munch an identifier block label.
    (@block [$(($elems:expr))*] ($ident:expr) [$(($labels:expr))*] $label:ident $($rest:tt)+) => {
        $crate::body_internal!(@block [$(($elems))*] ($ident) [$(($labels))* ($crate::block_label!($label))] $($rest)+)
    };

    // Munch a literal expression block label.
    (@block [$(($elems:expr))*] ($ident:expr) [$(($labels:expr))*] $label:literal $($rest:tt)+) => {
        $crate::body_internal!(@block [$(($elems))*] ($ident) [$(($labels))* ($crate::block_label!($label))] $($rest)+)
    };

    // Munch an expression block label.
    (@block [$(($elems:expr))*] ($ident:expr) [$(($labels:expr))*] ($label:expr) $($rest:tt)+) => {
        $crate::body_internal!(@block [$(($elems))*] ($ident) [$(($labels))* ($crate::block_label!(($label)))] $($rest)+)
    };

    // Block with identifier.
    (@block [$(($elems:expr))*] $ident:ident $($rest:tt)+) => {
        $crate::body_internal!(@block [$(($elems))*] (std::stringify!($ident)) [] $($rest)+)
    };

    // Block with expression as identifier.
    (@block [$(($elems:expr))*] ($ident:expr) $($rest:tt)+) => {
        $crate::body_internal!(@block [$(($elems))*] ($ident) [] $($rest)+)
    };

    //////////////////////////////////////////////////////////////////////////
    // The main implementation.
    //
    // Must be invoked as: body_internal!($($tt)+)
    //////////////////////////////////////////////////////////////////////////

    // Attribute structure.
    (@attribute $key:tt = $expr:tt) => {
        $crate::Structure::Attribute($crate::attribute_internal!($key = $expr))
    };

    // Block structure.
    (@block ($ident:expr) [$(($labels:expr))*] { $($body:tt)* }) => {
        $crate::Structure::Block($crate::block_internal!(($ident) [$(($labels))*] { $($body)* }))
    };

    // Body in braces.
    ({ $($tt:tt)* }) => {
        $crate::body_internal!($($tt)*)
    };

    // Invoke structure muncher.
    ($($tt:tt)*) => {
        $crate::Body($crate::body_internal!(@structures [] $($tt)*))
    };
}

/// Construct an `hcl::Structure` which may be either an HCL attribute or block.
///
/// ```
/// use hcl::{Attribute, Block, Structure};
///
/// assert_eq!(
///     hcl::structure!(foo = "bar"),
///     Structure::Attribute(Attribute::new("foo", "bar")),
/// );
///
/// assert_eq!(
///     hcl::structure!(
///         resource "aws_s3_bucket" "bucket" {
///             name = "the-bucket"
///         }
///     ),
///     Structure::Block(
///         Block::builder("resource")
///             .add_labels(["aws_s3_bucket", "bucket"])
///             .add_attribute(("name", "the-bucket"))
///             .build()
///     ),
/// );
/// ```
#[macro_export]
macro_rules! structure {
    // Hide distracting implementation details from the generated rustdoc.
    ($($structure:tt)+) => {
        $crate::structure_internal!($($structure)+)
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! structure_internal {
    // Structure in braces.
    ({ $($structure:tt)+ }) => {
        $crate::structure_internal!($($structure)+)
    };

    // An attribute structure.
    ($key:tt = $($expr:tt)+) => {
        $crate::Structure::Attribute($crate::attribute_internal!($key = $($expr)+))
    };

    // A block structure.
    ($($block:tt)+) => {
        $crate::Structure::Block($crate::block_internal!($($block)+))
    };
}

/// Construct an `hcl::Attribute` from a key and a value expression.
///
/// ```
/// use hcl::Attribute;
///
/// assert_eq!(hcl::attribute!(foo = "bar"), Attribute::new("foo", "bar"));
/// ```
#[macro_export]
macro_rules! attribute {
    // Hide distracting implementation details from the generated rustdoc.
    ($($attr:tt)+) => {
        $crate::attribute_internal!($($attr)+)
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! attribute_internal {
    // Attribute in curly braces.
    ({ $($attribute:tt)+ }) => {
        $crate::attribute_internal!($($attribute)+)
    };

    // Attribute with identifier as key.
    ($key:ident = $($expr:tt)+) => {
        $crate::Attribute {
            key: std::stringify!($key).to_owned(),
            expr: $crate::expression_internal!($($expr)+),
        }
    };

    // Attribute with expression as key.
    (($key:expr) = $($expr:tt)+) => {
        $crate::Attribute {
            key: ($key).into(),
            expr: $crate::expression_internal!($($expr)+),
        }
    };
}

/// Construct an `hcl::Block` from a block identifier, optional block labels and a block body.
///
/// ```
/// use hcl::Block;
///
/// assert_eq!(
///     hcl::block!(
///         resource "aws_s3_bucket" "bucket" {
///             name = "the-bucket"
///         }
///     ),
///     Block::builder("resource")
///         .add_labels(["aws_s3_bucket", "bucket"])
///         .add_attribute(("name", "the-bucket"))
///         .build(),
/// );
/// ```
#[macro_export]
macro_rules! block {
    // Hide distracting implementation details from the generated rustdoc.
    ($($block:tt)+) => {
        $crate::block_internal!($($block)+)
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! block_internal {
    // Block in curly braces.
    ({ $($block:tt)+ }) => {
        $crate::block_internal!($($block)+)
    };

    // Munch an identifier label.
    (($ident:expr) [$(($labels:expr))*] $label:ident $($rest:tt)+) => {
        $crate::block_internal!(($ident) [$(($labels))* ($crate::block_label!($label))] $($rest)+)
    };

    // Munch a literal expression label.
    (($ident:expr) [$(($labels:expr))*] $label:literal $($rest:tt)+) => {
        $crate::block_internal!(($ident) [$(($labels))* ($crate::block_label!($label))] $($rest)+)
    };

    // Munch an expression label.
    (($ident:expr) [$(($labels:expr))*] ($label:expr) $($rest:tt)+) => {
        $crate::block_internal!(($ident) [$(($labels))* ($crate::block_label!(($label)))] $($rest)+)
    };

    // Found block body, done.
    (($ident:expr) [$(($labels:expr))*] {$($body:tt)*}) => {
        $crate::Block {
            identifier: ($ident).into(),
            labels: std::vec![$($labels),*],
            body: $crate::body!($($body)*),
        }
    };

    // Munch expression as block identifier.
    (($ident:expr) $($rest:tt)+) => {
        $crate::block_internal!(($ident) [] $($rest)+)
    };

    // Munch normal block identifier.
    ($ident:ident $($rest:tt)+) => {
        $crate::block_internal!((std::stringify!($ident)) [] $($rest)+)
    };
}

/// Construct an `hcl::BlockLabel`.
///
/// ```
/// use hcl::BlockLabel;
///
/// assert_eq!(hcl::block_label!(some_identifier), BlockLabel::identifier("some_identifier"));
/// assert_eq!(hcl::block_label!("some string"), BlockLabel::string("some string"));
///
/// let label = "some expression";
///
/// assert_eq!(hcl::block_label!((label)), BlockLabel::string("some expression"));
/// ```
#[macro_export]
#[doc(hidden)]
macro_rules! block_label {
    ($ident:ident) => {
        $crate::BlockLabel::Identifier(std::stringify!($ident).into())
    };

    (($expr:expr)) => {
        $crate::BlockLabel::String(($expr).into())
    };

    ($literal:literal) => {
        $crate::BlockLabel::String($literal.into())
    };
}

/// Construct an `hcl::Expression` from an HCL attribute value expression literal.
///
/// ```
/// use hcl::{Expression, Object};
///
/// let expression = hcl::expression!({
///     "foo" = "bar"
///     "baz" = [1, 2]
/// });
///
/// let expected = Expression::Object({
///     let mut object = Object::new();
///     object.insert("foo".into(), "bar".into());
///     object.insert("baz".into(), vec![1u64, 2].into());
///     object
/// });
///
/// assert_eq!(expression, expected);
/// ```
#[macro_export]
macro_rules! expression {
    // Hide distracting implementation details from the generated rustdoc.
    ($($expr:tt)+) => {
        $crate::expression_internal!($($expr)+)
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! expression_internal {
    //////////////////////////////////////////////////////////////////////////
    // TT muncher for parsing the inside of an array [...]. Produces a vec![...]
    // of the elements.
    //
    // Must be invoked as: expression_internal!(@array [] $($tt)*)
    //////////////////////////////////////////////////////////////////////////

    // Done with trailing comma.
    (@array [$($elems:expr,)*]) => {
        std::vec![$($elems,)*]
    };

    // Done without trailing comma.
    (@array [$($elems:expr),*]) => {
        std::vec![$($elems),*]
    };

    // Next element is `null`.
    (@array [$($elems:expr,)*] null $($rest:tt)*) => {
        $crate::expression_internal!(@array [$($elems,)* $crate::expression_internal!(null)] $($rest)*)
    };

    // Next element is `true`.
    (@array [$($elems:expr,)*] true $($rest:tt)*) => {
        $crate::expression_internal!(@array [$($elems,)* $crate::expression_internal!(true)] $($rest)*)
    };

    // Next element is `false`.
    (@array [$($elems:expr,)*] false $($rest:tt)*) => {
        $crate::expression_internal!(@array [$($elems,)* $crate::expression_internal!(false)] $($rest)*)
    };

    // Next element is an array.
    (@array [$($elems:expr,)*] [$($array:tt)*] $($rest:tt)*) => {
        $crate::expression_internal!(@array [$($elems,)* $crate::expression_internal!([$($array)*])] $($rest)*)
    };

    // Next element is a map.
    (@array [$($elems:expr,)*] {$($map:tt)*} $($rest:tt)*) => {
        $crate::expression_internal!(@array [$($elems,)* $crate::expression_internal!({$($map)*})] $($rest)*)
    };

    // Next element is an expression followed by comma.
    (@array [$($elems:expr,)*] $next:expr, $($rest:tt)*) => {
        $crate::expression_internal!(@array [$($elems,)* $crate::expression_internal!($next),] $($rest)*)
    };

    // Last element is an expression with no trailing comma.
    (@array [$($elems:expr,)*] $last:expr) => {
        $crate::expression_internal!(@array [$($elems,)* $crate::expression_internal!($last)])
    };

    // Comma after the most recent element.
    (@array [$($elems:expr),*] , $($rest:tt)*) => {
        $crate::expression_internal!(@array [$($elems,)*] $($rest)*)
    };

    // Unexpected token after most recent element.
    (@array [$($elems:expr),*] $unexpected:tt $($rest:tt)*) => {
        $crate::hcl_unexpected!($unexpected)
    };

    //////////////////////////////////////////////////////////////////////////
    // TT muncher for parsing the inside of an object {...}. Each entry is
    // inserted into the given map variable.
    //
    // Must be invoked as: expression_internal!(@object $map () ($($tt)*) ($($tt)*))
    //
    // We require two copies of the input tokens so that we can match on one
    // copy and trigger errors on the other copy.
    //////////////////////////////////////////////////////////////////////////

    // Done.
    (@object $object:ident () () ()) => {};

    // Insert the current entry followed by trailing comma.
    (@object $object:ident [$($key:tt)+] ($value:expr) , $($rest:tt)*) => {
        let _ = $object.insert(($($key)+).into(), $value);
        $crate::expression_internal!(@object $object () ($($rest)*) ($($rest)*));
    };

    // Insert the current entry not followed by trailing comma.
    (@object $object:ident [$($key:tt)+] ($value:expr) $($rest:tt)*) => {
        let _ = $object.insert(($($key)+).into(), $value);
        $crate::expression_internal!(@object $object () ($($rest)*) ($($rest)*));
    };

    // Insert the last entry without trailing comma.
    (@object $object:ident [$($key:tt)+] ($value:expr)) => {
        let _ = $object.insert(($($key)+).into(), $value);
    };

    // Next value is `null`.
    (@object $object:ident ($($key:tt)+) (= null $($rest:tt)*) $copy:tt) => {
        $crate::expression_internal!(@object $object [$($key)+] ($crate::expression_internal!(null)) $($rest)*);
    };

    // Next value is `true`.
    (@object $object:ident ($($key:tt)+) (= true $($rest:tt)*) $copy:tt) => {
        $crate::expression_internal!(@object $object [$($key)+] ($crate::expression_internal!(true)) $($rest)*);
    };

    // Next value is `false`.
    (@object $object:ident ($($key:tt)+) (= false $($rest:tt)*) $copy:tt) => {
        $crate::expression_internal!(@object $object [$($key)+] ($crate::expression_internal!(false)) $($rest)*);
    };

    // Next value is an array.
    (@object $object:ident ($($key:tt)+) (= [$($array:tt)*] $($rest:tt)*) $copy:tt) => {
        $crate::expression_internal!(@object $object [$($key)+] ($crate::expression_internal!([$($array)*])) $($rest)*);
    };

    // Next value is a map.
    (@object $object:ident ($($key:tt)+) (= {$($map:tt)*} $($rest:tt)*) $copy:tt) => {
        $crate::expression_internal!(@object $object [$($key)+] ($crate::expression_internal!({$($map)*})) $($rest)*);
    };

    // Next value is an expression followed by comma.
    (@object $object:ident ($($key:tt)+) (= $value:expr , $($rest:tt)*) $copy:tt) => {
        $crate::expression_internal!(@object $object [$($key)+] ($crate::expression_internal!($value)) , $($rest)*);
    };

    // Next value is an expression not followed by comma.
    (@object $object:ident ($($key:tt)+) (= $value:tt $($rest:tt)*) $copy:tt) => {
        $crate::expression_internal!(@object $object [$($key)+] ($crate::expression_internal!($value)) $($rest)*);
    };

    // Last value is an expression with no trailing comma.
    (@object $object:ident ($($key:tt)+) (= $value:expr) $copy:tt) => {
        $crate::expression_internal!(@object $object [$($key)+] ($crate::expression_internal!($value)));
    };

    // Missing value for last entry. Trigger a reasonable error message.
    (@object $object:ident ($($key:tt)+) (=) $copy:tt) => {
        // "unexpected end of macro invocation"
        $crate::expression_internal!();
    };

    // Missing equals and value for last entry. Trigger a reasonable error
    // message.
    (@object $object:ident ($($key:tt)+) () $copy:tt) => {
        // "unexpected end of macro invocation"
        $crate::expression_internal!();
    };

    // Misplaced equals. Trigger a reasonable error message.
    (@object $object:ident () (= $($rest:tt)*) ($equals:tt $($copy:tt)*)) => {
        // Takes no arguments so "no rules expected the token `:`".
        $crate::hcl_unexpected!($equals);
    };

    // Found a comma inside a key. Trigger a reasonable error message.
    (@object $object:ident ($($key:tt)*) (, $($rest:tt)*) ($comma:tt $($copy:tt)*)) => {
        // Takes no arguments so "no rules expected the token `,`".
        $crate::hcl_unexpected!($comma);
    };

    // Key is fully parenthesized. This avoids clippy double_parens false
    // positives because the parenthesization may be necessary here.
    (@object $object:ident () (($key:expr) = $($rest:tt)*) $copy:tt) => {
        $crate::expression_internal!(@object $object ($key) (= $($rest)*) (= $($rest)*));
    };

    // Refuse to absorb equals token into key expression.
    (@object $object:ident ($($key:tt)*) (= $($unexpected:tt)+) $copy:tt) => {
        $crate::hcl_expect_expr_comma!($($unexpected)+);
    };

    // Munch a token into the current key.
    (@object $object:ident ($($key:tt)*) ($tt:tt $($rest:tt)*) $copy:tt) => {
        $crate::expression_internal!(@object $object ($($key)* $tt) ($($rest)*) ($($rest)*));
    };

    //////////////////////////////////////////////////////////////////////////
    // The main implementation.
    //
    // Must be invoked as: expression_internal!($($expr)+)
    //////////////////////////////////////////////////////////////////////////

    (null) => {
        $crate::Expression::Null
    };

    (true) => {
        $crate::Expression::Bool(true)
    };

    (false) => {
        $crate::Expression::Bool(false)
    };

    ([]) => {
        $crate::Expression::Array(std::vec![])
    };

    ([ $($tt:tt)+ ]) => {
        $crate::Expression::Array($crate::expression_internal!(@array [] $($tt)+))
    };

    ({}) => {
        $crate::Expression::Object($crate::Object::new())
    };

    ({ $($tt:tt)+ }) => {
        $crate::Expression::Object({
            let mut object = $crate::Object::new();
            $crate::expression_internal!(@object object () ($($tt)+) ($($tt)+));
            object
        })
    };

    // Any Serialize type: numbers, strings, struct literals, variables etc.
    // Must be below every other rule.
    ($other:expr) => {
        $crate::to_expression(&$other).unwrap()
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! hcl_unexpected {
    () => {};
}

#[macro_export]
#[doc(hidden)]
macro_rules! hcl_expect_expr_comma {
    ($e:expr , $($tt:tt)*) => {};
}
