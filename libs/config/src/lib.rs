#![deny(clippy::all, clippy::nursery, missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::cast_possible_truncation)]

//! HEMTT - Arma 3 Config Parser
//!
//! Requires that files first be tokenized by the [`hemtt_preprocessor`] crate.

mod error;
pub use error::Error;
mod model;
pub use model::*;
mod options;
pub use options::{Options, Preset};
mod rapify;
pub use rapify::Rapify;
