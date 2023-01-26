#![deny(clippy::all, clippy::nursery)]
#![warn(clippy::pedantic)]
#![allow(clippy::use_self)]

#[macro_use]
extern crate tracing;

pub mod addon;
pub mod project;
