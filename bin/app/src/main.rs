#![deny(clippy::all, clippy::nursery)]
#![warn(clippy::pedantic)]

use hemtt_error::DisplayStyle;

fn main() {
    if let Err(e) = hemtt::execute(&hemtt::cli().get_matches()) {
        eprintln!("{}", e.long(&DisplayStyle::Error));
        std::process::exit(1);
    }
}
