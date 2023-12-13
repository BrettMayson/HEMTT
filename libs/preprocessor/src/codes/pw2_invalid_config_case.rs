use ariadne::Fmt;
use hemtt_common::{
    position::{LineCol, Position},
    reporting::{Annotation, AnnotationLevel, Code},
    workspace::WorkspacePath,
};

#[allow(unused)]
/// Unexpected token
pub struct InvalidConfigCase {
    /// The [`WorkspacePath`] that was named with an invalid case
    path: WorkspacePath,
    /// The report
    report: Option<String>,
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

    fn help(&self) -> Option<String> {
        Some(format!("Rename to `{}`", self.path.as_str().to_lowercase()))
    }

    fn report(&self) -> Option<String> {
        self.report.clone()
    }

    fn ci(&self) -> Vec<Annotation> {
        vec![self.annotation(
            AnnotationLevel::Warning,
            self.path.as_str().to_string(),
            &Position::new(LineCol::default(), LineCol::default(), self.path.clone()),
        )]
    }
}

impl InvalidConfigCase {
    pub fn new(path: WorkspacePath) -> Self {
        Self { path, report: None }.report_generate()
    }

    fn report_generate(mut self) -> Self {
        self.report = Some(format!(
            "{} {}\n      {}: {}",
            format!("[{}] Warning:", self.ident()).fg(ariadne::Color::Yellow),
            self.message(),
            "Help".fg(ariadne::Color::Fixed(115)),
            self.help().expect("help should be Some")
        ));
        self
    }
}
