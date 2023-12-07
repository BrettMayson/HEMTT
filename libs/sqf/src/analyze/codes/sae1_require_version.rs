use std::ops::Range;

use ariadne::{sources, ColorGenerator, Fmt, Label, Report};
use arma3_wiki::model::Version;
use hemtt_common::reporting::{Annotation, AnnotationLevel, Code, Processed};

pub struct InsufficientRequiredVersion {
    command: String,
    span: Range<usize>,
    version: Version,
    required: (Version, String, Range<usize>),
    stable: Version,
}

impl InsufficientRequiredVersion {
    #[must_use]
    pub const fn new(
        command: String,
        span: Range<usize>,
        version: Version,
        required: (Version, String, Range<usize>),
        stable: Version,
    ) -> Self {
        Self {
            command,
            span,
            version,
            required,
            stable,
        }
    }
}

impl Code for InsufficientRequiredVersion {
    fn ident(&self) -> &'static str {
        "SAE1"
    }

    fn message(&self) -> String {
        format!(
            "command `{}` requires version {}",
            self.command, self.version
        )
    }

    fn label_message(&self) -> String {
        format!("requires version {}", self.version)
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
        let b = colors.next();
        let mut report = Report::build(
            ariadne::ReportKind::Error,
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
            .with_message(format!("requires version {}", self.version.fg(a)))
            .with_color(a),
            Label::new((
                self.required.1.clone(),
                self.required.2.start..self.required.2.end,
            ))
            .with_message(format!(
                "CfgPatch only requires version {}",
                self.required.0.fg(b)
            ))
            .with_color(b),
        ]);
        if self.version > self.stable {
            report = report.with_note(format!(
                "Current stable version is {}. Using {} will require the development branch.",
                self.stable.fg(if self.stable == self.required.0 {
                    b
                } else {
                    colors.next()
                }),
                self.version.fg(a)
            ));
        };
        report
            .finish()
            .write_for_stdout(
                sources({
                    let mut sources = processed
                        .sources()
                        .iter()
                        .map(|(p, c)| (p.to_string(), c.to_string()))
                        .collect::<Vec<_>>();
                    sources.push((
                        self.required.1.clone(),
                        std::fs::read_to_string(self.required.1.trim_start_matches('/')).unwrap(),
                    ));
                    sources
                }),
                &mut out,
            )
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
