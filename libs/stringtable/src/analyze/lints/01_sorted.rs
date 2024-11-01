use std::sync::Arc;

use hemtt_common::config::LintConfig;
use hemtt_workspace::{lint::{AnyLintRunner, Lint, LintRunner}, reporting::{Code, Codes, Diagnostic, Severity}, WorkspacePath};

use crate::{analyze::SqfLintData, Project};

crate::analyze::lint!(LintL01Sorted);

impl Lint<SqfLintData> for LintL01Sorted {
    fn ident(&self) -> &str {
        "sorted"
    }

    fn sort(&self) -> u32 {
        10
    }

    fn description(&self) -> &str {
        "Checks if stringtables are sorted"
    }

    fn documentation(&self) -> &str {
        "Stringtables should be sorted alphabetically and the keys in the order from the [Arma 3 Wiki](https://community.bistudio.com/wiki/Stringtable.xml#Supported_Languages)."
    }

    fn default_config(&self) -> LintConfig {
        LintConfig::warning()
    }

    fn runners(&self) -> Vec<Box<dyn AnyLintRunner<SqfLintData>>> {
        vec![Box::new(Runner)]
    }
}

pub type StringtableData = (Project, WorkspacePath, String);

pub struct Runner;
impl LintRunner<SqfLintData> for Runner {
    type Target = Vec<StringtableData>;
    fn run(
        &self,
        _project: Option<&hemtt_common::config::ProjectConfig>,
        config: &hemtt_common::config::LintConfig,
        _processed: Option<&hemtt_workspace::reporting::Processed>,
        target: &Vec<StringtableData>,
        _data: &SqfLintData,
    ) -> Codes {
        let mut unsorted = Vec::new();
        let mut codes: Codes = Vec::new();
        let only_lang = matches!(config.option("only-lang"), Some(toml::Value::Boolean(true)));
        for (project, path, existing) in target {
            let mut project = project.clone();
            if !only_lang {
                project.sort();
            }
            let mut writer = String::new();
            if let Err(e) = project.to_writer(&mut writer) {
                panic!("Failed to write stringtable for {path}: {e}");
            }
            if &writer != existing {
                unsorted.push(path.as_str().to_string());
            }
        }
        if unsorted.len() <= 3 {
            for path in unsorted {
                codes.push(Arc::new(CodeStringtableNotSorted::new(
                    Unsorted::Path(path),
                    only_lang,
                    config.severity(),
                )));
            }
        } else {
            codes.push(Arc::new(CodeStringtableNotSorted::new(
                Unsorted::Paths(unsorted),
                only_lang,
                config.severity(),
            )));
        }
        codes
    }
}

pub enum Unsorted {
    Path(String),
    Paths(Vec<String>),
}

#[allow(clippy::module_name_repetitions)]
pub struct CodeStringtableNotSorted {
    unsorted: Unsorted,
    only_lang: bool,
    severity: Severity,
    diagnostic: Option<Diagnostic>,
}

impl Code for CodeStringtableNotSorted {
    fn ident(&self) -> &'static str {
        "L-L01"
    }

    fn severity(&self) -> Severity {
        self.severity
    }

    fn message(&self) -> String {
        match &self.unsorted {
            Unsorted::Path(path) => format!("Stringtable at `{path}` is not sorted"),
            Unsorted::Paths(paths) => {
                format!("{} stringtables are not sorted", paths.len())
            }
        }
    }

    fn help(&self) -> Option<String> {
        if self.only_lang {
            Some("Run `hemtt ln sort --only-lang` to sort the stringtable".to_string())
        } else {
            Some("Run `hemtt ln sort` to sort the stringtable".to_string())
        }
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }
}

impl CodeStringtableNotSorted {
    #[must_use]
    pub fn new(unsorted: Unsorted, only_lang: bool, severity: Severity) -> Self {
        Self {
            unsorted,
            only_lang,
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
