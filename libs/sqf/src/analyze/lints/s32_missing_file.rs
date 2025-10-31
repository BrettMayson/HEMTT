use std::{ops::Range, sync::Arc};

use hemtt_common::config::LintConfig;
use hemtt_workspace::{
    lint::{AnyLintRunner, Lint, LintRunner},
    reporting::{Code, Codes, Diagnostic, Processed, Severity},
};

use crate::{analyze::LintData, Expression};

crate::analyze::lint!(LintS30ConfigOf);

impl Lint<LintData> for LintS30ConfigOf {
    fn ident(&self) -> &'static str {
        "missing_file"
    }
    fn sort(&self) -> u32 {
        320
    }
    fn description(&self) -> &'static str {
        "Checks for missing files referenced in sqf"
    }
    fn documentation(&self) -> &'static str {
        "### Explanation

Files should exists
"
    }
    fn default_config(&self) -> LintConfig {
        LintConfig::warning()
    }
    fn runners(&self) -> Vec<Box<dyn AnyLintRunner<LintData>>> {
        vec![Box::new(Runner)]
    }
}

struct Runner;
impl LintRunner<LintData> for Runner {
    type Target = crate::Expression;

    fn run(
        &self,
        project: Option<&hemtt_common::config::ProjectConfig>,
        config: &LintConfig,
        processed: Option<&hemtt_workspace::reporting::Processed>,
        _runtime: &hemtt_common::config::RuntimeArguments,
        target: &Self::Target,
        _data: &LintData,
    ) -> Codes {
        let Some(project) = project else {
            return Vec::new();
        };
        let Some(processed) = processed else {
            return Vec::new();
        };
        let Expression::String(target_str, span, _) = target else {
            return Vec::new();
        };
        let input_lower = target_str.to_ascii_lowercase();
        // ref c11:allow_no_extension
        if !input_lower.contains('.') || input_lower.contains("%1") {
            return vec![];
        }
        let project_prefix_lower = project
            .mainprefix()
            .map_or_else(
                || format!(r"{}\", project.prefix()),
                |mainprefix| format!(r"{}\{}\", mainprefix, project.prefix()),
            )
            .to_ascii_lowercase();
        // try matching with and without leading slash
        let relative_path = if let Some(r) = input_lower.strip_prefix(&project_prefix_lower) {
            r
        } else if let Some(r) = input_lower.strip_prefix(&format!(r"\{project_prefix_lower}")) {
            r
        } else {
            return vec![];
        };
        let sources = processed.sources();
        if sources.is_empty()
            || sources
                .iter()
                // optionals don't get added to VFS
                .any(|f| f.0.as_str().to_ascii_lowercase().starts_with(r"/optionals"))
        {
            return vec![];
        }
        // weird stuff to get a VFS root
        let root = sources[0].0.vfs().root();
        let path = root.join(relative_path).expect("Failed to join path");
        if path.exists().unwrap_or(false) {
            return vec![];
        }
        #[allow(clippy::case_sensitive_file_extension_comparisons)]
        // check for alternative extensions for textures
        for (x, y) in [(".paa", ".tga"), (".tga", ".paa"), (".png", ".paa")] {
            if relative_path.ends_with(x)
                && root
                    .join(relative_path.replace(x, y))
                    .expect("Failed to join path")
                    .exists()
                    .unwrap_or(false)
            {
                return vec![];
            }
        }
        let span = span.start + 1..span.end - 1;
        vec![Arc::new(CodeS32MissingFile::new(
            target_str.to_string(),
            span,
            processed,
            config.severity(),
        ))]
    }
}

#[allow(clippy::module_name_repetitions)]
pub struct CodeS32MissingFile {
    span: Range<usize>,
    path: String,
    severity: Severity,
    diagnostic: Option<Diagnostic>,
}

impl Code for CodeS32MissingFile {
    fn ident(&self) -> &'static str {
        "L-C32"
    }
    fn link(&self) -> Option<&str> {
        Some("/lints/sqf.html#file_missing")
    }
    fn severity(&self) -> Severity {
        self.severity
    }
    fn message(&self) -> String {
        "File Missing".to_string()
    }
    fn label_message(&self) -> String {
        "missing".to_string()
    }
    fn note(&self) -> Option<String> {
        Some(format!("file '{}' was not found in project", self.path))
    }
    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }
}

impl CodeS32MissingFile {
    #[must_use]
    pub fn new(
        path: String,
        span: Range<usize>,
        processed: &Processed,
        severity: Severity,
    ) -> Self {
        Self {
            path,
            span,
            severity,
            diagnostic: None,
        }
        .generate_processed(processed)
    }

    fn generate_processed(mut self, processed: &Processed) -> Self {
        self.diagnostic = Diagnostic::from_code_processed(&self, self.span.clone(), processed);
        self
    }
}
