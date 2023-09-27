use ariadne::Fmt;
use hemtt_common::{reporting::Code, workspace::WorkspacePath};

#[allow(unused)]
/// Unexpected token
pub struct InvalidConfigCase {
    /// The [`WorkspacePath`] that was named with an invalid case
    pub(crate) path: WorkspacePath,
}

impl Code for InvalidConfigCase {
    fn ident(&self) -> &'static str {
        "PW2"
    }

    fn message(&self) -> String {
        format!(
            "`{}` is not a valid case for a config",
            self.path.filename()
        )
    }

    fn label_message(&self) -> String {
        format!(
            "`{}` is not a valid case for a config",
            self.path.filename()
        )
    }

    fn help(&self) -> Option<String> {
        Some(format!("Rename to `{}`", self.path.as_str().to_lowercase()))
    }

    fn generate_report(&self) -> Option<String> {
        Some(format!(
            "{} {}\n      {}: {}",
            format!("[{}] Warning:", self.ident()).fg(ariadne::Color::Yellow),
            self.message(),
            "Help".fg(ariadne::Color::Fixed(115)),
            self.help().expect("help should be Some")
        ))
    }
}
