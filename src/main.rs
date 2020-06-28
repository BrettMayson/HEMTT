use simplelog::{Config, LevelFilter, TermLogger, TerminalMode};

#[macro_use]
pub mod macros;

use hemtt::*;

use crate::error::PrintableError;

fn main() {
    let args: Vec<_> = std::env::args().collect();

    TermLogger::init(
        match (*DEBUG, *TRACE) {
            (_, true) => LevelFilter::Trace,
            (true, _) => LevelFilter::Debug,
            _ => LevelFilter::Info,
        },
        Config::default(),
        TerminalMode::Mixed,
    ).unwrap();

    crate::execute(&args, true).unwrap_or_print();
}
