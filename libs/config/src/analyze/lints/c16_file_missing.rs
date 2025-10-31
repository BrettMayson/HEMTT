use std::{ops::Range, sync::Arc};

use hemtt_common::config::{LintConfig, ProjectConfig};
use hemtt_workspace::{
    lint::{AnyLintRunner, Lint, LintRunner},
    reporting::{Code, Diagnostic, Processed, Severity},
};

use crate::{analyze::LintData, Item, Value};

crate::analyze::lint!(LintC16FileMissing);

impl Lint<LintData> for LintC16FileMissing {
    fn ident(&self) -> &'static str {
        "file_missing"
    }
    fn sort(&self) -> u32 {
        160
    }
    fn description(&self) -> &'static str {
        "Checks for missing files referenced in config"
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
    type Target = crate::Value;
    fn run(
        &self,
        project: Option<&ProjectConfig>,
        config: &LintConfig,
        processed: Option<&Processed>,
        _runtime: &hemtt_common::config::RuntimeArguments,
        target: &crate::Value,
        _data: &LintData,
    ) -> Vec<std::sync::Arc<dyn Code>> {
        let Some(project) = project else {
            return Vec::new();
        };
        let Some(processed) = processed else {
            return vec![];
        };
        let mut codes = Vec::new();
        let project_prefix_lower = project
            .mainprefix()
            .map_or_else(
                || format!(r"{}\", project.prefix()),
                |mainprefix| format!(r"{}\{}\", mainprefix, project.prefix()),
            )
            .to_ascii_lowercase();

        match target {
            Value::Array(arr) => {
                for item in &arr.items {
                    check_item(item, processed, config, &project_prefix_lower, &mut codes);
                }
            }
            Value::Str(str) => check_str(str, processed, config, &project_prefix_lower, &mut codes),
            _ => {}
        }
        codes
    }
}
fn check_item(
    target: &crate::Item,
    processed: &Processed,
    config: &LintConfig,
    project_prefix_lower: &String,
    codes: &mut Vec<Arc<dyn Code>>,
) {
    match target {
        Item::Array(items) => {
            for element in items {
                check_item(element, processed, config, project_prefix_lower, codes);
            }
        }
        Item::Str(target_str) => {
            check_str(target_str, processed, config, project_prefix_lower, codes);
        }
        _ => {}
    }
}
fn check_str(
    target_str: &crate::Str,
    processed: &Processed,
    config: &LintConfig,
    project_prefix_lower: &String,
    codes: &mut Vec<Arc<dyn Code>>,
) {
    let input_lower = target_str.value().to_ascii_lowercase();
    // ref c11:allow_no_extension
    if !input_lower.contains('.') || input_lower.contains("%1") {
        return;
    }
    // try matching with and without leading slash
    let relative_path = if let Some(r) = input_lower.strip_prefix(project_prefix_lower) {
        r
    } else if let Some(r) = input_lower.strip_prefix(&format!(r"\{project_prefix_lower}")) {
        r
    } else {
        return;
    };
    let sources = processed.sources();
    if sources.is_empty()
        || sources
            .iter()
            // optionals don't get added to VFS
            .any(|f| f.0.as_str().to_ascii_lowercase().starts_with(r"/optionals"))
    {
        return;
    }
    // weird stuff to get a VFS root
    let root = sources[0].0.vfs().root();
    let path = root.join(relative_path).expect("Failed to join path");
    if path.exists().unwrap_or(false) {
        return;
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
            return;
        }
    }
    let span = target_str.span().start + 1..target_str.span().end - 1;
    codes.push(Arc::new(Code16FileMissing::new(
        span,
        target_str.value().to_owned(),
        processed,
        config.severity(),
    )));
}
#[allow(clippy::module_name_repetitions)]
pub struct Code16FileMissing {
    span: Range<usize>,
    path: String,
    severity: Severity,
    diagnostic: Option<Diagnostic>,
}

impl Code for Code16FileMissing {
    fn ident(&self) -> &'static str {
        "L-C16"
    }
    fn link(&self) -> Option<&str> {
        Some("/lints/config.html#file_missing")
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

impl Code16FileMissing {
    #[must_use]
    pub fn new(
        span: Range<usize>,
        path: String,
        processed: &Processed,
        severity: Severity,
    ) -> Self {
        Self {
            span,
            path,
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
