use indicatif::{ProgressBar, ProgressStyle};

#[allow(clippy::module_name_repetitions)]
pub fn progress_bar(size: u64) -> ProgressBar {
    ProgressBar::new(size).with_style(
        ProgressStyle::with_template(
            if std::env::var("CI").is_ok()
                || std::env::args().any(|a| a.starts_with("-v") && a.ends_with('v'))
            {
                ""
            } else {
                "{msg} [{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} "
            },
        )
        .expect("valid template"),
    )
}
