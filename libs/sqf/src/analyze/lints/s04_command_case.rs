use std::{ops::Range, sync::Arc};

use hemtt_common::config::{LintConfig, ProjectConfig};
use hemtt_workspace::{lint::{AnyLintRunner, Lint, LintRunner}, reporting::{Code, Codes, Diagnostic, Processed, Severity}};

use crate::{analyze::SqfLintData, Expression};

crate::lint!(LintS04CommandCase);

impl Lint<SqfLintData> for LintS04CommandCase {
    fn ident(&self) -> &str {
        "command_case"
    }

    fn description(&self) -> &str {
        "Command case"
    }

    fn documentation(&self) -> &str {
        "The command does not match the wiki's case"
    }

    fn default_config(&self) -> LintConfig {
        LintConfig::help()
    }

    fn runners(&self) -> Vec<Box<dyn AnyLintRunner<SqfLintData>>> {
        vec![Box::new(Runner)]
    }
}

struct Runner;
impl LintRunner<SqfLintData> for Runner {
    type Target = Expression;
    
    fn run(
        &self,
        _project: Option<&ProjectConfig>,
        config: &LintConfig,
        processed: Option<&Processed>,
        target: &Self::Target,
        data: &SqfLintData,
    ) -> Codes {
        let Some(processed) = processed else {
            return Vec::new();
        };
        let Some(command) = target.command_name() else {
            return Vec::new();
        };
        let Some(wiki) = data.1.wiki().commands().get(&command.to_lowercase()) else {
            return Vec::new();
        };
        if command != wiki.name() {
            return vec![Arc::new(CodeS04CommandCase::new(
                target.span(),
                command.to_string(),
                wiki.name().to_string(),
                processed,
                config.severity(),
            ))];
        }
        Vec::new()
    }
}

#[allow(clippy::module_name_repetitions)]
pub struct CodeS04CommandCase {
    span: Range<usize>,
    used: String,
    wiki: String,

    severity: Severity,
    diagnostic: Option<Diagnostic>,
}

impl Code for CodeS04CommandCase {
    fn ident(&self) -> &'static str {
        "L-S04"
    }

    fn severity(&self) -> Severity {
        self.severity
    }

    fn message(&self) -> String {
        format!("`{}` does not match the wiki's case", self.used)
    }

    fn label_message(&self) -> String {
        "non-standard command case".to_string()
    }

    fn suggestion(&self) -> Option<String> {
        Some(format!("\"{}\"", self.wiki))
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }
}

impl CodeS04CommandCase {
    #[must_use]
    pub fn new(span: Range<usize>, used: String, wiki: String, processed: &Processed, severity: Severity) -> Self {
        Self {
            span,
            used,
            wiki,

            severity,
            diagnostic: None,
        }
        .generate_processed(processed)
    }

    fn generate_processed(mut self, processed: &Processed) -> Self {
        self.diagnostic = Diagnostic::new_for_processed(&self, self.span.clone(), processed);
        self
    }
}
