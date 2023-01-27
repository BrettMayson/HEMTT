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

    let filter = if is_ci() {
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

pub fn is_ci() -> bool {
    // TODO: replace with crate if a decent one comes along
    let checks = vec![
        "CI",
        "APPVEYOR",
        "SYSTEM_TEAMFOUNDATIONCOLLECTIONURI",
        "bamboo_planKey",
        "BITBUCKET_COMMIT",
        "BITRISE_IO",
        "BUDDY_WORKSPACE_ID",
        "BUILDKITE",
        "CIRCLECI",
        "CIRRUS_CI",
        "CODEBUILD_BUILD_ARN",
        "DRONE",
        "DSARI",
        "GITLAB_CI",
        "GO_PIPELINE_LABEL",
        "HUDSON_URL",
        "MAGNUM",
        "NETLIFY_BUILD_BASE",
        "PULL_REQUEST",
        "NEVERCODE",
        "SAILCI",
        "SEMAPHORE",
        "SHIPPABLE",
        "TDDIUM",
        "STRIDER",
        "TEAMCITY_VERSION",
        "TRAVIS",
    ];
    for check in checks {
        if std::env::var(check).is_ok() {
            return true;
        }
    }
    false
}
