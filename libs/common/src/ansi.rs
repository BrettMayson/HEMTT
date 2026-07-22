use std::sync::LazyLock;

static ANSI_SUPPORTED: LazyLock<bool> = LazyLock::new(|| {
    if std::env::args().any(|arg| arg == "--no-color") || std::env::var("NO_COLOR").is_ok() {
        false
    } else {
        enable_ansi_support::enable_ansi_support().is_ok()
    }
});

#[must_use]
pub fn ansi_supported() -> bool {
    *ANSI_SUPPORTED
}
