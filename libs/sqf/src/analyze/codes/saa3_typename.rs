use std::ops::Range;

use ariadne::{sources, ColorGenerator, Fmt, Label, Report};
use hemtt_common::reporting::{Annotation, AnnotationLevel, Code, Processed};

pub struct Typename {
    span: Range<usize>,
    constant: (String, Range<usize>, usize),

    report: Option<String>,
    annotations: Vec<Annotation>,
}

impl Code for Typename {
    fn ident(&self) -> &'static str {
        "SAA3"
    }

    fn message(&self) -> String {
        String::from("using `typeName` on a constant is slower than using the type directly")
    }

    fn label_message(&self) -> String {
        format!("use `\"{}\"` directly", self.constant.0)
    }

    fn report(&self) -> Option<String> {
        self.report.clone()
    }

    fn ci(&self) -> Vec<Annotation> {
        self.annotations.clone()
    }
}

impl Typename {
    #[must_use]
    pub fn new(
        span: Range<usize>,
        constant: (String, Range<usize>, usize),
        processed: &Processed,
    ) -> Self {
        Self {
            span,
            constant,

            report: None,
            annotations: Vec::new(),
        }
        .report_generate_processed(processed)
        .ci_generate_processed(processed)
    }

    fn report_generate_processed(mut self, processed: &Processed) -> Self {
        let map = processed.mapping(self.span.start).unwrap();
        let map_file = processed.source(map.source()).unwrap();
        let map_constant = processed.mapping(self.constant.1.start).unwrap();
        let mut out = Vec::new();
        let mut colors = ColorGenerator::new();
        let color_typename = colors.next();
        let color_type = colors.next();
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
            .with_message(format!(
                "use `{}` directly",
                &format!("\"{}\"", self.constant.0).fg(color_typename)
            ))
            .with_color(color_typename),
        )
        .with_colored_span(
            (
                map_file.0.to_string(),
                map_constant.original_column()..map_constant.original_column() + self.constant.2,
            ),
            color_type,
        );
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
