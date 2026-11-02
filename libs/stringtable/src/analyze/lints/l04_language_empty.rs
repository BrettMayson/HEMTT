use std::sync::Arc;

use hemtt_common::config::LintConfig;
use hemtt_workspace::{
    lint::{AnyLintRunner, Lint, LintRunner},
    reporting::{Code, Codes, Diagnostic, Severity},
};

use crate::{analyze::LintData, Key, Package, Project};

crate::analyze::lint!(LintL04LanguageEmpty);

impl Lint<LintData> for LintL04LanguageEmpty {
    fn ident(&self) -> &'static str {
        "language_empty"
    }
    fn sort(&self) -> u32 {
        40
    }
    fn description(&self) -> &'static str {
        "Checks that localization tags are not empty"
    }
    fn documentation(&self) -> &'static str {
        "Localization tags should not be empty. Empty tags like `<English></English>`will result in missing translations."
    }
    fn default_config(&self) -> LintConfig {
        LintConfig::warning()
    }
    fn runners(&self) -> Vec<Box<dyn AnyLintRunner<LintData>>> {
        vec![Box::new(Runner)]
    }
}

pub struct Runner;
impl LintRunner<LintData> for Runner {
    type Target = Project;
    fn run(
        &self,
        _project: Option<&hemtt_common::config::ProjectConfig>,
        config: &hemtt_common::config::LintConfig,
        _processed: Option<&hemtt_workspace::reporting::Processed>,
        _runtime: &hemtt_common::config::RuntimeArguments,
        target: &Project,
        _data: &LintData,
    ) -> Codes {
        let mut codes: Codes = Vec::new();
        for package in target.packages() {
            check_package(package, target, config.severity(), &mut codes);
        }
        codes
    }
}

fn check_package(package: &Package, project: &Project, severity: Severity, codes: &mut Codes) {
    for key in package.keys() {
        check_key(key, project, severity, codes);
    }
    for container in package.containers() {
        check_package(container, project, severity, codes);
    }
}

fn check_key(key: &Key, project: &Project, severity: Severity, codes: &mut Codes) {
    println!("Checking key: {} (allow_empty: {})", key.id(), key.allow_empty());
    if key.allow_empty() {
        return;
    }
    for (lang_name, value) in key.as_list() {
        if value == Some("") {
            codes.push(Arc::new(CodeStringtableLanguageEmpty::new(
                project.path().to_string(),
                key.id().to_string(),
                lang_name.to_string(),
                severity,
            )));
        }
    }
}

#[allow(clippy::module_name_repetitions)]
pub struct CodeStringtableLanguageEmpty {
    path: String,
    key_id: String,
    language: String,
    severity: Severity,
    diagnostic: Option<Diagnostic>,
}

impl Code for CodeStringtableLanguageEmpty {
    fn ident(&self) -> &'static str {
        "L-L04"
    }
    fn severity(&self) -> Severity {
        self.severity
    }
    fn message(&self) -> String {
        format!(
            "Stringtable at `{}` has empty localization tag",
            self.path
        )
    }
    fn note(&self) -> Option<String> {
        Some(format!(
            "Key `{}` - Language `{}`",
            self.key_id, self.language
        ))
    }
    fn help(&self) -> Option<String> {
        Some("Remove empty localization tags and provide appropriate translations.\nOr add <Key allow-empty=\"true\">.".to_string())
    }
    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }
}

impl CodeStringtableLanguageEmpty {
    #[must_use]
    pub fn new(
        path: String,
        key_id: String,
        language: String,
        severity: Severity,
    ) -> Self {
        Self {
            path,
            key_id,
            language,
            severity,
            diagnostic: None,
        }
        .generate_processed()
    }

    fn generate_processed(mut self) -> Self {
        self.diagnostic = Some(Diagnostic::from_code(&self));
        self
    }
}
