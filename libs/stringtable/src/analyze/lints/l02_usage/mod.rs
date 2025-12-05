mod code_duplicate_file;
mod code_missing_file;
mod code_unused_file;

use std::io::Write;
use std::{collections::HashMap, sync::Arc};

use code_duplicate_file::CodeStringtableDuplicateFile;
use code_missing_file::CodeStringtableMissingFile;
use code_unused_file::CodeStringtableUnusedFile;
use hemtt_common::config::LintConfig;
use hemtt_workspace::position::Position;
use hemtt_workspace::{
    lint::{AnyLintRunner, Lint, LintRunner},
    reporting::{Codes, Severity},
};
use regex::Regex;

use crate::{Project, analyze::LintData};

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
        r#"### Stringtable keys should be unique and used. This lint checks for unused, missing, or duplicate keys.
        
Configuration

- **ignore**: Array of stringtable entries to ignore, supports regex or wildcards (`*`)
- **ignore_missing**: Bool to ignore missing stringtables (still written to .hemttout when disabled)
- **ignore_unused**: Bool to ignore missing stringtables (still written to .hemttout when disabled)
- **ignore_duplicate**: Bool to ignore missing stringtables (still written to .hemttout when disabled)
```toml
[lints.stringtables.usage]
options.ignore = [
    "str_myproject_mystring",
]
options.ignore_unused = true
"#
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
        config: &hemtt_common::config::LintConfig,
        _processed: Option<&hemtt_workspace::reporting::Processed>,
        _runtime: &hemtt_common::config::RuntimeArguments,
        target: &Vec<Project>,
        data: &LintData,
    ) -> Codes {
        // Ignore if using `--just`
        // leaving previous (missing_stringtables.txt...) untouched
        if project.is_some_and(|p| p.runtime().is_just()) {
            return Vec::new();
        }
        let mut codes: Codes = Vec::new();
        let mut all = HashMap::with_capacity({
            target
                .iter()
                .map(|p| p.keys().iter().map(|(_, v)| v.len()).sum::<usize>())
                .sum::<usize>()
        });
        for stringtable in target {
            for (key, positions) in stringtable.keys() {
                all.entry(key.to_lowercase())
                    .or_insert_with(Vec::new)
                    .extend(positions.clone());
            }
        }
        let mut unused = all.keys().cloned().collect::<Vec<_>>();
        let mut usages = Vec::new();
        let mut regex = Vec::new();
        for addon in &data.addons {
            let usage = addon
                .build_data()
                .localizations()
                .lock()
                .expect("lock")
                .clone();
            for (original, pos) in usage {
                if original.contains('%') {
                    let mut u = original.clone();
                    let mut i = 1;
                    while u.contains(&format!("%{i}")) {
                        u = u.replace(&format!("%{i}"), "(.+)");
                        i += 1;
                    }
                    if let Ok(re) = Regex::new(&u) {
                        regex.push((re, original, pos));
                    } else {
                        usages.push((u, pos));
                    }
                } else {
                    usages.push((original, pos));
                }
            }
        }
        let mut missing = Vec::new();
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
        for (re, key, position) in regex {
            if all.iter().any(|(k, _)| re.is_match(k)) {
                unused.retain(|k| !re.is_match(k));
            } else if key.starts_with(&prefix) {
                missing.push((key, position));
            }
        }

        // Get and apply lint options
        if let Some(toml::Value::Array(ignore)) = config.option("ignore") {
            for i in ignore {
                let i_lower = i.as_str().map_or(String::new(), str::to_lowercase);
                if i_lower.contains('*') {
                    let regex_pattern = &i_lower.replace('*', ".*");
                    if let Ok(re) = Regex::new(&format!("^{regex_pattern}$")) {
                        unused.retain(|s| !re.is_match(s));
                        missing.retain(|sp| !re.is_match(&sp.0));
                    }
                } else {
                    unused.retain(|s| *s != i_lower);
                    missing.retain(|sp| *sp.0 != i_lower);
                }
            }
        }
        let ignore_missing = config
            .option("ignore_missing")
            .is_some_and(|o| o.as_bool().is_some_and(|b| b));
        let ignore_unused = config
            .option("ignore_unused")
            .is_some_and(|o| o.as_bool().is_some_and(|b| b));
        let ignore_duplicate = config
            .option("ignore_duplicate")
            .is_some_and(|o| o.as_bool().is_some_and(|b| b));

        let _ = fs_err::create_dir_all(".hemttout");
        codes.extend(unused_codes(unused, &all, ignore_unused));
        codes.extend(missing_codes(&missing, ignore_missing));
        codes.extend(duplicate_codes(&all, ignore_duplicate));
        codes
    }
}

fn unused_codes(
    mut unused: Vec<String>,
    all: &HashMap<String, Vec<Position>>,
    ignore: bool,
) -> Codes {
    let _ = fs_err::remove_file(".hemttout/unused_stringtables.txt");
    let mut codes: Codes = Vec::new();
    let _ = fs_err::remove_file(".hemttout/unused_stringtables.txt");
    if !unused.is_empty() {
        unused.sort();
        unused.dedup();
        let mut file = fs_err::File::create(".hemttout/unused_stringtables.txt")
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
                pos.start().1.0,
                pos.start().1.1
            )
            .expect("Failed to write to file");
        }
        if !ignore {
            codes.push(Arc::new(CodeStringtableUnusedFile::new(
                unused.len() as u64,
                Severity::Warning,
            )));
        }
    }
    codes
}

fn missing_codes(missing: &[(String, Position)], ignore: bool) -> Codes {
    let _ = fs_err::remove_file(".hemttout/missing_stringtables.txt");
    let mut codes: Codes = Vec::new();
    if !missing.is_empty() {
        let mut file = fs_err::File::create(".hemttout/missing_stringtables.txt")
            .expect("Failed to create file");
        for (key, pos) in missing {
            writeln!(
                file,
                "{} - {}:{}:{}",
                key,
                pos.path().as_str().trim_start_matches('/'),
                pos.start().1.0,
                pos.start().1.1
            )
            .expect("Failed to write to file");
        }
        if !ignore {
            codes.push(Arc::new(CodeStringtableMissingFile::new(
                missing.len() as u64,
                Severity::Warning,
            )));
        }
    }
    codes
}

fn duplicate_codes(all: &HashMap<String, Vec<Position>>, ignore: bool) -> Codes {
    let _ = fs_err::remove_file(".hemttout/duplicate_stringtables.txt");
    let mut codes: Codes = Vec::new();
    let duplicates = all
        .iter()
        .filter(|(_, v)| v.len() > 1)
        .map(|(k, v)| (k, v.clone()))
        .collect::<Vec<_>>();
    if !duplicates.is_empty() {
        let mut file = fs_err::File::create(".hemttout/duplicate_stringtables.txt")
            .expect("Failed to create file");
        for (key, positions) in &duplicates {
            writeln!(file, "{key}").expect("Failed to write to file");
            for pos in positions {
                writeln!(
                    file,
                    "  {}:{}:{}",
                    pos.path().as_str().trim_start_matches('/'),
                    pos.start().1.0,
                    pos.start().1.1
                )
                .expect("Failed to write to file");
            }
        }
        if !ignore {
            codes.push(Arc::new(CodeStringtableDuplicateFile::new(
                duplicates.len() as u64,
                Severity::Error,
            )));
        }
    }
    codes
}
