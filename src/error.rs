use colored::*;

use std::fmt::{Debug, Display};

pub trait ErrorExt<T, E> {
    fn print(self) -> Option<T>;
    fn unwrap_or_print(self) -> T;
    fn unwrap_or_print_with_context(self, context: &str) -> T;
}
impl<T, E: Debug + Display> ErrorExt<T, E> for Result<T, E> {
    fn print(self) -> Option<T> {
        if let Err(error) = &self {
            eprintln!("{}: {}", "error".red().bold(), error);
            return None;
        }
        Some(self.unwrap())
    }
    fn unwrap_or_print_with_context(self, context: &str) -> T {
        if let Err(error) = &self {
            eprintln!("{}: {}{}", "error".red().bold(), context, error);
            std::process::exit(1);
        }
        self.unwrap()
    }
    fn unwrap_or_print(self) -> T {
        self.unwrap_or_print_with_context("")
    }
}
