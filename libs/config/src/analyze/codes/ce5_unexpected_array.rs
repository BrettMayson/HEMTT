use ariadne::{sources, ColorGenerator, Fmt, Label, Report};
use hemtt_common::reporting::{Annotation, AnnotationLevel, Code, Processed};

use crate::model::Value;
use crate::Property;

pub struct UnexpectedArray {
    property: Property,
    report: Option<String>,
    annotations: Vec<Annotation>,
}

impl Code for UnexpectedArray {
    fn ident(&self) -> &'static str {
        "CE5"
    }

    fn message(&self) -> String {
        "property was not expected to be an array".to_string()
    }

    fn label_message(&self) -> String {
        "unexpected array".to_string()
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
            value: Value::UnexpectedArray(array),
            ..
        } = &self.property
        else {
            return vec![];
        };
        let array_start = processed.mapping(array.span.start).unwrap();
        let array_file = processed.source(array_start.source()).unwrap();
        let array_end = processed.mapping(array.span.end).unwrap();
        let Some(path) = array_file.1 .0.clone() else {
            return vec![];
        };
        vec![(
            path,
            self.diagnostic(lsp_types::Range::new(
                array_start.original().to_lsp(),
                array_end.original().to_lsp(),
            )),
        )]
    }
}

impl UnexpectedArray {
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
            value: Value::UnexpectedArray(array),
            ..
        } = &self.property
        else {
            panic!(
                "UnexpectedArray::report_generate_processed called on non-UnexpectedArray property"
            );
        };
        let array_start = processed.mapping(array.span.start).unwrap();
        let array_file = processed.source(array_start.source()).unwrap();
        let array_end = processed.mapping(array.span.end).unwrap();
        let ident_start = processed.mapping(name.span.start).unwrap();
        let ident_end = processed.mapping(name.span.end).unwrap();
        let mut out = Vec::new();
        let mut colors = ColorGenerator::new();
        let a = colors.next();
        let b = colors.next();
        Report::build(
            ariadne::ReportKind::Error,
            array_file.0.clone(),
            array_start.original_column(),
        )
        .with_code(self.ident())
        .with_message(self.message())
        .with_label(
            #[allow(clippy::range_plus_one)] // not supported by ariadne
            Label::new((
                array_file.0.clone(),
                array_start.original_column()..array_end.original_column(),
            ))
            .with_message(self.label_message())
            .with_color(a),
        )
        .with_label(
            Label::new((
                array_file.0.clone(),
                ident_start.original_column()..ident_end.original_column(),
            ))
            .with_message(format!(
                "expected `{}` here",
                format!("{}[]", name.as_str()).fg(b)
            ))
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
