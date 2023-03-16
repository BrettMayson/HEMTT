use std::{
    fs::{create_dir_all, File},
    sync::Arc,
};

use tracing::metadata::LevelFilter;
use tracing_subscriber::{
    prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt, Layer,
};

pub fn init(verbosity: u8, trace: bool) {
    let format = tracing_subscriber::fmt::format()
        .without_time()
        .with_target(false)
        .compact();

    let stdout = tracing_subscriber::fmt::layer().event_format(format);

    let filter = if crate::is_ci() {
        LevelFilter::TRACE
    } else {
        match verbosity {
            0 => LevelFilter::INFO,
            1 => LevelFilter::DEBUG,
            _ => LevelFilter::TRACE,
        }
    };

    create_dir_all(".hemttout").expect("Unable to create `.hemttout`");
    let out_file =
        File::create(".hemttout/latest.log").expect("Unable to create `.hemttout/latest.log`");
    let debug_log = tracing_subscriber::fmt::layer()
        .with_writer(Arc::new(out_file))
        .with_thread_ids(true)
        .with_target(false)
        .with_ansi(false);

    if trace {
        tracing_subscriber::registry()
            .with(stdout.with_filter(filter).and_then(debug_log))
            .with(tracing_tracy::TracyLayer::new())
            .init();
    } else {
        tracing_subscriber::registry()
            .with(stdout.with_filter(filter).and_then(debug_log))
            .init();
    }
}
