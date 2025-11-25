//! Handles database of external functions

use std::{
    io::{BufReader, BufWriter},
    path::Path,
    sync::{Arc},
};

const FUNCTION_DIR: &str = ".hemttout/functions";

use indexmap::IndexMap;
use tracing::{error, info, trace};

use crate::{analyze::inspector::headers::FunctionInfo};
use super::Database;

impl Database {
    pub(crate) fn project_functions_push(&self, func: Arc<FunctionInfo>) {
        let Ok(mut guard) = self.project_functions.lock() else {
            error!("Failed to lock project functions mutex");
            return;
        };
        guard.push(func);
    }
    pub fn project_functions_testing(&self) -> Vec<Arc<FunctionInfo>> {
        let Ok(guard) = self.project_functions.lock() else {
            error!("Failed to lock project functions mutex");
            return Vec::new();
        };
        guard.clone()
    }
    #[must_use]
    pub fn external_functions_get(&self, name: &str) -> Option<&FunctionInfo> {
        self.external_functions.get(&name.to_lowercase())
    }
    /// Load ALL external functions from the function directory
    #[must_use]
    pub(crate) fn external_functions_load() -> IndexMap<String, FunctionInfo> {
        let path = Path::new(FUNCTION_DIR);
        let Ok(dir) = fs_err::read_dir(path) else {
            trace!("Failed to open {FUNCTION_DIR} for reading");
            return IndexMap::new();
        };
        let mut functions = Vec::new();
        for entry in dir {
            let Ok(entry) = entry else {
                continue;
            };
            let Ok(file) = fs_err::File::open(entry.path()) else {
                continue;
            };
            let reader = BufReader::new(file);
            let serde_vec: Vec<FunctionInfo> = match serde_json::from_reader(reader) {
                Ok(u) => u,
                Err(e) => {
                    error!("Failed to read: {}", e);
                    continue;
                }
            };
            trace!("Loaded {} functions from {:?}", serde_vec.len(), entry.path());
            functions.extend(serde_vec);
        }
        let result: IndexMap<String, FunctionInfo> = functions.into_iter()
            .filter(|f| f.func_name().is_some())
            .map(|f| {
                (
                    f.func_name().map(|s| s.to_lowercase()).unwrap_or_default(),
                    f,
                )
            })
            .collect();
        trace!("Loaded {} external functions", result.len());
        result
    }
    pub(crate) fn project_functions_export(&self) {
        let path = Path::new(FUNCTION_DIR);
        let Ok(exists) = fs_err::exists(path) else {
            trace!("Failed even look at {FUNCTION_DIR}?");
            return;
        };
        if !exists
            && fs_err::create_dir_all(path).is_err() {
                error!("Failed to create directory {FUNCTION_DIR} for exporting functions");
                return;
            }
        let path = path.join("project.json");
        let _ = fs_err::remove_file(&path);
        let Ok(file) = fs_err::File::create(&path) else {
            error!("Failed to create {} for writing", path.display());
            return;
        };
        let mut writer = BufWriter::new(file);
        let Ok(guard) = self.project_functions.lock() else {
            return;
        };
        let funcs: Vec<&FunctionInfo> = guard.iter().map(std::convert::AsRef::as_ref).collect();
        match serde_json::to_writer(&mut writer, &funcs) {
            Ok(()) => info!("Exported {} project functions", funcs.len()),
            Err(e) => error!("Failed to write {}: {}", path.display(), e),
        }
    }
}