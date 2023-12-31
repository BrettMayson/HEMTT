use std::ops::Range;

use ariadne::{sources, ColorGenerator, Fmt, Label, Report};
use hemtt_common::reporting::{Annotation, AnnotationLevel, Code, Processed};

pub struct FindInStr {
    span: Range<usize>,
    haystack: (String, Range<usize>),
    needle: (String, Range<usize>),

    report: Option<String>,
    annotations: Vec<Annotation>,
}

impl Code for FindInStr {
    fn ident(&self) -> &'static str {
        "SAA2"
    }

    fn message(&self) -> String {
        String::from("string search using `in` is faster than `find`")
    }

    fn label_message(&self) -> String {
        String::from("use `in` instead of `find`")
    }

    fn report(&self) -> Option<String> {
        self.report.clone()
    }

    fn ci(&self) -> Vec<Annotation> {
        self.annotations.clone()
    }
}

impl FindInStr {
    #[must_use]
    pub fn new(
        span: Range<usize>,
        haystack: (String, Range<usize>),
        needle: (String, Range<usize>),
        processed: &Processed,
    ) -> Self {
        Self {
            span,
            haystack,
            needle,

            report: None,
            annotations: Vec::new(),
        }
        .report_generate_processed(processed)
        .ci_generate_processed(processed)
    }

    fn report_generate_processed(mut self, processed: &Processed) -> Self {
        let map = processed.mapping(self.span.start).unwrap();
        let map_file = processed.source(map.source()).unwrap();
        let map_haystack = processed.mapping(self.haystack.1.start).unwrap();
        let map_needle = processed.mapping(self.needle.1.start).unwrap();
        let mut out = Vec::new();
        let mut colors = ColorGenerator::new();
        let color_if = colors.next();
        let color_lhs = colors.next();
        colors.next(); // the third color is fairly hard to see here
        let color_rhs = colors.next();
        let report = Report::build(
            ariadne::ReportKind::Advice,
            map_file.0.clone().to_string(),
            map.original_column(),
        )
        .with_code(self.ident())
        .with_message(self.message())
        .with_label(
            Label::new((
                map_file.0.to_string(),
                map.original_column()..map.original_column() + self.span.len(),
            ))
            .with_message(self.label_message())
            .with_color(color_if),
        )
        .with_colored_spans(vec![
            (
                (
                    map_file.0.to_string(),
                    map_haystack.original_column()
                        ..map_haystack.original_column() + self.haystack.1.len(),
                ),
                color_lhs,
            ),
            (
                (
                    map_file.0.to_string(),
                    map_needle.original_column()
                        ..map_needle.original_column() + self.needle.1.len(),
                ),
                color_rhs,
            ),
        ])
        .with_help(format!(
            "try `{} in {}`",
            self.needle.0.as_str().fg(color_rhs),
            self.haystack.0.as_str().fg(color_lhs),
        ));
        report
            .finish()
            .write_for_stdout(
                sources({
                    let mut sources = processed
                        .sources()
                        .iter()
                        .map(|(p, c)| (p.to_string(), c.to_string()))
                        .collect::<Vec<_>>();
                    sources.push((map_file.0.to_string(), map_file.0.read_to_string().unwrap()));
                    sources
                }),
                &mut out,
            )
            .unwrap();
        self.report = Some(String::from_utf8(out).unwrap());
        self
    }

    fn ci_generate_processed(mut self, processed: &Processed) -> Self {
        let map = processed.mapping(self.span.start).unwrap();
        let map_file = processed.source(map.source()).unwrap();
        self.annotations = vec![self.annotation(
            AnnotationLevel::Notice,
            map_file.0.as_str().to_string(),
            map.original(),
        )];
        self
    }
}
