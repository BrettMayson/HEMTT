use hemtt_common::{
    position::{LineCol, Position},
    reporting::{simple, Annotation, AnnotationLevel, Code},
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
        Some(simple(self, ariadne::ReportKind::Warning, self.help()))
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
        Self { path, report: None }
    }
}
