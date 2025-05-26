pub mod analyze;
mod derapify;
mod key;
mod package;
mod project;
pub mod rapify;
mod totals;

pub use derapify::derapify;
pub use key::Key;
pub use package::Package;
pub use project::Project;
pub use totals::Totals;

/// Languages in className format
static ALL_LANGUAGES: [&str; 25] = [
    "English",
    "Czech",
    "French",
    "Spanish",
    "Italian",
    "Polish",
    "Portuguese",
    "Russian",
    "German",
    "Korean",
    "Japanese",
    "Chinese",
    "Chinesesimp",
    "Turkish",
    "Swedish",
    "Slovak",
    "SerboCroatian",
    "Norwegian",
    "Icelandic",
    "Hungarian",
    "Greek",
    "Finnish",
    "Dutch",
    "Ukrainian",
    "Danish",
];
