#![deny(clippy::all, clippy::nursery)]
#![warn(clippy::pedantic)]

use hemtt_error::DisplayStyle;

fn main() {
    #[cfg(windows)]
    if ansi_term::enable_ansi_support().is_err() {
        colored::control::set_override(false);
    }
    if let Err(e) = hemtt::execute(&hemtt::cli().get_matches()) {
        eprintln!("{}", e.long(&DisplayStyle::Error));
        std::process::exit(1);
    }
}
