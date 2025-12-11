//! Handles database of external functions

use std::{io::Write, path::Path, sync::Arc};

const FUNCTION_DIR: &str = ".hemttout/functions";

use arma3_wiki::{Wiki, functions::Functions, model::Function};
use indexmap::IndexMap;
use tracing::{error, trace};

use super::Database;

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
        for (_source, functions) in wiki.functions().iter() { // could filter by source if needed?
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

    pub(crate) fn export_project_functions_to_file(&self, prefix: &str) {
        let path = Path::new(FUNCTION_DIR);
        let Ok(exists) = fs_err::exists(path) else {
            trace!("Failed even look at {FUNCTION_DIR}?");
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
        let Ok(guard) = self.project_functions.lock() else {
            unreachable!("Failed to lock project functions mutex");
        };
        let mut funcs: Vec<&Function> = guard.iter().map(std::convert::AsRef::as_ref).collect();
        funcs.sort_by_key(|f| f.name());
        let funcs = funcs.into_iter().cloned().collect();
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
}
