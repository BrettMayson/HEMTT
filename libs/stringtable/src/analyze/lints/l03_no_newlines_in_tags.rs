use std::sync::Arc;

use hemtt_common::config::LintConfig;
use hemtt_workspace::{
    lint::{AnyLintRunner, Lint, LintRunner},
    reporting::{Code, Codes, Diagnostic, Severity},
};

use crate::{analyze::LintData, Key, Package, Project};

crate::analyze::lint!(LintL03NoNewlinesInTags);

impl Lint<LintData> for LintL03NoNewlinesInTags {
    fn ident(&self) -> &'static str {
        "no_newlines_in_tags"
    }

    fn sort(&self) -> u32 {
        30
    }

    fn description(&self) -> &'static str {
        "Checks that localization tags do not contain newlines"
    }

    fn documentation(&self) -> &'static str {
        "Localization tags should not contain newlines. When stringtable contains newlines inside tags like `<English>\\n    Text\\n</English>`, after binarization in Arma it will include unwanted whitespace: `\"   Text   \"`."
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

fn check_key(key: &Key, _project: &Project, severity: Severity, codes: &mut Codes) {
    let languages = [
        ("Original", key.original()),
        ("English", key.english()),
        ("Czech", key.czech()),
        ("French", key.french()),
        ("Spanish", key.spanish()),
        ("Italian", key.italian()),
        ("Polish", key.polish()),
        ("Portuguese", key.portuguese()),
        ("Russian", key.russian()),
        ("German", key.german()),
        ("Korean", key.korean()),
        ("Japanese", key.japanese()),
        ("Chinese", key.chinese()),
        ("Chinesesimp", key.chinesesimp()),
        ("Turkish", key.turkish()),
        ("Swedish", key.swedish()),
        ("Slovak", key.slovak()),
        ("SerboCroatian", key.serbocroatian()),
        ("Norwegian", key.norwegian()),
        ("Icelandic", key.icelandic()),
        ("Hungarian", key.hungarian()),
        ("Greek", key.greek()),
        ("Finnish", key.finnish()),
        ("Dutch", key.dutch()),
        ("Ukrainian", key.ukrainian()),
        ("Danish", key.danish()),
    ];
    
    for (lang_name, value) in languages {
        if let Some(text) = value {
            // Check if the text starts or ends with newlines/whitespace
            // This would indicate the pattern from the issue where text is on separate lines
            if text != text.trim() {
                codes.push(Arc::new(CodeStringtableNewlineInTag::new(
                    key.id().to_string(),
                    lang_name.to_string(),
                    severity,
                )));
            }
        }
    }
}

#[allow(clippy::module_name_repetitions)]
pub struct CodeStringtableNewlineInTag {
    key_id: String,
    language: String,
    severity: Severity,
    diagnostic: Option<Diagnostic>,
}

impl Code for CodeStringtableNewlineInTag {
    fn ident(&self) -> &'static str {
        "L-L03"
    }

    fn severity(&self) -> Severity {
        self.severity
    }

    fn message(&self) -> String {
        format!(
            "Key `{}` has leading or trailing whitespace in `{}` tag",
            self.key_id, self.language
        )
    }

    fn help(&self) -> Option<String> {
        Some("Remove newlines and extra whitespace from inside localization tags. The text should be on the same line as the tag, e.g., `<English>Text</English>` instead of `<English>\\n    Text\\n</English>`.".to_string())
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }
}

impl CodeStringtableNewlineInTag {
    #[must_use]
    pub fn new(
        key_id: String,
        language: String,
        severity: Severity,
    ) -> Self {
        Self {
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
