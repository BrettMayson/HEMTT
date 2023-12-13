use std::ops::Range;

use ariadne::{sources, ColorGenerator, Fmt, Label, Report};
use hemtt_common::reporting::{Annotation, AnnotationLevel, Code, Processed};

use crate::Ident;

pub struct MagwellMissingMagazine {
    array: Ident,
    ident: String,
    span: Range<usize>,

    report: Option<String>,
    annotations: Vec<Annotation>,
}

// TODO: maybe we could have a `did you mean` here without too much trouble?

impl Code for MagwellMissingMagazine {
    fn ident(&self) -> &'static str {
        "CW2"
    }

    fn message(&self) -> String {
        "magazine defined in CfgMagazineWells was not found in CfgMagazines".to_string()
    }

    fn label_message(&self) -> String {
        format!("no matching magazine was found: `{}`", self.ident)
    }

    fn report(&self) -> Option<String> {
        self.report.clone()
    }

    fn ci(&self) -> Vec<Annotation> {
        self.annotations.clone()
    }

    #[cfg(feature = "lsp")]
    fn generate_processed_lsp(&self, processed: &Processed) -> Vec<(vfs::VfsPath, Diagnostic)> {}
}

impl MagwellMissingMagazine {
    pub fn new(array: Ident, ident: String, span: Range<usize>, processed: &Processed) -> Self {
        Self {
            array,
            ident,
            span,

            report: None,
            annotations: vec![],
        }
        .report_generate_processed(processed)
        .ci_generate_processed(processed)
    }

    fn report_generate_processed(mut self, processed: &Processed) -> Self {
        let map = processed.mapping(self.array.span.start).unwrap();
        let array_token = map.token();
        let map = processed.mapping(self.span.start).unwrap();
        let value_token_start = map.token();
        let map = processed.mapping(self.span.end).unwrap();
        let value_token_end = map.token();
        let mut out = Vec::new();
        let mut colors = ColorGenerator::new();
        let color = colors.next();
        Report::build(
            ariadne::ReportKind::Warning,
            array_token.position().path().as_str(),
            map.original_column(),
        )
        .with_code(self.ident())
        .with_message(self.message())
        .with_label(
            Label::new((
                value_token_start.position().path().to_string(),
                value_token_start.position().start().0..value_token_end.position().end().0,
            ))
            .with_message(format!(
                "no matching magazine was found: `{}`",
                self.ident.as_str().fg(color)
            ))
            .with_color(color),
        )
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
            AnnotationLevel::Warning,
            map_file.0.as_str().to_string(),
            map.original(),
        )];
        self
    }
}
