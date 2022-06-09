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
/// let block_identifier = "resource";
/// let attribute_key = "name";
///
/// let body = hcl::body!({
///     (block_identifier) "aws_sns_topic" "topic" {
///         (attribute_key) = "my-topic"
///     }
/// });
///
/// let expected = hcl::Body::builder()
///     .add_block(
///         hcl::Block::builder(block_identifier)
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
    // Empty body.
    () => {
        $crate::Body::default()
    };

    // Body in braces.
    ({$($rest:tt)*}) => {
        $crate::body!($($rest)*)
    };

    // Consumes all tokens and adds matched attributes and blocks to the body builder.
    ($($rest:tt)*) => {
        {
            let mut builder = $crate::Body::builder();
            $crate::body_internal!(@any builder $($rest)*);
            builder.build()
        }
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! body_internal {
    // Add attribute to the builder and consume remaining structures.
    (@attr $builder:ident ($attr:expr) ($($rest:tt)*)) => {
        $builder = $builder.add_attribute($attr);
        $crate::body_internal!(@any $builder $($rest)*);
    };

    // Add block to the builder and consume remaining structures.
    (@block $builder:ident ($block:expr) ($($rest:tt)*)) => {
        $builder = $builder.add_block($block);
        $crate::body_internal!(@any $builder $($rest)*);
    };

    // Consume attribute.
    (@any $builder:ident $key:tt = $expr:tt $($rest:tt)*) => {
        $crate::body_internal!(@attr $builder ($crate::attr!($key = $expr)) ($($rest)*))
    };

    // Consume block with with identifiers as labels.
    (@any $builder:ident $ident:ident $($label:ident)* {$($body:tt)*} $($rest:tt)*) => {
        $crate::body_internal!(@block $builder ($crate::block!($ident $($label)* {$($body)*})) ($($rest)*))
    };

    // Consume block with with literals as labels.
    (@any $builder:ident $ident:ident $($label:literal)* {$($body:tt)*} $($rest:tt)*) => {
        $crate::body_internal!(@block $builder ($crate::block!($ident $($label)* {$($body)*})) ($($rest)*))
    };

    // Consume block labels from expressions.
    (@any $builder:ident $ident:ident $(($label:expr))* {$($body:tt)*} $($rest:tt)*) => {
        $crate::body_internal!(@block $builder ($crate::block!($ident $(($label))* {$($body)*})) ($($rest)*))
    };

    // Consume block with identifier from expression and identifiers as labels.
    (@any $builder:ident ($ident:expr) $($label:ident)* {$($body:tt)*} $($rest:tt)*) => {
        $crate::body_internal!(@block $builder ($crate::block!(($ident) $($label)* {$($body)*})) ($($rest)*))
    };

    // Consume block with identifier from expression and literals as labels.
    (@any $builder:ident ($ident:expr) $($label:literal)* {$($body:tt)*} $($rest:tt)*) => {
        $crate::body_internal!(@block $builder ($crate::block!(($ident) $($label)* {$($body)*})) ($($rest)*))
    };

    // Consume block with identifier from expression and labels from expressions.
    (@any $builder:ident ($ident:expr) $(($label:expr))* {$($body:tt)*} $($rest:tt)*) => {
        $crate::body_internal!(@block $builder ($crate::block!(($ident) $(($label))* {$($body)*})) ($($rest)*))
    };

    // Done, no more tokens to consume.
    (@any $builder:ident) => {};
}

/// Construct an `hcl::Structure` which may be either an HCL attribute or block.
#[macro_export]
#[doc(hidden)]
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
        $crate::structure!($($structure)+)
    };

    // An attribute structure.
    ($key:tt = $($expr:tt)+) => {
        $crate::Structure::Attribute($crate::attr!($key = $($expr)+))
    };

    // A block structure.
    ($($block:tt)+) => {
        $crate::Structure::Block($crate::block!($($block)+))
    };
}

/// Construct an `hcl::Attribute` from a key and a value expression.
#[macro_export]
#[doc(hidden)]
macro_rules! attr {
    // Attribute in braces.
    ({ $($rest:tt)+ }) => {
        $crate::attr!($($rest)+)
    };

    // Attribute with identifier as key.
    ($key:ident = $($expr:tt)+) => {
        $crate::Attribute {
            key: std::stringify!($key).to_owned(),
            expr: $crate::expr!($($expr)+),
        }
    };

    // Attribute with expression as key.
    (($key:expr) = $($expr:tt)+) => {
        $crate::Attribute {
            key: ($key).into(),
            expr: $crate::expr!($($expr)+),
        }
    };
}

/// Construct an `hcl::Block` from a block identifier, optional block labels and a block body.
#[macro_export]
#[doc(hidden)]
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
    ({ $($tt:tt)+ }) => {
        $crate::block_internal!($($tt)+)
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
#[macro_export]
#[doc(hidden)]
macro_rules! expr {
    // Hide distracting implementation details from the generated rustdoc.
    ($($expr:tt)+) => {
        $crate::expr_internal!($($expr)+)
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! expr_internal {
    //////////////////////////////////////////////////////////////////////////
    // TT muncher for parsing the inside of an array [...]. Produces a vec![...]
    // of the elements.
    //
    // Must be invoked as: expr_internal!(@array [] $($tt)*)
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
        $crate::expr_internal!(@array [$($elems,)* $crate::expr_internal!(null)] $($rest)*)
    };

    // Next element is `true`.
    (@array [$($elems:expr,)*] true $($rest:tt)*) => {
        $crate::expr_internal!(@array [$($elems,)* $crate::expr_internal!(true)] $($rest)*)
    };

    // Next element is `false`.
    (@array [$($elems:expr,)*] false $($rest:tt)*) => {
        $crate::expr_internal!(@array [$($elems,)* $crate::expr_internal!(false)] $($rest)*)
    };

    // Next element is an array.
    (@array [$($elems:expr,)*] [$($array:tt)*] $($rest:tt)*) => {
        $crate::expr_internal!(@array [$($elems,)* $crate::expr_internal!([$($array)*])] $($rest)*)
    };

    // Next element is a map.
    (@array [$($elems:expr,)*] {$($map:tt)*} $($rest:tt)*) => {
        $crate::expr_internal!(@array [$($elems,)* $crate::expr_internal!({$($map)*})] $($rest)*)
    };

    // Next element is an expression followed by comma.
    (@array [$($elems:expr,)*] $next:expr, $($rest:tt)*) => {
        $crate::expr_internal!(@array [$($elems,)* $crate::expr_internal!($next),] $($rest)*)
    };

    // Last element is an expression with no trailing comma.
    (@array [$($elems:expr,)*] $last:expr) => {
        $crate::expr_internal!(@array [$($elems,)* $crate::expr_internal!($last)])
    };

    // Comma after the most recent element.
    (@array [$($elems:expr),*] , $($rest:tt)*) => {
        $crate::expr_internal!(@array [$($elems,)*] $($rest)*)
    };

    // Unexpected token after most recent element.
    (@array [$($elems:expr),*] $unexpected:tt $($rest:tt)*) => {
        $crate::expr_unexpected!($unexpected)
    };

    //////////////////////////////////////////////////////////////////////////
    // TT muncher for parsing the inside of an object {...}. Each entry is
    // inserted into the given map variable.
    //
    // Must be invoked as: expr_internal!(@object $map () ($($tt)*) ($($tt)*))
    //
    // We require two copies of the input tokens so that we can match on one
    // copy and trigger errors on the other copy.
    //////////////////////////////////////////////////////////////////////////

    // Done.
    (@object $object:ident () () ()) => {};

    // Insert the current entry followed by trailing comma.
    (@object $object:ident [$($key:tt)+] ($value:expr) , $($rest:tt)*) => {
        let _ = $object.insert(($($key)+).into(), $value);
        $crate::expr_internal!(@object $object () ($($rest)*) ($($rest)*));
    };

    // Current entry followed by unexpected token.
    (@object $object:ident [$($key:tt)+] ($value:expr) $unexpected:tt $($rest:tt)*) => {
        $crate::expr_unexpected!($unexpected);
    };

    // Insert the last entry without trailing comma.
    (@object $object:ident [$($key:tt)+] ($value:expr)) => {
        let _ = $object.insert(($($key)+).into(), $value);
    };

    // Next value is `null`.
    (@object $object:ident ($($key:tt)+) (= null $($rest:tt)*) $copy:tt) => {
        $crate::expr_internal!(@object $object [$($key)+] ($crate::expr_internal!(null)) $($rest)*);
    };

    // Next value is `true`.
    (@object $object:ident ($($key:tt)+) (= true $($rest:tt)*) $copy:tt) => {
        $crate::expr_internal!(@object $object [$($key)+] ($crate::expr_internal!(true)) $($rest)*);
    };

    // Next value is `false`.
    (@object $object:ident ($($key:tt)+) (= false $($rest:tt)*) $copy:tt) => {
        $crate::expr_internal!(@object $object [$($key)+] ($crate::expr_internal!(false)) $($rest)*);
    };

    // Next value is an array.
    (@object $object:ident ($($key:tt)+) (= [$($array:tt)*] $($rest:tt)*) $copy:tt) => {
        $crate::expr_internal!(@object $object [$($key)+] ($crate::expr_internal!([$($array)*])) $($rest)*);
    };

    // Next value is a map.
    (@object $object:ident ($($key:tt)+) (= {$($map:tt)*} $($rest:tt)*) $copy:tt) => {
        $crate::expr_internal!(@object $object [$($key)+] ($crate::expr_internal!({$($map)*})) $($rest)*);
    };

    // Next value is an expression followed by comma.
    (@object $object:ident ($($key:tt)+) (= $value:expr , $($rest:tt)*) $copy:tt) => {
        $crate::expr_internal!(@object $object [$($key)+] ($crate::expr_internal!($value)) , $($rest)*);
    };

    // Last value is an expression with no trailing comma.
    (@object $object:ident ($($key:tt)+) (= $value:expr) $copy:tt) => {
        $crate::expr_internal!(@object $object [$($key)+] ($crate::expr_internal!($value)));
    };

    // Missing value for last entry. Trigger a reasonable error message.
    (@object $object:ident ($($key:tt)+) (=) $copy:tt) => {
        // "unexpected end of macro invocation"
        $crate::expr_internal!();
    };

    // Missing equals and value for last entry. Trigger a reasonable error
    // message.
    (@object $object:ident ($($key:tt)+) () $copy:tt) => {
        // "unexpected end of macro invocation"
        $crate::expr_internal!();
    };

    // Misplaced equals. Trigger a reasonable error message.
    (@object $object:ident () (= $($rest:tt)*) ($equals:tt $($copy:tt)*)) => {
        // Takes no arguments so "no rules expected the token `:`".
        $crate::expr_unexpected!($equals);
    };

    // Found a comma inside a key. Trigger a reasonable error message.
    (@object $object:ident ($($key:tt)*) (, $($rest:tt)*) ($comma:tt $($copy:tt)*)) => {
        // Takes no arguments so "no rules expected the token `,`".
        $crate::expr_unexpected!($comma);
    };

    // Key is fully parenthesized. This avoids clippy double_parens false
    // positives because the parenthesization may be necessary here.
    (@object $object:ident () (($key:expr) = $($rest:tt)*) $copy:tt) => {
        $crate::expr_internal!(@object $object ($key) (= $($rest)*) (= $($rest)*));
    };

    // Refuse to absorb equals token into key expression.
    (@object $object:ident ($($key:tt)*) (= $($unexpected:tt)+) $copy:tt) => {
        $crate::expr_expect_expr_comma!($($unexpected)+);
    };

    // Munch a token into the current key.
    (@object $object:ident ($($key:tt)*) ($tt:tt $($rest:tt)*) $copy:tt) => {
        $crate::expr_internal!(@object $object ($($key)* $tt) ($($rest)*) ($($rest)*));
    };

    //////////////////////////////////////////////////////////////////////////
    // The main implementation.
    //
    // Must be invoked as: expr_internal!($($expr)+)
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
        $crate::Expression::Array($crate::expr_internal!(@array [] $($tt)+))
    };

    ({}) => {
        $crate::Expression::Object($crate::Object::new())
    };

    ({ $($tt:tt)+ }) => {
        $crate::Expression::Object({
            let mut object = $crate::Object::new();
            $crate::expr_internal!(@object object () ($($tt)+) ($($tt)+));
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
macro_rules! expr_unexpected {
    () => {};
}

#[macro_export]
#[doc(hidden)]
macro_rules! expr_expect_expr_comma {
    ($e:expr , $($tt:tt)*) => {};
}
