#[macro_use]
extern crate log;

#[macro_use]
extern crate hemtt_macros;

use simplelog::{
    CombinedLogger, ConfigBuilder, LevelFilter, LevelPadding, TermLogger, TerminalMode, WriteLogger,
};

use hemtt_app::*;

fn main() {
    let args: Vec<_> = std::env::args().collect();

    let config = ConfigBuilder::new()
        .set_location_level(LevelFilter::Trace)
        .set_target_level(LevelFilter::Trace)
        .set_thread_level(LevelFilter::Trace)
        .set_time_level(LevelFilter::Off)
        .set_level_padding(LevelPadding::Right)
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
            create_file!(log_path(true)).unwrap(),
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
