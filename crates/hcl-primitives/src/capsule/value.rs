use core::{any::Any, fmt};
use dyn_clone::DynClone;
use dyn_std::{any::Dyn, cmp::PartialEq as DynPartialEq};

/// A trait for opaque encapsulated values.
///
/// Types that implement `Clone`, `PartialEq`, `Eq` and `Any` automatically implement
/// `CapsuleValue`.
pub trait CapsuleValue: DynClone + DynPartialEq + Dyn {}

impl dyn CapsuleValue {
    pub(super) fn is<T: 'static>(&self) -> bool {
        Dyn::as_any(self).is::<T>()
    }

    pub(super) fn downcast<T: 'static>(self: Box<Self>) -> Result<Box<T>, Box<Self>> {
        if self.is::<T>() {
            Ok(Dyn::as_any_box(self).downcast::<T>().unwrap())
        } else {
            Err(self)
        }
    }

    pub(super) fn downcast_ref<T: 'static>(&self) -> Option<&T> {
        Dyn::as_any(self).downcast_ref::<T>()
    }

    pub(super) fn downcast_mut<T: 'static>(&mut self) -> Option<&mut T> {
        Dyn::as_any_mut(self).downcast_mut::<T>()
    }
}

impl Clone for Box<dyn CapsuleValue> {
    fn clone(&self) -> Self {
        dyn_clone::clone_box(&**self)
    }
}

impl PartialEq for dyn CapsuleValue {
    fn eq(&self, other: &Self) -> bool {
        DynPartialEq::dyn_eq(self, Dyn::as_any(other))
    }
}

impl PartialEq<&Self> for Box<dyn CapsuleValue> {
    fn eq(&self, other: &&Self) -> bool {
        DynPartialEq::dyn_eq(self, Dyn::as_any(*other))
    }
}

impl Eq for Box<dyn CapsuleValue> {}

impl fmt::Debug for dyn CapsuleValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("CapsuleValue")
    }
}

impl<T> CapsuleValue for T where T: Clone + PartialEq + Eq + Any {}
