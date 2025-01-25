mod code_missing_file;
mod code_unused_file;

use std::io::Write;
use std::{collections::HashMap, sync::Arc};

use code_missing_file::CodeStringtableMissingFile;
use code_unused_file::CodeStringtableUnusedFile;
use hemtt_common::config::LintConfig;
use hemtt_workspace::{
    lint::{AnyLintRunner, Lint, LintRunner},
    reporting::{Codes, Severity},
};

use crate::{analyze::LintData, Project};

crate::analyze::lint!(LintL02Usage);

impl Lint<LintData> for LintL02Usage {
    fn ident(&self) -> &'static str {
        "usage"
    }

    fn sort(&self) -> u32 {
        20
    }

    fn description(&self) -> &'static str {
        "Checks for unused, missing, or duplicate stringtable keys."
    }

    fn documentation(&self) -> &'static str {
        "Stringtable keys should be unique and used. This lint checks for unused, missing, or duplicate keys."
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
                all.entry(key.to_lowercase())
                    .or_insert_with(Vec::new)
                    .extend(positions.clone());
            }
        }
        let mut unused = all.keys().cloned().collect::<Vec<_>>();
        let mut usages = Vec::new();
        let mut missing = Vec::new();
        for addon in &data.addons {
            let usage = addon
                .build_data()
                .localizations()
                .lock()
                .expect("lock")
                .clone();
            usages.extend(usage);
        }
        let prefix = format!("str_{}", project.map_or("", |p| p.prefix()));
        for (key, position) in usages {
            if all.iter().any(|(k, _)| k == &key) {
                if let Some(pos) = unused.iter().position(|k| k == &key) {
                    unused.remove(pos);
                }
            } else if key.starts_with(&prefix) && !key.contains('%') {
                missing.push((key, position));
            }
        }
        if !unused.is_empty() {
            unused.sort();
            unused.dedup();
            let mut file = std::fs::File::create(".hemttout/unused_stringtables.txt")
                .expect("Failed to create file");
            for key in &unused {
                let pos = all
                    .get(key)
                    .expect("unused must exist in all")
                    .first()
                    .expect("must have a position");
                writeln!(
                    file,
                    "{} - {}:{}:{}",
                    key,
                    pos.path().as_str().trim_start_matches('/'),
                    pos.start().1 .0,
                    pos.start().1 .1
                )
                .expect("Failed to write to file");
            }
            codes.push(Arc::new(CodeStringtableUnusedFile::new(
                unused.len() as u64,
                Severity::Warning,
            )));
        }
        if !missing.is_empty() {
            let mut file = std::fs::File::create(".hemttout/missing_stringtables.txt")
                .expect("Failed to create file");
            for (key, pos) in &missing {
                writeln!(
                    file,
                    "{} - {}:{}:{}",
                    key,
                    pos.path().as_str().trim_start_matches('/'),
                    pos.start().1 .0,
                    pos.start().1 .1
                )
                .expect("Failed to write to file");
            }
            codes.push(Arc::new(CodeStringtableMissingFile::new(
                missing.len() as u64,
                Severity::Error,
            )));
        }
        codes
    }
}
