//! Iterators over HCL structures.

use super::{Attribute, Block, Body, Structure};
use std::iter::FusedIterator;
use std::slice;
use std::vec;

macro_rules! impl_find_map_iterator {
    ($ty:ident$(<$lt:lifetime>)?, $item:ty, $map:expr) => {
        impl$(<$lt>)* Iterator for $ty$(<$lt>)* {
            type Item = $item;

            fn next(&mut self) -> Option<Self::Item> {
                self.iter.find_map($map)
            }
        }

        impl$(<$lt>)* DoubleEndedIterator for $ty$(<$lt>)* {
            fn next_back(&mut self) -> Option<Self::Item> {
                loop {
                    match self.iter.next_back() {
                        Some(val) => {
                            if let Some(val) = $map(val) {
                                return Some(val);
                            }
                        }
                        None => return None,
                    };
                }
            }
        }

        impl$(<$lt>)* FusedIterator for $ty$(<$lt>)* {}
    };
}

macro_rules! impl_exact_size_iterator {
    ($ty:ident$(<$lt:lifetime>)?, $item:ty) => {
        impl$(<$lt>)* Iterator for $ty$(<$lt>)* {
            type Item = $item;

            fn next(&mut self) -> Option<Self::Item> {
                self.iter.next()
            }

            fn size_hint(&self) -> (usize, Option<usize>) {
                self.iter.size_hint()
            }
        }

        impl$(<$lt>)* DoubleEndedIterator for $ty$(<$lt>)* {
            fn next_back(&mut self) -> Option<Self::Item> {
                self.iter.next_back()
            }
        }

        impl$(<$lt>)* ExactSizeIterator for $ty$(<$lt>)* {
            fn len(&self) -> usize {
                self.iter.len()
            }
        }

        impl$(<$lt>)* FusedIterator for $ty$(<$lt>)* {}
    };
}

impl<T> Extend<T> for Body
where
    T: Into<Structure>,
{
    fn extend<I>(&mut self, iterable: I)
    where
        I: IntoIterator<Item = T>,
    {
        self.0.extend(iterable.into_iter().map(Into::into));
    }
}

impl<T> FromIterator<T> for Body
where
    T: Into<Structure>,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        let iter = iter.into_iter();
        let lower = iter.size_hint().0;
        let mut body = Body(Vec::with_capacity(lower));
        body.extend(iter);
        body
    }
}

impl IntoIterator for Body {
    type Item = Structure;

    type IntoIter = IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter::new(self)
    }
}

impl<'a> IntoIterator for &'a Body {
    type Item = &'a Structure;

    type IntoIter = Iter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a> IntoIterator for &'a mut Body {
    type Item = &'a mut Structure;

    type IntoIter = IterMut<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

/// An iterator over the structures within a `Body`.
///
/// This `struct` is created by the [`iter`][Body::iter] method on [`Body`]. See its documentation
/// for more.
#[derive(Debug, Clone)]
pub struct Iter<'a> {
    iter: slice::Iter<'a, Structure>,
}

impl<'a> Iter<'a> {
    pub(super) fn new(body: &'a Body) -> Iter<'a> {
        Iter {
            iter: body.0.iter(),
        }
    }
}

impl_exact_size_iterator!(Iter<'a>, &'a Structure);

/// A mutable iterator over the structures within a `Body`.
///
/// This `struct` is created by the [`iter_mut`][Body::iter_mut] method on [`Body`]. See its
/// documentation for more.
#[derive(Debug)]
pub struct IterMut<'a> {
    iter: slice::IterMut<'a, Structure>,
}

impl<'a> IterMut<'a> {
    pub(super) fn new(body: &'a mut Body) -> IterMut<'a> {
        IterMut {
            iter: body.0.iter_mut(),
        }
    }
}

impl_exact_size_iterator!(IterMut<'a>, &'a mut Structure);

/// An owning iterator over the structures within a `Body`.
///
/// This `struct` is created by the [`into_iter`] method on [`Body`] (provided by the
/// [`IntoIterator`] trait). See its documentation for more.
///
/// [`into_iter`]: IntoIterator::into_iter
/// [`IntoIterator`]: core::iter::IntoIterator
#[derive(Debug, Clone)]
pub struct IntoIter {
    iter: vec::IntoIter<Structure>,
}

impl IntoIter {
    pub(super) fn new(body: Body) -> IntoIter {
        IntoIter {
            iter: body.0.into_iter(),
        }
    }
}

impl_exact_size_iterator!(IntoIter, Structure);

/// An iterator over the attributes within a `Body`.
///
/// This `struct` is created by the [`attributes`][Body::attributes] method on [`Body`]. See its
/// documentation for more.
#[derive(Debug, Clone)]
pub struct Attributes<'a> {
    iter: Iter<'a>,
}

impl<'a> Attributes<'a> {
    pub(super) fn new(body: &'a Body) -> Attributes<'a> {
        Attributes { iter: body.iter() }
    }
}

impl_find_map_iterator!(Attributes<'a>, &'a Attribute, Structure::as_attribute);

/// A mutable iterator over the attributes within a `Body`.
///
/// This `struct` is created by the [`attributes_mut`][Body::attributes_mut] method on [`Body`].
/// See its documentation for more.
#[derive(Debug)]
pub struct AttributesMut<'a> {
    iter: IterMut<'a>,
}

impl<'a> AttributesMut<'a> {
    pub(super) fn new(body: &'a mut Body) -> AttributesMut<'a> {
        AttributesMut {
            iter: body.iter_mut(),
        }
    }
}

impl_find_map_iterator!(
    AttributesMut<'a>,
    &'a mut Attribute,
    Structure::as_attribute_mut
);

/// An owning iterator over the attributes within a `Body`.
///
/// This `struct` is created by the [`into_attributes`][Body::into_attributes] method on [`Body`].
/// See its documentation for more.
#[derive(Debug, Clone)]
pub struct IntoAttributes {
    iter: IntoIter,
}

impl IntoAttributes {
    pub(super) fn new(body: Body) -> IntoAttributes {
        IntoAttributes {
            iter: body.into_iter(),
        }
    }
}

impl_find_map_iterator!(IntoAttributes, Attribute, Structure::into_attribute);

/// An iterator over the blocks within a `Body`.
///
/// This `struct` is created by the [`blocks`][Body::blocks] method on [`Body`]. See its
/// documentation for more.
#[derive(Debug, Clone)]
pub struct Blocks<'a> {
    iter: Iter<'a>,
}

impl<'a> Blocks<'a> {
    pub(super) fn new(body: &'a Body) -> Blocks<'a> {
        Blocks { iter: body.iter() }
    }
}

impl_find_map_iterator!(Blocks<'a>, &'a Block, Structure::as_block);

/// A mutable iterator over the blocks within a `Body`.
///
/// This `struct` is created by the [`blocks_mut`][Body::blocks_mut] method on [`Body`]. See its
/// documentation for more.
#[derive(Debug)]
pub struct BlocksMut<'a> {
    iter: IterMut<'a>,
}

impl<'a> BlocksMut<'a> {
    pub(super) fn new(body: &'a mut Body) -> BlocksMut<'a> {
        BlocksMut {
            iter: body.iter_mut(),
        }
    }
}

impl_find_map_iterator!(BlocksMut<'a>, &'a mut Block, Structure::as_block_mut);

/// An owning iterator over the blocks within a `Body`.
///
/// This `struct` is created by the [`into_blocks`][Body::into_blocks] method on [`Body`]. See its
/// documentation for more.
#[derive(Debug, Clone)]
pub struct IntoBlocks {
    iter: IntoIter,
}

impl IntoBlocks {
    pub(super) fn new(body: Body) -> IntoBlocks {
        IntoBlocks {
            iter: body.into_iter(),
        }
    }
}

impl_find_map_iterator!(IntoBlocks, Block, Structure::into_block);
