use std::ops::Range;

use ariadne::{sources, ColorGenerator, Fmt, Label, Report};
use hemtt_common::reporting::{Annotation, AnnotationLevel, Code, Processed};

use crate::{BinaryCommand, Expression};

pub struct SelectParseNumber {
    span: Range<usize>,
    expr: Expression,
    negate: bool,

    report: Option<String>,
    annotations: Vec<Annotation>,
}

impl Code for SelectParseNumber {
    fn ident(&self) -> &'static str {
        "SAA5"
    }

    fn message(&self) -> String {
        String::from("using `select` where `parseNumber` is more appropriate")
    }

    fn label_message(&self) -> String {
        format!(
            "use `parseNumber {}`",
            if matches!(
                self.expr,
                Expression::UnaryCommand(_, _, _) | Expression::BinaryCommand(_, _, _, _)
            ) {
                format!("({})", self.expr.source())
            } else {
                self.expr.source()
            }
            .fg(ColorGenerator::new().next()),
        )
    }

    fn report(&self) -> Option<String> {
        self.report.clone()
    }

    fn ci(&self) -> Vec<Annotation> {
        self.annotations.clone()
    }
}

impl SelectParseNumber {
    #[must_use]
    pub fn new(span: Range<usize>, expr: Expression, processed: &Processed, negate: bool) -> Self {
        Self {
            span,
            expr,
            negate,

            report: None,
            annotations: Vec::new(),
        }
        .report_generate_processed(processed)
        .ci_generate_processed(processed)
    }

    #[allow(clippy::too_many_lines)]
    fn report_generate_processed(mut self, processed: &Processed) -> Self {
        let map = processed.mapping(self.span.start).unwrap();
        let map_file = processed.source(map.source()).unwrap();
        let map_expr_start = processed.mapping(self.expr.full_span().start).unwrap();
        let map_expr_end = processed.mapping(self.expr.full_span().end).unwrap();
        let mut out = Vec::new();
        let mut colors = ColorGenerator::new();
        let color = colors.next();
        let color_expr = colors.next();
        let mut report = Report::build(
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
                "parseNumber".fg(color),
                if matches!(
                    self.expr,
                    Expression::UnaryCommand(_, _, _) | Expression::BinaryCommand(_, _, _, _)
                ) || self.negate
                {
                    let mut display_negate = true;
                    let expr = if self.negate {
                        if let Expression::BinaryCommand(BinaryCommand::NotEq, a, b, c) = &self.expr
                        {
                            display_negate = false;
                            Expression::BinaryCommand(
                                BinaryCommand::Eq,
                                a.clone(),
                                b.clone(),
                                c.clone(),
                            )
                            .source()
                        } else if let Expression::BinaryCommand(BinaryCommand::Eq, a, b, c) =
                            &self.expr
                        {
                            display_negate = false;
                            Expression::BinaryCommand(
                                BinaryCommand::NotEq,
                                a.clone(),
                                b.clone(),
                                c.clone(),
                            )
                            .source()
                        } else {
                            self.expr.source()
                        }
                    } else {
                        self.expr.source()
                    };
                    format!(
                        "{}({})",
                        if self.negate && display_negate {
                            "!"
                        } else {
                            ""
                        },
                        expr
                    )
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
        if let Expression::BinaryCommand(BinaryCommand::NotEq, _, _, _) = &self.expr {
            if self.negate {
                report = report.with_note(format!(
                    "`{}` is now `{}`",
                    "!=".fg(color_expr),
                    "==".fg(color_expr)
                ));
            }
        } else if let Expression::BinaryCommand(BinaryCommand::Eq, _, _, _) = &self.expr {
            if self.negate {
                report = report.with_note(format!(
                    "`{}` is now `{}`",
                    "==".fg(color_expr),
                    "!=".fg(color_expr)
                ));
            }
        } else if self.negate {
            report = report.with_note(format!(
                "The condition is now negated with `{}`",
                "!".fg(color_expr)
            ));
        }
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
