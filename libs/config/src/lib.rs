#![deny(clippy::all, clippy::nursery)]
#![warn(clippy::pedantic)]
#![allow(clippy::cast_possible_truncation)]

mod error;
pub use error::Error;
mod model;
pub use model::*;
mod options;
pub use options::{Options, Preset};
mod rapify;
pub use rapify::Rapify;
