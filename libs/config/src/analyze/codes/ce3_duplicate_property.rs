use ariadne::{sources, ColorGenerator, Label, Report};
use hemtt_common::reporting::Code;
use hemtt_preprocessor::Processed;
use lsp_types::Diagnostic;

use crate::Ident;

pub struct DuplicateProperty {
    conflicts: Vec<Ident>,
}

impl DuplicateProperty {
    pub fn new(conflicts: Vec<Ident>) -> Self {
        Self { conflicts }
    }
}

impl Code for DuplicateProperty {
    fn ident(&self) -> &'static str {
        "CE3"
    }

    fn message(&self) -> String {
        "property was defined more than once".to_string()
    }

    fn label_message(&self) -> String {
        "duplicate property".to_string()
    }

    fn help(&self) -> Option<String> {
        None
    }

    fn generate_processed_report(&self, processed: &Processed) -> Option<String> {
        let first = self.conflicts.first().unwrap();
        let first_map = processed.original_col(first.span.start).unwrap();
        let first_file = processed.source(first_map.source()).unwrap();
        let mut out = Vec::new();
        let mut colors = ColorGenerator::new();

        Report::build(
            ariadne::ReportKind::Error,
            first_file.0.clone(),
            first_map.original().0,
        )
        .with_code(self.ident())
        .with_message(self.message())
        .with_labels(self.conflicts.iter().map(|b| {
            let map_start = processed.original_col(b.span.start).unwrap();
            let map_end = processed.original_col(b.span.end).unwrap();
            let token_start = map_start.token().walk_up();
            let token_end = map_end.token().walk_up();
            Label::new((
                token_start.source().path_or_builtin(),
                token_start.source().start().0..token_end.source().end().0 - 1,
            ))
            .with_color(colors.next())
            .with_message(if b == first {
                "first defined here"
            } else {
                "also defined here"
            })
        }))
        .finish()
        .write_for_stdout(sources(processed.sources()), &mut out)
        .unwrap();
        Some(String::from_utf8(out).unwrap())
    }

    #[cfg(feature = "lsp")]
    fn generate_processed_lsp(&self, processed: &Processed) -> Vec<(vfs::VfsPath, Diagnostic)> {
        let first = self.conflicts.last().unwrap();
        let first_map = processed.original_col(first.span.start).unwrap();
        let first_file = processed.source(first_map.source()).unwrap();
        let Some(path) = first_file.1.0.clone() else {
            return vec![];
        };
        vec![(
            path,
            self.diagnostic(lsp_types::Range::new(first_map.original().to_lsp(), {
                let mut end = first_map.original().to_lsp();
                end.character += first.value.len() as u32;
                end
            })),
        )]
    }
}
