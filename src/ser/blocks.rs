use super::in_internal_serialization;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::ops;
use vecmap::VecMap;

pub(crate) const BLOCK_MARKER: &str = "$hcl::Block";

/// A transparent wrapper type which hints the [`Serializer`][crate::ser::Serializer] to serialize
/// `T` as an HCL block.
///
/// When passed to a serializer other than the one from this crate, a `Block<T>` serializes
/// exactly like `T`, if `T` implements `serde::Serialize`.
///
/// A `Block<T>` can only be used in the *value position of a map-like structure*. For example:
///
/// - It can be used to wrap the *value type of a map*, e.g. `Map<K, Block<T>>`
/// - As the value of a *struct field*, e.g. `struct S { field: Block<T> }`
/// - Or as the value of an *enum variant*, e.g. `enum E { Variant(Block<T>) }`
///
/// **The serialized block's identifier will be the respective map key, struct field name or variant
/// name.**
///
/// The wrapped `T` must be shaped as follows to be serialized as an HCL block:
///
/// - A *map-like* value (e.g. a map or struct).
/// - A *sequence-like* value (e.g. a vector, slice or tuple) with map-like elements as described
///   above. In this case, multiple blocks with the same identifier are produced.
///
/// Wrapping a type `T` that does not fulfil one of the criteria above in a `Block<T>` will result
/// in serialization errors.
///
/// For more convenient usage, see the [`block`][crate::ser::block] function.
///
/// # Example
///
/// ```
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use hcl::ser::Block;
/// use serde::Serialize;
///
/// #[derive(Serialize)]
/// struct Config {
///     user: Block<Vec<User>>,
/// }
///
/// #[derive(Serialize)]
/// struct User {
///     name: String,
///     email: String,
/// }
///
/// let users = vec![
///     User {
///         name: "john".into(),
///         email: "johndoe@example.com".into(),
///     },
///     User {
///         name: "jane".into(),
///         email: "janedoe@example.com".into(),
///     },
/// ];
///
/// let config = Config {
///     user: Block::new(users),
/// };
///
/// let expected = r#"
/// user {
///   name = "john"
///   email = "johndoe@example.com"
/// }
///
/// user {
///   name = "jane"
///   email = "janedoe@example.com"
/// }
/// "#.trim_start();
///
/// assert_eq!(hcl::to_string(&config)?, expected);
/// #    Ok(())
/// # }
/// ```
pub struct Block<T>(T);

impl<T> Block<T> {
    /// Create a new `Block<T>` from a `T`.
    pub fn new(value: T) -> Block<T> {
        Block(value)
    }

    /// Consume the `Block` and return the wrapped `T`.
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> ops::Deref for Block<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> ops::DerefMut for Block<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> Clone for Block<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        Block(self.0.clone())
    }
}

impl<T> fmt::Debug for Block<T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Block").field(&self.0).finish()
    }
}

impl<T> Serialize for Block<T>
where
    T: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if in_internal_serialization() {
            serializer.serialize_newtype_struct(BLOCK_MARKER, &self.0)
        } else {
            self.0.serialize(serializer)
        }
    }
}

impl<'de, T> Deserialize<'de> for Block<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        T::deserialize(deserializer).map(Block)
    }
}

pub(crate) const LABELED_BLOCK_MARKER: &str = "$hcl::LabeledBlock";

/// A transparent wrapper type which hints the [`Serializer`][crate::ser::Serializer] to serialize
/// `T` as a labeled HCL block.
///
/// When passed to a serializer other than the one from this crate, a `LabeledBlock<T>` serializes
/// exactly like `T`, if `T` implements `serde::Serialize`.
///
/// A `LabeledBlock<T>` can only be used in the *value position of a map-like structure*. For example:
///
/// - It can be used to wrap the *value type of a map*, e.g. `Map<K, LabeledBlock<T>>`
/// - As the value of a *struct field*, e.g. `struct S { field: LabeledBlock<T> }`
/// - Or as the value of an *enum variant*, e.g. `enum E { Variant(LabeledBlock<T>) }`
///
/// **The serialized block's identifier will be the respective map key, struct field name or variant
/// name.**
///
/// The wrapped `T` must be shaped as follows to be serialized as a labeled HCL block:
///
/// - A *map-like* value (e.g. a map or struct) where the value may to be another
///   `LabeledBlock<T>`, in which case a block with multiple labels is produced. Can be nested
///   arbitrarily deep to allow for any number of block labels.
/// - A *sequence-like* value (e.g. a vector, slice or tuple) with map-like elements as described
///   above. In this case, multiple blocks with the same identifier and labels are produced.
///
/// Wrapping a type `T` that does not fulfil one of the criteria above in a [`LabeledBlock<T>`]
/// will result in serialization errors.
///
/// For more convenient usage, see the [`labeled_block`] and [`doubly_labeled_block`] functions.
///
/// # Example
///
/// ```
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use hcl::ser::LabeledBlock;
/// use indexmap::{indexmap, IndexMap};
/// use serde::Serialize;
///
/// #[derive(Serialize)]
/// struct Config {
///     user: LabeledBlock<IndexMap<String, User>>,
/// }
///
/// #[derive(Serialize)]
/// struct User {
///     email: String,
/// }
///
/// let users = indexmap! {
///     "john".into() => User {
///         email: "johndoe@example.com".into(),
///     },
///     "jane".into() => User {
///         email: "janedoe@example.com".into(),
///     },
/// };
///
/// let config = Config {
///     user: LabeledBlock::new(users),
/// };
///
/// let expected = r#"
/// user "john" {
///   email = "johndoe@example.com"
/// }
///
/// user "jane" {
///   email = "janedoe@example.com"
/// }
/// "#.trim_start();
///
/// assert_eq!(hcl::to_string(&config)?, expected);
/// #    Ok(())
/// # }
/// ```
pub struct LabeledBlock<T>(T);

impl<T> LabeledBlock<T> {
    /// Create a new `LabeledBlock<T>` from a `T`.
    pub fn new(value: T) -> LabeledBlock<T> {
        LabeledBlock(value)
    }

    /// Consume the `LabeledBlock` and return the wrapped `T`.
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> ops::Deref for LabeledBlock<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> ops::DerefMut for LabeledBlock<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> Clone for LabeledBlock<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        LabeledBlock(self.0.clone())
    }
}

impl<T> fmt::Debug for LabeledBlock<T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("LabeledBlock").field(&self.0).finish()
    }
}

impl<T> Serialize for LabeledBlock<T>
where
    T: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if in_internal_serialization() {
            serializer.serialize_newtype_struct(LABELED_BLOCK_MARKER, &self.0)
        } else {
            self.0.serialize(serializer)
        }
    }
}

impl<'de, T> Deserialize<'de> for LabeledBlock<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        T::deserialize(deserializer).map(LabeledBlock)
    }
}

/// Hints the [`Serializer`][crate::ser::Serializer] to serialize `T` as an HCL block.
///
/// This function is intended to be used in the `#[serde(serialize_with)]` attribute and wraps `T`
/// with a [`Block<T>`].
///
/// See the type-level documentation of [`Block<T>`] for more.
///
/// # Example
///
/// ```
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use serde::Serialize;
///
/// #[derive(Serialize)]
/// struct Config {
///     #[serde(serialize_with = "hcl::ser::block")]
///     user: Vec<User>,
/// }
///
/// #[derive(Serialize)]
/// struct User {
///     name: String,
///     email: String,
/// }
///
/// let config = Config {
///     user: vec![
///         User {
///             name: "john".into(),
///             email: "johndoe@example.com".into(),
///         },
///         User {
///             name: "jane".into(),
///             email: "janedoe@example.com".into(),
///         },
///     ],
/// };
///
/// let expected = r#"
/// user {
///   name = "john"
///   email = "johndoe@example.com"
/// }
///
/// user {
///   name = "jane"
///   email = "janedoe@example.com"
/// }
/// "#.trim_start();
///
/// assert_eq!(hcl::to_string(&config)?, expected);
/// #    Ok(())
/// # }
/// ```
///
/// # Errors
///
/// Serialization fails if the type's shape makes it impossible to represent it as an HCL block
/// with two labels.
pub fn block<T, S>(value: T, serializer: S) -> Result<S::Ok, S::Error>
where
    T: Serialize,
    S: serde::Serializer,
{
    Block::new(value).serialize(serializer)
}

/// Hints the [`Serializer`][crate::ser::Serializer] to serialize `T` as a labeled HCL block.
///
/// This function is intended to be used in the `#[serde(serialize_with)]` attribute and wraps `T`
/// with a [`LabeledBlock<T>`].
///
/// See the type-level documentation of [`LabeledBlock<T>`] for more.
///
/// # Example
///
/// ```
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use indexmap::{indexmap, IndexMap};
/// use serde::Serialize;
///
/// #[derive(Serialize)]
/// struct Config {
///     #[serde(serialize_with = "hcl::ser::labeled_block")]
///     user: IndexMap<String, User>,
/// }
///
/// #[derive(Serialize)]
/// struct User {
///     email: String,
/// }
///
/// let config = Config {
///     user: indexmap! {
///         "john".into() => User {
///             email: "johndoe@example.com".into(),
///         },
///         "jane".into() => User {
///             email: "janedoe@example.com".into(),
///         },
///     },
/// };
///
/// let expected = r#"
/// user "john" {
///   email = "johndoe@example.com"
/// }
///
/// user "jane" {
///   email = "janedoe@example.com"
/// }
/// "#.trim_start();
///
/// assert_eq!(hcl::to_string(&config)?, expected);
/// #    Ok(())
/// # }
/// ```
///
/// # Errors
///
/// Serialization fails if the type's shape makes it impossible to represent it as a labeled HCL
/// block.
pub fn labeled_block<T, S>(value: T, serializer: S) -> Result<S::Ok, S::Error>
where
    T: Serialize,
    S: serde::Serializer,
{
    LabeledBlock::new(value).serialize(serializer)
}

/// Hints the [`Serializer`][crate::ser::Serializer] to serialize `T` as an HCL block with two
/// labels.
///
/// This function is intended to be used in the `#[serde(serialize_with)]` attribute and wraps `T`
/// and each value of `T` with a [`LabeledBlock<T>`]. One use case for this function is terraform
/// configuration where blocks with two labels are common in various places.
///
/// See the type-level documentation of [`LabeledBlock<T>`] for more.
///
/// # Example
///
/// The following example shows a very simplified and incomplete way to serialize terraform
/// configuration.
///
/// ```
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use indexmap::{indexmap, IndexMap};
/// use serde::Serialize;
///
/// #[derive(Serialize)]
/// struct Config {
///     #[serde(serialize_with = "hcl::ser::doubly_labeled_block")]
///     resource: IndexMap<String, IndexMap<String, IndexMap<String, String>>>,
/// }
///
/// let config = Config {
///     resource: indexmap! {
///         "aws_sns_topic".into() => indexmap! {
///             "mytopic".into() => indexmap! {
///                 "name".into() => "mytopic".into(),
///             },
///         },
///     },
/// };
///
/// let expected = r#"
/// resource "aws_sns_topic" "mytopic" {
///   name = "mytopic"
/// }
/// "#.trim_start();
///
/// assert_eq!(hcl::to_string(&config)?, expected);
/// #    Ok(())
/// # }
/// ```
///
/// # Errors
///
/// Serialization fails if the type's shape makes it impossible to represent it as an HCL block
/// with two labels.
pub fn doubly_labeled_block<T, K, V, S>(value: T, serializer: S) -> Result<S::Ok, S::Error>
where
    T: IntoIterator<Item = (K, V)>,
    K: Serialize + Eq,
    V: Serialize,
    S: serde::Serializer,
{
    let value: VecMap<K, LabeledBlock<V>> = value
        .into_iter()
        .map(|(k, v)| (k, LabeledBlock::new(v)))
        .collect();
    labeled_block(value, serializer)
}
