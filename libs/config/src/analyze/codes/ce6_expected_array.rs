use ariadne::{sources, ColorGenerator, Fmt, Label, Report};
use hemtt_common::reporting::{Annotation, AnnotationLevel, Code, Processed};

use crate::{Property, Value};

pub struct ExpectedArray {
    property: Property,
    report: Option<String>,
    annotations: Vec<Annotation>,
}

impl Code for ExpectedArray {
    fn ident(&self) -> &'static str {
        "CE6"
    }

    fn message(&self) -> String {
        "property was expected to be an array".to_string()
    }

    fn label_message(&self) -> String {
        "expected array".to_string()
    }

    fn report(&self) -> Option<String> {
        self.report.clone()
    }

    fn ci(&self) -> Vec<Annotation> {
        self.annotations.clone()
    }

    #[cfg(feature = "lsp")]
    fn generate_processed_lsp(&self, processed: &Processed) -> Vec<(vfs::VfsPath, Diagnostic)> {
        let Property::Entry {
            value,
            expected_array,
            ..
        } = &self.property
        else {
            return vec![];
        };
        if !expected_array {
            return vec![];
        }
        if let Value::Array(_) = value {
            return vec![];
        }
        let value_start = processed.mapping(value.span().start).unwrap();
        let value_file = processed.source(value_start.source()).unwrap();
        let value_end = processed.mapping(value.span().end).unwrap();
        let Some(path) = value_file.1 .0.clone() else {
            return vec![];
        };
        vec![(
            path,
            self.diagnostic(lsp_types::Range::new(
                value_start.original().to_lsp(),
                value_end.original().to_lsp(),
            )),
        )]
    }
}

impl ExpectedArray {
    pub fn new(property: Property, processed: &Processed) -> Self {
        Self {
            property,
            report: None,
            annotations: vec![],
        }
        .report_generate_processed(processed)
        .ci_generate_processed(processed)
    }

    fn report_generate_processed(mut self, processed: &Processed) -> Self {
        let Property::Entry {
            name,
            value,
            expected_array,
        } = &self.property
        else {
            panic!("ExpectedArray::report_generate_processed called on non-ExpectedArray property");
        };
        assert!(
            expected_array,
            "ExpectedArray::report_generate_processed called on non-ExpectedArray property"
        );
        if let Value::Array(_) = value {
            panic!("ExpectedArray::report_generate_processed called on non-ExpectedArray property");
        }
        let ident_start = processed.mapping(name.span.start).unwrap();
        let ident_file = processed.source(ident_start.source()).unwrap();
        let ident_end = processed.mapping(name.span.end).unwrap();
        let value_start = processed.mapping(value.span().start).unwrap();
        let value_end = processed.mapping(value.span().end).unwrap();
        let mut out = Vec::new();
        let mut colors = ColorGenerator::new();
        let a = colors.next();
        let b = colors.next();
        Report::build(
            ariadne::ReportKind::Error,
            ident_file.0.clone(),
            ident_start.original_column(),
        )
        .with_code(self.ident())
        .with_message(self.message())
        .with_label(
            Label::new((
                ident_file.0.clone(),
                value_start.original_column()..value_end.original_column(),
            ))
            .with_message(self.label_message())
            .with_color(a),
        )
        .with_label(
            Label::new((
                ident_file.0.clone(),
                (ident_end.original_column())..(ident_end.original_column() + 2),
            ))
            .with_message(format!("`{}` indicates an upcoming array", "[]".fg(b)))
            .with_color(b),
        )
        .finish()
        .write_for_stdout(sources(processed.sources()), &mut out)
        .unwrap();
        self.report = Some(String::from_utf8(out).unwrap());
        self
    }

    fn ci_generate_processed(mut self, processed: &Processed) -> Self {
        let map = processed.mapping(self.property.name().span.start).unwrap();
        let map_file = processed.source(map.source()).unwrap();
        self.annotations = vec![self.annotation(
            AnnotationLevel::Error,
            map_file.0.as_str().to_string(),
            map.original(),
        )];
        self
    }
}
