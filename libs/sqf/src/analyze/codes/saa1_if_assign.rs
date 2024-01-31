use std::ops::Range;

use ariadne::{sources, ColorGenerator, Fmt, Label, Report};
use hemtt_common::reporting::{Annotation, AnnotationLevel, Code, Processed};

pub struct IfAssign {
    condition: (String, Range<usize>),
    lhs: (String, Range<usize>),
    rhs: (String, Range<usize>),

    report: Option<String>,
    annotations: Vec<Annotation>,
}

impl Code for IfAssign {
    fn ident(&self) -> &'static str {
        "SAA1"
    }

    fn message(&self) -> String {
        if self.lhs.0 == "1" && self.rhs.0 == "0" {
            String::from("assignment to if can be replaced with parseNumber")
        } else {
            String::from("assignment to if can be replaced with select")
        }
    }

    fn label_message(&self) -> String {
        if self.lhs.0 == "1" && self.rhs.0 == "0" {
            format!(
                "use `parseNumber {}`",
                self.condition.0.as_str().fg(ColorGenerator::new().next()),
            )
        } else {
            format!(
                "try `[{}, {}] select ({})`",
                self.rhs.0.as_str().fg(ColorGenerator::new().next()),
                self.lhs.0.as_str().fg(ColorGenerator::new().next()),
                self.condition.0.as_str().fg(ColorGenerator::new().next()),
            )
        }
    }

    fn report(&self) -> Option<String> {
        self.report.clone()
    }

    fn ci(&self) -> Vec<Annotation> {
        self.annotations.clone()
    }
}

impl IfAssign {
    #[must_use]
    pub fn new(
        condition: (String, Range<usize>),
        lhs: (String, Range<usize>),
        rhs: (String, Range<usize>),
        processed: &Processed,
    ) -> Self {
        Self {
            condition,
            lhs,
            rhs,

            report: None,
            annotations: Vec::new(),
        }
        .report_generate_processed(processed)
        .ci_generate_processed(processed)
    }

    fn report_generate_processed(mut self, processed: &Processed) -> Self {
        let map = processed.mapping(self.condition.1.start).unwrap();
        let map_file = processed.source(map.source()).unwrap();
        let map_lhs = processed.mapping(self.lhs.1.start).unwrap();
        let map_rhs = processed.mapping(self.rhs.1.start).unwrap();
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
        .with_labels(vec![
            Label::new((
                map_file.0.to_string(),
                map.original_column()..map.original_column() + self.condition.1.len(),
            ))
            .with_message(if self.lhs.0 == "1" && self.rhs.0 == "0" {
                format!(
                    "use `parseNumber ({})`",
                    self.condition.0.as_str().fg(color_if),
                )
            } else {
                format!(
                    "try `[{}, {}] select ({})`",
                    self.rhs.0.as_str().fg(color_rhs),
                    self.lhs.0.as_str().fg(color_lhs),
                    self.condition.0.as_str().fg(color_if),
                )
            })
            .with_color(color_if),
            Label::new(
                (
                    map_file.0.to_string(),
                    map_lhs.original_column()..map_lhs.original_column() + self.lhs.1.len(),
                )
            ).with_color(color_lhs),
            Label::new(
                (
                    map_file.0.to_string(),
                    map_rhs.original_column()..map_rhs.original_column() + self.rhs.1.len(),
                )
            ).with_color(color_rhs),
        ])
        .with_note(
            if self.lhs.0 == "1" && self.rhs.0 == "0" {
                format!("`parseNumber` returns `{}` for `true` and `{}` for `false`", "1".fg(color_lhs), "0".fg(color_rhs))
            } else {
                "The `if` and `else` blocks simply return a constant value, `select` is faster in this case".to_string()
            },
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
        let map = processed.mapping(self.condition.1.start).unwrap();
        let map_file = processed.source(map.source()).unwrap();
        self.annotations = vec![self.annotation(
            AnnotationLevel::Notice,
            map_file.0.as_str().to_string(),
            map.original(),
        )];
        self
    }
}
