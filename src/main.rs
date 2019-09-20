#[cfg(windows)]
use ansi_term;

#[macro_use]
pub mod macros;

use hemtt::*;

use crate::error::PrintableError;

fn main() {
    if cfg!(windows) {
        ansi_support();
    }

    let args: Vec<_> = std::env::args().collect();

    crate::execute(&args, true).unwrap_or_print();
}

#[cfg(windows)]
fn ansi_support() {
    // Attempt to enable ANSI support in terminal
    // Disable colored output if failed
    if ansi_term::enable_ansi_support().is_err() {
        colored::control::set_override(false);
    }
}

#[cfg(not(windows))]
fn ansi_support() {
    unreachable!();
}
