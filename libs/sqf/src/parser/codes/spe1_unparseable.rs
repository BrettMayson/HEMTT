use std::ops::Range;

use ariadne::{sources, ColorGenerator, Label, Report};
use hemtt_common::reporting::{Annotation, AnnotationLevel, Code, Processed};

pub struct UnparseableSyntax {
    span: Range<usize>,
}

impl UnparseableSyntax {
    #[must_use]
    pub const fn new(span: Range<usize>) -> Self {
        Self { span }
    }
}

#[allow(clippy::range_plus_one)] // chumsky problem
impl Code for UnparseableSyntax {
    fn ident(&self) -> &'static str {
        "SPE1"
    }

    fn message(&self) -> String {
        "unparseable syntax".to_string()
    }

    fn label_message(&self) -> String {
        "unparseable syntax".to_string()
    }

    fn help(&self) -> Option<String> {
        None
    }

    fn report_generate_processed(&self, processed: &Processed) -> Option<String> {
        let map = processed.mapping(self.span.start).unwrap();
        let map_file = processed.source(map.source()).unwrap();
        let mut out = Vec::new();
        let mut colors = ColorGenerator::new();
        let a = colors.next();
        let end = map.original_column() + map.token().to_string().len();
        Report::build(
            ariadne::ReportKind::Error,
            map_file.0.clone(),
            map.original_column(),
        )
        .with_code(self.ident())
        .with_message(self.message())
        .with_label(
            Label::new((map_file.0.clone(), end..end + 1))
                .with_message(self.label_message())
                .with_color(a),
        )
        .finish()
        .write_for_stdout(sources(processed.sources()), &mut out)
        .unwrap();
        Some(String::from_utf8(out).unwrap())
    }

    fn ci_generate_processed(&self, processed: &Processed) -> Vec<Annotation> {
        let map = processed.mapping(self.span.start).unwrap();
        let map_file = processed.source(map.source()).unwrap();
        vec![self.annotation(
            AnnotationLevel::Error,
            map_file.0.as_str().to_string(),
            map.original(),
        )]
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
