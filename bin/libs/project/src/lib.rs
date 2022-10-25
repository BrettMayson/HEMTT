#![deny(clippy::all, clippy::nursery)]
#![warn(clippy::pedantic)]
#![allow(clippy::use_self)]

pub mod config;
pub mod hemtt;

mod error;
pub use error::Error;
