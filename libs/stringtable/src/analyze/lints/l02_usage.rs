use std::{collections::HashMap, sync::Arc};
use std::io::Write;

use hemtt_common::config::LintConfig;
use hemtt_workspace::{lint::{AnyLintRunner, Lint, LintRunner}, reporting::{Code, Codes, Diagnostic, Severity}};

use crate::{analyze::LintData, Project};

crate::analyze::lint!(LintL02Usage);

impl Lint<LintData> for LintL02Usage {
    fn ident(&self) -> &'static str {
        "usasge"
    }

    fn sort(&self) -> u32 {
        20
    }

    fn description(&self) -> &'static str {
        "Checks if stringtables are sorted"
    }

    fn documentation(&self) -> &'static str {
        "Stringtables should be sorted alphabetically and the keys in the order from the [Arma 3 Wiki](https://community.bistudio.com/wiki/Stringtable.xml#Supported_Languages)."
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
    type Target = Vec<Project>;
    fn run(
        &self,
        project: Option<&hemtt_common::config::ProjectConfig>,
        _config: &hemtt_common::config::LintConfig,
        _processed: Option<&hemtt_workspace::reporting::Processed>,
        target: &Vec<Project>,
        data: &LintData,
    ) -> Codes {
        let mut codes: Codes = Vec::new();
        let mut all = HashMap::new();
        for project in target {
            for (key, positions) in &project.keys {
                all.entry(key.to_lowercase()).or_insert_with(Vec::new).extend(positions.clone());
            }
        }
        let mut unused = all.keys().cloned().collect::<Vec<_>>();
        let mut usages = Vec::new();
        for addon in &data.addons {
            let usage = addon.build_data().localizations().lock().expect("lock").clone();
            usages.extend(usage);
        }
        let prefix = format!("str_{}", project.map_or("", |p| p.prefix()));
        println!("{} usages", usages.len());
        for (key, position) in usages {
            if all.iter().any(|(k, _)| k == &key) {
                if let Some(pos) = unused.iter().position(|k| k == &key) {
                    unused.remove(pos);
                }
            } else if key.starts_with(&prefix) {
                println!("{key} not found - {position:?}");
            }
        }
        if !unused.is_empty() {
            unused.sort();
            unused.dedup();
            let mut file = std::fs::File::create(".hemttout/unused_stringtables.txt").expect("Failed to create file");
            for key in &unused {
                let pos = all.get(key).expect("unused must exist in all").first().expect("must have a position");
                writeln!(file, "{} - {}:{}:{}", key, pos.path().as_str().trim_start_matches('/'), pos.start().1.0, pos.start().1.1).expect("Failed to write to file");
            }
            codes.push(Arc::new(CodeStringtableUnused::new(unused.len() as u64, Severity::Warning)));
        }
        codes
    }
}


#[allow(clippy::module_name_repetitions)]
pub struct CodeStringtableUnused {
    count: u64,
    severity: Severity,
    diagnostic: Option<Diagnostic>,
}

impl Code for CodeStringtableUnused {
    fn ident(&self) -> &'static str {
        "L-L02U"
    }

    fn severity(&self) -> Severity {
        self.severity
    }

    fn message(&self) -> String {
        format!("There are {} unused keys in stringtables.", self.count)
    }

    fn note(&self) -> Option<String> {
        Some(String::from("A list has been generated in .hemttout/unused_stringtables.txt"))
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }
}

impl CodeStringtableUnused {
    #[must_use]
    pub fn new(count: u64, severity: Severity) -> Self {
        Self {
            count,
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
