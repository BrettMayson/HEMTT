use std::fs::File;

#[macro_use]
extern crate log;

use simplelog::{
    CombinedLogger, ConfigBuilder, LevelFilter, TermLogger, TerminalMode, WriteLogger,
};

use hemtt_app::*;

fn main() {
    let args: Vec<_> = std::env::args().collect();

    let config = ConfigBuilder::new()
        .set_location_level(LevelFilter::Trace)
        .set_target_level(LevelFilter::Debug)
        .set_thread_level(LevelFilter::Trace)
        .set_time_level(LevelFilter::Off)
        .build();

    let level = match (*DEBUG, *TRACE) {
        (_, true) => LevelFilter::Trace,
        (true, _) => LevelFilter::Debug,
        _ => LevelFilter::Info,
    };
    CombinedLogger::init(vec![
        TermLogger::new(level, config.clone(), TerminalMode::Mixed),
        WriteLogger::new(
            LevelFilter::Trace,
            config,
            File::create(log_path(true)).unwrap(),
        ),
    ])
    .unwrap();

    debug!("args: {:?}", args);
    if let Err(e) = crate::execute(&args, true) {
        error!("{}", e);
        if !*CI && e.can_submit_bug() {
            println!("Do you want to submit a bug report?");
        }
        std::process::exit(1);
    }
}
