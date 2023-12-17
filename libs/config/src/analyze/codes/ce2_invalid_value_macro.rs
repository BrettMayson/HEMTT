use std::ops::Range;

use ariadne::{sources, ColorGenerator, Fmt, Label, Report};
use hemtt_common::reporting::{Annotation, AnnotationLevel, Code, Processed};

pub struct InvalidValueMacro {
    span: Range<usize>,
    report: Option<String>,
    annotations: Vec<Annotation>,
}

impl Code for InvalidValueMacro {
    fn ident(&self) -> &'static str {
        "CE2"
    }

    fn message(&self) -> String {
        "macro's result could not be parsed".to_string()
    }

    fn label_message(&self) -> String {
        "invalid macro result".to_string()
    }

    fn help(&self) -> Option<String> {
        Some("perhaps this macro has a `Q_` variant or you need `QUOTE(..)`".to_string())
    }

    fn report(&self) -> Option<String> {
        self.report.clone()
    }

    fn ci(&self) -> Vec<Annotation> {
        self.annotations.clone()
    }

    #[cfg(feature = "lsp")]
    fn generate_processed_lsp(&self, processed: &Processed) -> Vec<(vfs::VfsPath, Diagnostic)> {
        let map = processed.mapping(self.span.start).unwrap();
        let token = map.token().walk_up();
        let Some(path) = token.position().path() else {
            return vec![];
        };
        vec![(
            path.clone(),
            self.diagnostic(lsp_types::Range::new(
                token.position().start().to_lsp(),
                token.position().end().to_lsp(),
            )),
        )]
    }
}

impl InvalidValueMacro {
    pub fn new(span: Range<usize>, processed: &Processed) -> Self {
        Self {
            span,
            report: None,
            annotations: vec![],
        }
        .report_generate_processed(processed)
        .ci_generate_processed(processed)
    }

    fn report_generate_processed(mut self, processed: &Processed) -> Self {
        let map = processed.mapping(self.span.start).unwrap();
        let token = map.token();
        let invalid = &processed.as_str()[self.span.start..self.span.end];
        let mut out = Vec::new();
        let mut colors = ColorGenerator::new();
        let a = colors.next();
        Report::build(
            ariadne::ReportKind::Error,
            token.position().path().as_str(),
            token.position().start().0,
        )
        .with_code(self.ident())
        .with_message(self.message())
        .with_label(
            Label::new((
                token.position().path().to_string(),
                token.position().start().0..token.position().end().0,
            ))
            .with_message(self.label_message())
            .with_color(a),
        )
        .with_help(self.help().unwrap())
        .with_note(format!("The processed output was `{}`", invalid.fg(a)))
        .finish()
        .write_for_stdout(sources(processed.sources_adrianne()), &mut out)
        .unwrap();
        self.report = Some(String::from_utf8(out).unwrap());
        self
    }

    fn ci_generate_processed(mut self, processed: &Processed) -> Self {
        let map = processed.mapping(self.span.start).unwrap();
        let map_file = processed.source(map.source()).unwrap();
        self.annotations = vec![self.annotation(
            AnnotationLevel::Error,
            map_file.0.as_str().to_string(),
            map.original(),
        )];
        self
    }
}
