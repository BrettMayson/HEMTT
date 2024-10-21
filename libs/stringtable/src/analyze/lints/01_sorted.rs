use std::{io::BufReader, sync::Arc};

use hemtt_common::config::LintConfig;
use hemtt_workspace::{addons::Addon, lint::{AnyLintRunner, Lint, LintRunner}, reporting::{Code, Codes, Diagnostic, Severity}};
use tracing::debug;

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

pub struct Runner;
impl LintRunner<SqfLintData> for Runner {
    type Target = Vec<Addon>;
    fn run(
        &self,
        _project: Option<&hemtt_common::config::ProjectConfig>,
        config: &hemtt_common::config::LintConfig,
        _processed: Option<&hemtt_workspace::reporting::Processed>,
        target: &Vec<Addon>,
        data: &SqfLintData,
    ) -> Codes {
        let mut unsorted = Vec::new();
        let mut codes: Codes = Vec::new();
        for addon in target {
            let stringtable_path = data.workspace()
                .join(addon.folder()).expect("vfs issue")
                .join("stringtable.xml").expect("vfs issue");
            if stringtable_path.exists().expect("vfs issue") {
                let existing = stringtable_path.read_to_string().expect("vfs issue");
                match Project::from_reader(BufReader::new(existing.as_bytes())) {
                    Ok(mut project) => {
                        project.sort();
                        let mut writer = String::new();
                        if let Err(e) = project.to_writer(&mut writer) {
                            panic!("Failed to write stringtable for {}: {}", addon.folder(), e);
                        }
                        if writer != existing {
                            unsorted.push(addon.folder().to_string());
                        }
                    }
                    Err(e) => {
                        debug!("Failed to parse stringtable for {}: {}", addon.folder(), e);
                        codes.push(Arc::new(CodeStringtableInvalid::new(addon.folder())));
                    }
                }
            }
        }
        if unsorted.len() <= 3 {
            for addon in unsorted {
                codes.push(Arc::new(CodeStringtableNotSorted::new(
                    Unsorted::Addon(addon),
                    config.severity(),
                )));
            }
        } else {
            codes.push(Arc::new(CodeStringtableNotSorted::new(
                Unsorted::Addons(unsorted),
                config.severity(),
            )));
        }
        codes
    }
}

pub enum Unsorted {
    Addon(String),
    Addons(Vec<String>),
}

#[allow(clippy::module_name_repetitions)]
pub struct CodeStringtableNotSorted {
    unsorted: Unsorted,
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
            Unsorted::Addon(addon) => format!("Stringtable in `{addon}` is not sorted"),
            Unsorted::Addons(addons) => {
                format!("Stringtables in {} addons are not sorted", addons.len())
            }
        }
    }

    fn help(&self) -> Option<String> {
        Some("Run `hemtt ln sort` to sort the stringtable".to_string())
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }
}

impl CodeStringtableNotSorted {
    #[must_use]
    pub fn new(unsorted: Unsorted, severity: Severity) -> Self {
        Self {
            unsorted,
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

#[allow(clippy::module_name_repetitions)]
pub struct CodeStringtableInvalid {
    addon: String,
    diagnostic: Option<Diagnostic>,
}

impl Code for CodeStringtableInvalid {
    fn ident(&self) -> &'static str {
        "L-L02"
    }

    fn message(&self) -> String {
        format!("Stringtable in `{}` is invalid", self.addon)
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }
}

impl CodeStringtableInvalid {
    #[must_use]
    pub fn new(addon: String) -> Self {
        Self {
            addon,
            diagnostic: None,
        }
        .generate_processed()
    }

    fn generate_processed(mut self) -> Self {
        self.diagnostic = Some(Diagnostic::from_code(&self));
        self
    }
}
