use std::{fs::File, sync::Arc};

use tracing::metadata::LevelFilter;
use tracing_subscriber::{
    Layer, prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt,
};

use crate::Error;

/// Initialize the logger
///
/// # Errors
/// If `hemttout` is true, but no `.hemtt` folder is found
///
/// # Panics
/// If the log file could not be created
pub fn init(verbosity: u8, hemttout: bool) -> Result<(), Error> {
    let format = tracing_subscriber::fmt::format()
        .without_time()
        .with_target(false)
        .compact();

    let stdout = tracing_subscriber::fmt::layer().event_format(format);

    let filter = if crate::is_ci() && !cfg!(debug_assertions) {
        LevelFilter::TRACE
    } else {
        match verbosity {
            0 => LevelFilter::INFO,
            1 => LevelFilter::DEBUG,
            _ => LevelFilter::TRACE,
        }
    };

    if hemttout {
        if !std::path::Path::new(".hemtt").exists() {
            tracing_subscriber::registry()
                .with(stdout.with_filter(filter))
                .init();
            return Err(Error::ConfigNotFound);
        }
        fs_err::create_dir_all(".hemttout").expect("Unable to create `.hemttout`");
        let out_file =
            File::create(".hemttout/latest.log").expect("Unable to create `.hemttout/latest.log`");
        let debug_log = tracing_subscriber::fmt::layer()
            .with_writer(Arc::new(out_file))
            .with_thread_ids(true)
            .with_target(false)
            .with_ansi(false);

        tracing_subscriber::registry()
            .with(stdout.with_filter(filter).and_then(debug_log))
            .init();
    } else {
        tracing_subscriber::registry()
            .with(stdout.with_filter(filter))
            .init();
    }

    Ok(())
}
