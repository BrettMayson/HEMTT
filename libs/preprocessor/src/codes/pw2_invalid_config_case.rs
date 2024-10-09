use hemtt_workspace::{
    reporting::{Code, Severity},
    WorkspacePath,
};

#[allow(unused)]
/// Unexpected token
pub struct InvalidConfigCase {
    /// The [`WorkspacePath`] that was named with an invalid case
    path: WorkspacePath,
}

impl Code for InvalidConfigCase {
    fn ident(&self) -> &'static str {
        "PW2"
    }

    fn severity(&self) -> Severity {
        Severity::Warning
    }

    fn message(&self) -> String {
        format!(
            "`{}` is not a valid case for a config",
            self.path.filename()
        )
    }

    fn help(&self) -> Option<String> {
        Some(format!("Rename to `{}`", self.path.as_str().to_lowercase()))
    }
}

impl InvalidConfigCase {
    #[must_use]
    pub const fn new(path: WorkspacePath) -> Self {
        Self { path }
    }
}
