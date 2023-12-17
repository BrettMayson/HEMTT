use std::sync::Arc;

use ariadne::{ColorGenerator, Label, Report, ReportKind, Source};
use hemtt_common::{
    position::{LineCol, Position},
    reporting::{Annotation, AnnotationLevel, Code},
    workspace::WorkspacePath,
};
use pest::error::InputLocation;

use crate::{parse::Rule, Error};

#[allow(unused)]
/// Unexpected token
pub struct ParsingFailed {
    error: pest::error::Error<Rule>,
    position: Position,
    report: Option<String>,
}

impl Code for ParsingFailed {
    fn ident(&self) -> &'static str {
        "PE24"
    }

    fn message(&self) -> String {
        "failed to parse".to_string()
    }

    fn report(&self) -> Option<String> {
        self.report.clone()
    }

    fn ci(&self) -> Vec<Annotation> {
        vec![self.annotation(
            AnnotationLevel::Error,
            self.position.path().as_str().to_string(),
            &self.position,
        )]
    }
}

impl ParsingFailed {
    pub fn new(error: pest::error::Error<Rule>, file: WorkspacePath) -> Self {
        let content = file.read_to_string().unwrap_or_default();
        let span = match &error.location {
            InputLocation::Pos(pos) => pos..pos,
            InputLocation::Span((start, end)) => start..end,
        };
        let start = LineCol::from_content(&content, *span.start);
        let end = LineCol::from_content(&content, *span.end);
        let position = Position::new(start, end, file);
        Self {
            error,
            position,
            report: None,
        }
        .report_generate()
    }

    pub fn code(error: pest::error::Error<Rule>, file: WorkspacePath) -> Error {
        Error::Code(Arc::new(Self::new(error, file)))
    }

    fn report_generate(mut self) -> Self {
        let mut colors = ColorGenerator::default();
        let a = colors.next();
        let mut out = Vec::new();
        let span = self.position.span();
        let report = Report::build(ReportKind::Error, self.position.path().as_str(), span.start)
            .with_code(self.ident())
            .with_message(self.message())
            .with_label(
                Label::new((self.position.path().as_str(), span.start..span.end))
                    .with_color(a)
                    .with_message("failed to parse"),
            );
        if let Err(e) = report.finish().write_for_stdout(
            (
                self.position.path().as_str(),
                Source::from(self.position.path().read_to_string().unwrap_or_default()),
            ),
            &mut out,
        ) {
            panic!("while reporting: {e}");
        }
        self.report = Some(String::from_utf8(out).unwrap_or_default());
        self
    }
}
