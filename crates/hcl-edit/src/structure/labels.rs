//! The block label matching API.

use crate::structure::BlockLabel;

/// The type returned by the `into_prefix_matcher` method of the [`Matcher`] trait.
#[derive(Debug, Clone, Copy)]
pub struct PrefixMatcher<T>(T);

/// The type returned by the `into_suffix_matcher` method of the [`Matcher`] trait.
#[derive(Debug, Clone, Copy)]
pub struct SuffixMatcher<T>(T);

/// Matches blocks with no labels.
#[derive(Debug, Clone, Copy)]
pub struct UnlabeledMatcher;

/// A trait that can be implemented to control matching behaviour for blocks having a certain set
/// of labels.
pub trait Matcher: Sized {
    /// Converts into a `Matcher` which matches the label prefix.
    fn into_prefix_matcher(self) -> PrefixMatcher<Self>
    where
        Self: Pattern,
    {
        PrefixMatcher(self)
    }

    /// Converts into a `Matcher` which matches the label suffix.
    fn into_suffix_matcher(self) -> SuffixMatcher<Self>
    where
        Self: Pattern,
    {
        SuffixMatcher(self)
    }

    /// Returns `true` if the pattern matches the `labels` slice.
    fn matches_labels(self, labels: &[BlockLabel]) -> bool;
}

/// A trait that can be implemented to control matching behaviour for blocks having a certain set
/// of labels.
#[allow(clippy::wrong_self_convention)]
pub trait Pattern: Sized {
    /// Returns `true` if the pattern matches the prefix of the `labels` slice.
    fn is_prefix_of(self, labels: &[BlockLabel]) -> bool;

    /// Returns `true` if the pattern matches the suffix of the `labels` slice.
    fn is_suffix_of(self, labels: &[BlockLabel]) -> bool;

    /// Returns `true` if the pattern matches the `labels` slice.
    fn is_exact_match(self, labels: &[BlockLabel]) -> bool;
}

impl<T> Matcher for PrefixMatcher<T>
where
    T: Pattern,
{
    fn matches_labels(self, labels: &[BlockLabel]) -> bool {
        self.0.is_prefix_of(labels)
    }
}

impl<T> Matcher for SuffixMatcher<T>
where
    T: Pattern,
{
    fn matches_labels(self, labels: &[BlockLabel]) -> bool {
        self.0.is_suffix_of(labels)
    }
}

impl Matcher for UnlabeledMatcher {
    fn matches_labels(self, labels: &[BlockLabel]) -> bool {
        labels.is_empty()
    }
}

impl<'a> Matcher for &'a str {
    fn matches_labels(self, labels: &[BlockLabel]) -> bool {
        self.is_exact_match(labels)
    }
}

impl<'a> Matcher for &'a BlockLabel {
    fn matches_labels(self, labels: &[BlockLabel]) -> bool {
        self.is_exact_match(labels)
    }
}

impl<'a, T> Matcher for &'a [T]
where
    T: AsRef<str>,
{
    fn matches_labels(self, labels: &[BlockLabel]) -> bool {
        self.is_exact_match(labels)
    }
}

impl<'a, T, const N: usize> Matcher for &'a [T; N]
where
    T: AsRef<str>,
{
    fn matches_labels(self, labels: &[BlockLabel]) -> bool {
        self.is_exact_match(labels)
    }
}

impl<F> Matcher for F
where
    F: FnMut(&[BlockLabel]) -> bool,
{
    fn matches_labels(mut self, labels: &[BlockLabel]) -> bool {
        (self)(labels)
    }
}

impl<'a> Pattern for &'a str {
    fn is_prefix_of(self, labels: &[BlockLabel]) -> bool {
        labels.first().map_or(false, |label| label == self)
    }

    fn is_suffix_of(self, labels: &[BlockLabel]) -> bool {
        labels.last().map_or(false, |label| label == self)
    }

    fn is_exact_match(self, labels: &[BlockLabel]) -> bool {
        labels.len() == 1 && self == &labels[0]
    }
}

impl<'a> Pattern for &'a BlockLabel {
    fn is_prefix_of(self, labels: &[BlockLabel]) -> bool {
        self.as_str().is_prefix_of(labels)
    }

    fn is_suffix_of(self, labels: &[BlockLabel]) -> bool {
        self.as_str().is_suffix_of(labels)
    }

    fn is_exact_match(self, labels: &[BlockLabel]) -> bool {
        self.as_str().is_exact_match(labels)
    }
}

impl<'a, T> Pattern for &'a [T]
where
    T: AsRef<str>,
{
    fn is_prefix_of(self, labels: &[BlockLabel]) -> bool {
        self.len() <= labels.len() && self.iter().zip(labels).all(|(a, b)| a.as_ref() == b)
    }

    fn is_suffix_of(self, labels: &[BlockLabel]) -> bool {
        self.len() <= labels.len()
            && self
                .iter()
                .rev()
                .zip(labels.iter().rev())
                .all(|(a, b)| a.as_ref() == b)
    }

    fn is_exact_match(self, labels: &[BlockLabel]) -> bool {
        self.len() == labels.len() && self.iter().zip(labels).all(|(a, b)| a.as_ref() == b)
    }
}

impl<'a, T, const N: usize> Pattern for &'a [T; N]
where
    T: AsRef<str>,
{
    fn is_prefix_of(self, labels: &[BlockLabel]) -> bool {
        self[..].is_prefix_of(labels)
    }

    fn is_suffix_of(self, labels: &[BlockLabel]) -> bool {
        self[..].is_suffix_of(labels)
    }

    fn is_exact_match(self, labels: &[BlockLabel]) -> bool {
        self[..].is_exact_match(labels)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! label {
        ($expr:expr) => {
            BlockLabel::from($expr)
        };
    }

    macro_rules! labels {
        ($($label:expr),* $(,)?) => {
            &[$(label!($label),)*]
        };
    }

    #[test]
    fn unlabeled() {
        assert!(UnlabeledMatcher.matches_labels(labels!()));
        assert!(!UnlabeledMatcher.matches_labels(labels!("label")));
    }

    #[test]
    fn str() {
        let (foo, bar) = ("foo", "bar");
        assert!(!foo.matches_labels(&[]));
        assert!(foo.matches_labels(labels!("foo")));
        assert!(!foo.matches_labels(labels!("foo", "bar")));

        assert!(foo
            .into_prefix_matcher()
            .matches_labels(labels!("foo", "bar")));
        assert!(!foo
            .into_suffix_matcher()
            .matches_labels(labels!("foo", "bar")));

        assert!(!bar
            .into_prefix_matcher()
            .matches_labels(labels!("foo", "bar")));
        assert!(bar
            .into_suffix_matcher()
            .matches_labels(labels!("foo", "bar")));
    }

    #[test]
    fn block_label() {
        let (foo, bar) = (label!("foo"), label!("bar"));
        assert!(!foo.matches_labels(labels!()));
        assert!(foo.matches_labels(labels!("foo")));
        assert!(!foo.matches_labels(labels!("foo", "bar")));

        assert!(foo
            .into_prefix_matcher()
            .matches_labels(labels!("foo", "bar")));
        assert!(!foo
            .into_suffix_matcher()
            .matches_labels(labels!("foo", "bar")));

        assert!(!bar
            .into_prefix_matcher()
            .matches_labels(labels!("foo", "bar")));
        assert!(bar
            .into_suffix_matcher()
            .matches_labels(labels!("foo", "bar")));
    }

    #[test]
    fn slice() {
        let foo_bar = ["foo", "bar"];
        assert!(!foo_bar.matches_labels(labels!()));
        assert!(!foo_bar.matches_labels(labels!("foo")));
        assert!(foo_bar.matches_labels(labels!("foo", "bar")));
        assert!(!foo_bar.matches_labels(labels!("foo", "bar", "baz")));

        let foo_bar = labels!("foo", "bar");
        assert!(!foo_bar.matches_labels(labels!()));
        assert!(!foo_bar.matches_labels(labels!("foo")));
        assert!(foo_bar.matches_labels(labels!("foo", "bar")));
        assert!(!foo_bar.matches_labels(labels!("foo", "bar", "baz")));

        assert!(foo_bar
            .into_prefix_matcher()
            .matches_labels(labels!("foo", "bar", "baz")));
        assert!(!labels!("foo", "bar", "baz")
            .into_prefix_matcher()
            .matches_labels(&[label!("foo"), label!("bar")]));

        assert!(labels!("bar", "baz")
            .into_suffix_matcher()
            .matches_labels(labels!("foo", "bar", "baz")));
        assert!(!labels!("foo", "bar", "baz")
            .into_suffix_matcher()
            .matches_labels(labels!("bar", "baz")));
    }

    #[test]
    fn func() {
        fn f(labels: &[BlockLabel]) -> bool {
            labels.iter().any(|label| label.starts_with("f"))
        }

        assert!(!f.matches_labels(labels!()));
        assert!(f.matches_labels(labels!("foo")));
        assert!(!f.matches_labels(labels!("bar")));
        assert!(f.matches_labels(labels!("foo", "bar")));
        assert!(f.matches_labels(labels!("bar", "foo")));
        assert!(!f.matches_labels(labels!("bar", "baz")));
    }
}
