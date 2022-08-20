//! Contains the logic to format HCL data structure.

// @NOTE(mohmann): This module is not exported yet since it is subject to change due to a bigger
// multi-step refactoring. It will eventually replace the formatting code inside serializer
// module.

mod impls;

use crate::{ser, Result};
use std::io;

mod private {
    pub trait Sealed {}
}

/// A trait to format data structures as HCL.
///
/// This trait is sealed to prevent implementation outside of this crate.
pub trait Format: private::Sealed {
    /// Formats a HCL structure using a formatter and writes the result to the provided writer.
    ///
    /// ## Errors
    ///
    /// Formatting the data structure or writing to the writer may fail with an `Error`.
    fn format<W, F>(&self, writer: &mut W, fmt: &mut F) -> Result<()>
    where
        W: ?Sized + io::Write,
        F: ?Sized + ser::Format;
}
