#![macro_use]

use std::io::{Error};
use std::fmt::{Display};

use colored::*;

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => (
        std::io::Error::new(std::io::ErrorKind::Other, format!($($arg)*))
    )
}

pub trait ErrorExt<T> {
    fn prepend_error<M: AsRef<[u8]> + Display>(self, msg: M) -> Result<T, Error>;
    fn print_error(self, exit: bool) -> ();
}
impl<T> ErrorExt<T> for Result<T, Error> {
    fn prepend_error<M: AsRef<[u8]> + Display>(self, msg: M) -> Result<T, Error> {
        match self {
            Ok(t) => Ok(t),
            Err(e) => Err(error!("{}\n{}", msg, e))
        }
    }

    fn print_error(self, exit: bool) -> () {
        if let Err(error) = self {
            eprintln!("{}: {}", "error".red().bold(), error);

            if exit {
                std::process::exit(1);
            }
        }
    }
}
