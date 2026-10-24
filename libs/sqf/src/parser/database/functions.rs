//! Handles database of external functions
use std::{io::Write, path::Path, sync::Arc};
use arma3_wiki::{Wiki, functions::Functions, model::Function};
use hemtt_common::config::ProjectConfig;
use indexmap::IndexMap;
use tracing::{error, trace};
use crate::analyze::LintData;
use super::Database;

const FUNCTION_DIR: &str = ".hemttout/functions";

impl Database {
    pub(crate) fn project_functions_push(&self, func: Arc<Function>) {
        let Ok(mut guard) = self.project_functions.lock() else {
            unreachable!("Failed to lock project functions mutex");
        };
        guard.push(func);
    }
    #[must_use]
    pub fn project_functions_testing(&self) -> Vec<Arc<Function>> {
        let Ok(guard) = self.project_functions.lock() else {
            unreachable!("Failed to lock project functions mutex");
        };
        guard.clone()
    }
    #[must_use]
    pub fn external_functions_get(&self, name: &str) -> Option<&Function> {
        self.external_functions.get(&name.to_lowercase())
    }

    /// Load ALL external functions from the function directory
    #[must_use]
    pub(crate) fn load_functions(wiki: &Wiki) -> IndexMap<String, Function> {
        let mut result = IndexMap::new();
        Self::load_functions_wiki(&mut result, wiki);
        Self::load_functions_local(&mut result);
        trace!("Loaded {} external functions", result.len());
        result
    }
    /// Load external functions from the wiki
    fn load_functions_wiki(map: &mut IndexMap<String, Function>, wiki: &Wiki) {
        for (_source, functions) in wiki.functions().iter() {
            // could filter by source if needed?
            map.extend(
                functions
                    .iter()
                    .filter_map(|f| f.name().map(|name| (name.to_lowercase(), f.clone()))),
            );
        }
    }
    /// Load external functions from local files
    fn load_functions_local(map: &mut IndexMap<String, Function>) {
        let path = Path::new(FUNCTION_DIR);
        let Ok(dir) = fs_err::read_dir(path) else {
            trace!("Function directory {FUNCTION_DIR} does not exist, skipping");
            return;
        };
        for entry in dir {
            let Ok(entry) = entry else {
                continue;
            };
            let Ok(file) = fs_err::File::open(entry.path()) else {
                continue;
            };
            let Ok(functions) = Functions::from_file(file.into_file()) else {
                continue;
            };
            map.extend(
                functions
                    .iter()
                    .filter_map(|f| f.name().map(|name| (name.to_lowercase(), f.clone()))),
            );
        }
    }

    /// Export all project functions to a YAML file
    pub(crate) fn export_project_functions_to_file(
        &self,
        project_config: Option<&ProjectConfig>,
        lint_data: &LintData,
    ) {
        let Some(inspector_config) = self.inspector_config() else {
            return;
        };
        let Some(project_config) = project_config else {
            return;
        };
        if project_config.runtime().is_just() {
            return;
        }
        let prefix = project_config.prefix();
        let export_prefixes = inspector_config.export_functions();
        let funcs = self.collect_functions(export_prefixes, lint_data);

        let path = Path::new(FUNCTION_DIR);
        let Ok(exists) = fs_err::exists(path) else {
            trace!("Failed to even look at {FUNCTION_DIR}?");
            return;
        };
        if !exists && fs_err::create_dir_all(path).is_err() {
            error!("Failed to create directory {FUNCTION_DIR} for exporting functions");
            return;
        }
        let path = path.join(format!("{prefix}.yaml"));
        let _ = fs_err::remove_file(&path);
        let Ok(mut file) = fs_err::File::create(&path) else {
            error!("Failed to create {} for writing", path.display());
            return;
        };
        let Ok(str) = Functions::to_string(&funcs) else {
            error!(
                "Failed to serialize functions for writing to {}",
                path.display()
            );
            return;
        };
        let Ok(_) = file.write(str.as_bytes()) else {
            error!("Failed to write functions to {}", path.display());
            return;
        };
    }
    /// Collect all project functions that match the export prefixes
    #[allow(clippy::significant_drop_tightening)]
    fn collect_functions(&self, export_prefixes: &[String], lint_data: &LintData) -> Vec<Function> {
        let project_functions = self.project_functions.lock().expect("mutex");
        let all_defined = lint_data.functions_defined.lock().expect("mutex");
        let mut funcs: Vec<Function> = project_functions
            .iter()
            .filter_map(|f| {
                let name = f.name()?;
                if !name.starts_with('#') {
                    return Some(f.as_ref().clone());
                }
                let file = &name[1..]; // use the filename to search in all_defined
                let mut search = all_defined.iter().filter(|(low, _)| low.ends_with(file));
                let (Some((_, pretty_name)), None) = (search.next(), search.next()) else {
                    // println!("DEBUG: Could not func name for: {file}");
                    return None; // either 0 or 2+ (ambigious, like multiple fnc_handleDamage)
                };
                Some(Function::new(
                    Some(pretty_name.to_string()),
                    f.ret().cloned(),
                    f.params().to_vec(),
                    f.example().to_string(),
                ))
            })
            .filter(|f| {
                f.name()
                    .is_some_and(|n| export_prefixes.iter().any(|p| p == "*" || n.starts_with(p)))
            })
            .collect();
        funcs.sort_by(|a, b| a.name().cmp(&b.name()));
        funcs
    }
}
