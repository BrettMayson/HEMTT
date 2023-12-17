use std::ops::Range;

use ariadne::{sources, ColorGenerator, Label, Report};
use hemtt_common::reporting::{Annotation, AnnotationLevel, Code, Processed};

pub struct InvalidValue {
    span: Range<usize>,
    report: Option<String>,
    annotations: Vec<Annotation>,
}

impl Code for InvalidValue {
    fn ident(&self) -> &'static str {
        "CE1"
    }

    fn message(&self) -> String {
        "property's value could not be parsed.".to_string()
    }

    fn label_message(&self) -> String {
        "invalid value".to_string()
    }

    fn help(&self) -> Option<String> {
        Some(
            "use quotes `\"` around the value, or a QUOTE macro if it contains #define values"
                .to_string(),
        )
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
        let map_file = processed.source(map.source()).unwrap();
        let Some(path) = map_file.1 .0.clone() else {
            return vec![];
        };
        vec![(
            path,
            self.diagnostic(lsp_types::Range::new(map.original().to_lsp(), {
                let mut end = map.original().to_lsp();
                end.character += self.span.len() as u32;
                end
            })),
        )]
    }
}

impl InvalidValue {
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
        let map_file = processed.source(map.source()).unwrap();
        let mut out = Vec::new();
        let mut colors = ColorGenerator::new();
        let a = colors.next();
        Report::build(
            ariadne::ReportKind::Error,
            map_file.0.clone(),
            map.original_column(),
        )
        .with_code(self.ident())
        .with_message(self.message())
        .with_label(
            Label::new((
                map_file.0.clone(),
                map.original_column()..map.original_column() + self.span.len(),
            ))
            .with_message(self.label_message())
            .with_color(a),
        )
        .with_help(self.help().unwrap())
        .finish()
        .write_for_stdout(sources(processed.sources()), &mut out)
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
