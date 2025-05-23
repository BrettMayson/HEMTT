//! Rapify configs into a binary format

mod array;
mod class;
mod config;
mod expression;
mod number;
mod property;
mod str;
mod value;

use std::io::Write;

/// Trait for rapifying objects
pub trait Rapify {
    /// Rapify the object into the output stream
    ///
    /// # Errors
    /// if the output stream fails
    fn rapify<O: Write>(&self, output: &mut O, offset: usize) -> Result<usize, std::io::Error>;
    /// Get the length of the rapified object
    fn rapified_length(&self) -> usize;
    /// Get the rapified element code
    fn rapified_code(&self) -> u8 {
        3
    }
}

pub trait Derapify {
    /// Derapify the object from the input stream
    ///
    /// # Errors
    /// if the input stream fails
    fn derapify<I: std::io::Read + std::io::Seek>(input: &mut I) -> Result<Self, std::io::Error>
    where
        Self: Sized;
}
