use std::ops::Range;

use ariadne::{sources, ColorGenerator, Fmt, Label, Report};
use hemtt_common::reporting::{Annotation, AnnotationLevel, Code, Processed};

use crate::Expression;

pub struct StrFormat {
    span: Range<usize>,
    expr: Expression,

    report: Option<String>,
    annotations: Vec<Annotation>,
}

impl Code for StrFormat {
    fn ident(&self) -> &'static str {
        "SAA4"
    }

    fn message(&self) -> String {
        String::from("using `format [\"%1\", ...]` is slower than using `str ...`")
    }

    fn label_message(&self) -> String {
        format!("use `str {}`", self.expr.source())
    }

    fn report(&self) -> Option<String> {
        self.report.clone()
    }

    fn ci(&self) -> Vec<Annotation> {
        self.annotations.clone()
    }
}

impl StrFormat {
    #[must_use]
    pub fn new(span: Range<usize>, expr: Expression, processed: &Processed) -> Option<Self> {
        Self {
            span,
            expr,

            report: None,
            annotations: Vec::new(),
        }
        .report_generate_processed(processed)
        .map(|s| s.ci_generate_processed(processed))
    }

    fn report_generate_processed(mut self, processed: &Processed) -> Option<Self> {
        let map = processed.mapping(self.span.start).unwrap();
        if map.was_macro() {
            // Don't emit for WARNING_1 and such macros
            return None;
        }
        let map_file = processed.source(map.source()).unwrap();
        let map_expr_start = processed.mapping(self.expr.full_span().start).unwrap();
        let map_expr_end = processed.mapping(self.expr.full_span().end).unwrap();
        let mut out = Vec::new();
        let mut colors = ColorGenerator::new();
        let color = colors.next();
        let color_expr = colors.next();
        let report = Report::build(
            ariadne::ReportKind::Advice,
            map_file.0.clone().to_string(),
            map.original_column(),
        )
        .with_code(self.ident())
        .with_message(self.message())
        .with_labels(vec![
            Label::new((
                map_file.0.to_string(),
                map.original_column()..map.original_column() + self.span.len(),
            ))
            .with_message(format!(
                "use `{} {}`",
                "str".fg(color),
                if matches!(
                    self.expr,
                    Expression::UnaryCommand(_, _, _) | Expression::BinaryCommand(_, _, _, _)
                ) {
                    format!("({})", self.expr.source())
                } else {
                    self.expr.source()
                }
                .fg(color_expr)
            ))
            .with_color(color),
            Label::new((
                map_file.0.to_string(),
                map_expr_start.original_column()..map_expr_end.original_column(),
            ))
            .with_color(color_expr),
        ]);
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
        Some(self)
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
