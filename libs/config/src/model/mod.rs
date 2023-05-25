mod array;
mod class;
mod entry;
mod ident;
mod number;
mod str;

pub use self::str::Str;
pub use array::Array;
pub use class::{Children, Class, Properties, Property};
pub use entry::Entry;
pub use ident::Ident;
pub use number::Number;

#[derive(Debug)]
/// A config file
pub struct Config {
    /// The root, unnamed class
    pub root: Class,
}
