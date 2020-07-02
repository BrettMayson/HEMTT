use simplelog::{ConfigBuilder, LevelFilter, TermLogger, TerminalMode};

#[macro_use]
pub mod macros;

use hemtt::*;

use crate::error::PrintableError;

fn main() {
    let args: Vec<_> = std::env::args().collect();

    let config = ConfigBuilder::new()
        .set_location_level(LevelFilter::Trace)
        .set_target_level(LevelFilter::Trace)
        .set_thread_level(LevelFilter::Trace)
        .build();

    TermLogger::init(
        match (*DEBUG, *TRACE) {
            (_, true) => LevelFilter::Trace,
            (true, _) => LevelFilter::Debug,
            _ => LevelFilter::Info,
        },
        config,
        TerminalMode::Mixed,
    )
    .unwrap();

    crate::execute(&args, true).unwrap_or_print();
}
