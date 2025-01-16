//! Support for HCL capsule types.

mod value;

pub use self::value::CapsuleValue;
use core::fmt;

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
    pub fn is<T: 'static>(&self) -> bool {
        self.0.is::<T>()
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
    pub fn downcast<T: 'static>(self) -> Result<T, Capsule> {
        self.0.downcast::<T>().map(|boxed| *boxed).map_err(Capsule)
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
    pub fn downcast_ref<T: 'static>(&self) -> Option<&T> {
        self.0.downcast_ref::<T>()
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
    pub fn downcast_mut<T: 'static>(&mut self) -> Option<&mut T> {
        self.0.downcast_mut::<T>()
    }
}

impl fmt::Debug for Capsule {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("Capsule").finish_non_exhaustive()
    }
}
