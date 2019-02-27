#![macro_use]

use colored::*;

use std::fmt::{Display};
use std::io::{Error};

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => (
        std::io::Error::new(std::io::ErrorKind::Other, format!($($arg)*))
    )
}

pub trait ErrorExt<T> {
    fn prepend_error<M: AsRef<[u8]> + Display>(self, msg: M) -> Result<T, Error>;
    fn print_error(self, exit: bool) -> Option<T>;
}
impl<T> ErrorExt<T> for Result<T, Error> {
    fn prepend_error<M: AsRef<[u8]> + Display>(self, msg: M) -> Result<T, Error> {
        match self {
            Ok(t) => Ok(t),
            Err(e) => Err(error!("{}\n{}", msg, e))
        }
    }

    fn print_error(self, exit: bool) -> Option<T> {
        if let Err(error) = &self {
            eprintln!("{}: {}", "error".red().bold(), error);

            if exit {
                std::process::exit(1);
            }
            return None;
        }
        Some(self.unwrap())
    }
}
