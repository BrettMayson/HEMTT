#![deny(clippy::all, clippy::nursery, missing_docs)]
#![warn(clippy::pedantic)]

//! HEMTT - Common Library

pub mod arma;
pub mod error;
pub mod io;
pub mod math;
pub mod position;
pub mod prefix;
pub mod project;
pub mod reporting;
pub mod version;
pub mod workspace;

mod sign_version;
pub use sign_version::BISignVersion;
