//! Support for HCL capsule types.
use core::{any::Any, fmt};
use dyn_clone::DynClone;
use dyn_std::{any::Dyn, cmp::PartialEq as DynPartialEq};

/// A trait for opaque encapsulated values.
///
/// Types that implement `Clone`, `PartialEq`, `Eq` and `Any` automatically implement
/// `CapsuleValue`.
pub trait CapsuleValue: DynClone + DynPartialEq + Dyn {}

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

impl<T> CapsuleValue for T where T: Clone + PartialEq + Eq + Any {}

/// A Capsule wraps values of custom types defined by the calling application.
///
/// A value wrapped in a capsule is considered opaque to HCL, but may be accepted by functions
/// provided by the calling application.
#[derive(Clone, PartialEq, Eq)]
pub struct Capsule(Box<dyn CapsuleValue>);

impl Capsule {
    /// Creates a new Capsule for a value of type `T`.
    pub fn new<T: CapsuleValue>(value: T) -> Capsule {
        Capsule(Box::new(value))
    }

    /// Returns `true` if the inner type is the same as `T`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use hcl_primitives::capsule::Capsule;
    ///
    /// let v = Capsule::new(1u32);
    ///
    /// assert!(v.is::<u32>());
    /// assert!(!v.is::<String>());
    /// ```
    pub fn is<T: Any>(&self) -> bool {
        Dyn::as_any(&*self.0).is::<T>()
    }

    /// Attempts to downcast the Capsule's value to a concrete type.
    ///
    /// # Examples
    ///
    /// ```
    /// # use hcl_primitives::capsule::Capsule;
    ///
    /// fn print_if_string(capsule: Capsule) {
    ///     if let Ok(string) = capsule.downcast::<String>() {
    ///         println!("String ({}): {}", string.len(), string);
    ///     }
    /// }
    ///
    /// let my_string = "Hello World".to_string();
    /// print_if_string(Capsule::new(my_string));
    /// print_if_string(Capsule::new(0i8));
    /// ```
    ///
    /// # Errors
    ///
    /// If a downcast into `T` is not possible, the original Capsule is returned via `Result`'s
    /// `Err` variant.
    pub fn downcast<T: Any>(self) -> Result<T, Capsule> {
        if self.is::<T>() {
            let boxed = Dyn::as_any_box(self.0).downcast().unwrap();
            Ok(*boxed)
        } else {
            Err(self)
        }
    }

    /// Returns some reference to the inner value if it is of type `T`, or
    /// `None` if it isn't.
    ///
    /// # Examples
    ///
    /// ```
    /// # use hcl_primitives::capsule::Capsule;
    ///
    /// fn print_if_string(capsule: &Capsule) {
    ///     if let Some(string) = capsule.downcast_ref::<String>() {
    ///         println!("It's a string({}): '{}'", string.len(), string);
    ///     } else {
    ///         println!("Not a string...");
    ///     }
    /// }
    ///
    /// print_if_string(&Capsule::new(0));
    /// print_if_string(&Capsule::new("cookie monster".to_string()));
    /// ```
    pub fn downcast_ref<T: Any>(&self) -> Option<&T> {
        Dyn::as_any(&*self.0).downcast_ref()
    }

    /// Returns some mutable reference to the inner value if it is of type `T`, or
    /// `None` if it isn't.
    ///
    /// # Examples
    ///
    /// ```
    /// # use hcl_primitives::capsule::Capsule;
    ///
    /// fn modify_if_u32(capsule: &mut Capsule) {
    ///     if let Some(num) = capsule.downcast_mut::<u32>() {
    ///         *num = 42u32;
    ///     }
    /// }
    ///
    /// let mut x = Capsule::new(10u32);
    /// let mut s = Capsule::new("starlord".to_string());
    ///
    /// modify_if_u32(&mut x);
    /// modify_if_u32(&mut s);
    ///
    /// assert_eq!(x, Capsule::new(42u32));
    /// assert_eq!(s, Capsule::new("starlord".to_string()));
    /// ```
    pub fn downcast_mut<T: Any>(&mut self) -> Option<&mut T> {
        Dyn::as_any_mut(&mut *self.0).downcast_mut()
    }
}

impl fmt::Debug for Capsule {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Capsule").finish_non_exhaustive()
    }
}
