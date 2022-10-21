#![deny(clippy::all, clippy::nursery)]
#![warn(clippy::pedantic)]

mod error;
mod model;
pub use model::*;
mod rapify;
pub use rapify::Rapify;
