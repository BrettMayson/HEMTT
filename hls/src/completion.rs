use std::collections::HashSet;

use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::{
    CompletionContext, CompletionItem, CompletionItemKind, CompletionResponse,
    TextDocumentPositionParams,
};
use tracing::warn;

use crate::config::ConfigAnalyzer;
use crate::sqf::SqfAnalyzer;
use crate::workspace::EditorWorkspace;

pub async fn completion(
    position: TextDocumentPositionParams,
    _context: Option<CompletionContext>,
) -> Result<Option<CompletionResponse>> {
    let Some(workspace) = crate::workspace::EditorWorkspaces::get()
        .guess_workspace_retry(&position.text_document.uri)
        .await
    else {
        warn!(
            "Failed to find workspace for {:?}",
            position.text_document.uri
        );
        return Ok(None);
    };
    let Ok(source) = workspace.join_url(&position.text_document.uri) else {
        warn!(
            "Failed to join url {:?} to workspace",
            position.text_document.uri
        );
        return Ok(None);
    };
    let text = crate::files::FileCache::get()
        .text(&position.text_document.uri)
        .unwrap_or_default();
    let text_at_cursor = text
        .lines()
        .nth(position.position.line as usize)
        .map(|line| {
            let line = line.trim_end();
            if position.position.character as usize > line.len() {
                line.to_string()
            } else {
                line[..position.position.character as usize].to_string()
            }
        })
        .unwrap_or_default();
    let split: Vec<&str> = text_at_cursor.rsplit(' ').collect();
    let mut prefix = (*split.first().unwrap_or(&"")).to_string();
    let mut params = Vec::new();
    if prefix.contains(',') || prefix.contains('\\') {
        let tmp_prefix = {
            let (new_prefix, possible_params) = prefix.split_once('(').unwrap_or((&prefix, ""));
            params = possible_params
                .trim_end_matches(')')
                .split(',')
                .map(|s| s.trim().to_string())
                .collect();
            if !params.is_empty() {
                tracing::debug!("Found params: {:?}", params);
            }
            format!("{new_prefix}(")
        };
        prefix = tmp_prefix;
    }
    let parts: Vec<&str> = source.as_str().split('/').collect();
    if parts.len() < 3 {
        warn!("Invalid as_str for FUNC completion: {}", source.as_str());
        return Ok(None);
    }
    let folder = parts[1];
    let addon = parts[2];
    match prefix.as_str() {
        "FUNC(" | "QFUNC(" => {
            let items = get_functions_defined(addon);
            Ok(Some(CompletionResponse::Array(
                items
                    .into_iter()
                    .map(|item| CompletionItem {
                        label: item,
                        kind: Some(CompletionItemKind::FUNCTION),
                        ..Default::default()
                    })
                    .collect(),
            )))
        }
        "EFUNC(" | "QEFUNC(" => params.first().map_or_else(
            || {
                let items = get_addons();
                Ok(Some(CompletionResponse::Array(
                    items
                        .into_iter()
                        .map(|item| CompletionItem {
                            label: item,
                            kind: Some(CompletionItemKind::MODULE),
                            ..Default::default()
                        })
                        .collect(),
                )))
            },
            |addon| {
                let items = get_functions_defined(addon);
                Ok(Some(CompletionResponse::Array(
                    items
                        .into_iter()
                        .map(|item| CompletionItem {
                            label: item,
                            kind: Some(CompletionItemKind::FUNCTION),
                            ..Default::default()
                        })
                        .collect(),
                )))
            },
        ),
        "PATHTOF(" | "QPATHTOF(" => {
            tracing::debug!("PATHTOF completion for {folder}/{addon}");
            let path = params
                .first()
                .map_or_else(String::new, std::string::ToString::to_string)
                .replace('\\', "/");
            let items = crate::completion::path(&workspace, folder, addon, &path);
            if items.is_empty() {
                warn!("No files found in path for PATHTOF completion: {folder}/{addon}");
                return Ok(None);
            }
            Ok(Some(CompletionResponse::Array(items)))
        }
        "PATHTOEF(" | "QPATHTOEF(" => {
            if let Some(addon) = params.first() {
                tracing::debug!("PATHTOEF completion for {folder}/{addon}");
                let path = params
                    .get(1)
                    .map_or_else(String::new, std::string::ToString::to_string)
                    .replace('\\', "/");
                let items = crate::completion::path(&workspace, folder, addon, &path);
                if items.is_empty() {
                    warn!("No files found in path for PATHTOEF completion: {folder}/{addon}");
                    return Ok(None);
                }
                Ok(Some(CompletionResponse::Array(items)))
            } else {
                let items = get_addons();
                Ok(Some(CompletionResponse::Array(
                    items
                        .into_iter()
                        .map(|item| CompletionItem {
                            label: item,
                            kind: Some(CompletionItemKind::MODULE),
                            ..Default::default()
                        })
                        .collect(),
                )))
            }
        }
        _ => Ok(None),
    }
}

fn get_functions_defined(addon: &str) -> Vec<String> {
    let config_analyzer = ConfigAnalyzer::get();
    let Some(config_functions) = config_analyzer
        .functions_defined
        .iter()
        .find(|item| addon == item.key())
    else {
        warn!("No addon found for {addon:?}",);
        return Vec::new();
    };
    let sqf_analyzer = SqfAnalyzer::get();
    let Some(sqf_functions) = sqf_analyzer
        .functions_defined
        .iter()
        .find(|item| addon == item.key())
    else {
        warn!("No addon found for {addon:?}",);
        return Vec::new();
    };
    let mut items = config_functions
        .value()
        .iter()
        .map(|(_, func)| func.to_string())
        .collect::<HashSet<_>>();
    for funcs in sqf_functions.value().values() {
        for (_, func) in funcs {
            items.insert(func.to_string());
        }
    }
    let mut ret = items
        .into_iter()
        .map(|item| {
            if item.contains("_fnc_") {
                item.rsplit("_fnc_").next().unwrap_or(&item).to_string()
            } else {
                item
            }
        })
        .collect::<Vec<_>>();
    ret.sort();
    ret
}

fn get_addons() -> Vec<String> {
    let config_analyzer = ConfigAnalyzer::get();
    let sqf_analyzer = SqfAnalyzer::get();
    let mut addons: HashSet<String> = HashSet::new();
    for item in config_analyzer.functions_defined.iter() {
        addons.insert(item.key().clone());
    }
    for item in sqf_analyzer.functions_defined.iter() {
        addons.insert(item.key().clone());
    }
    addons.into_iter().collect()
}

fn path(workspace: &EditorWorkspace, folder: &str, addon: &str, path: &str) -> Vec<CompletionItem> {
    let list_path = format!("/{folder}/{addon}/{path}");
    let list_path = list_path.trim_end_matches('/');
    let Ok(path) = workspace.root().join(list_path) else {
        tracing::warn!("Failed to join path: {list_path}");
        return Vec::new();
    };
    let mut items = Vec::new();
    let Ok(entries) = path.read_dir() else {
        tracing::warn!("Failed to read directory: {list_path}");
        return Vec::new();
    };
    for item in entries {
        if item.is_file().unwrap_or(false) {
            let file_name = item.filename();
            items.push(CompletionItem {
                label: file_name,
                kind: Some(CompletionItemKind::FILE),
                ..Default::default()
            });
        } else if item.is_dir().unwrap_or(false) {
            let dir_name = item.filename();
            items.push(CompletionItem {
                label: dir_name.clone(),
                kind: Some(CompletionItemKind::FOLDER),
                ..Default::default()
            });
        }
    }
    if items.is_empty() {
        tracing::warn!("No files found in path: {list_path}");
        return Vec::new();
    }
    items
}

#[cfg(test)]
mod tests {
    use tower_lsp::lsp_types::CompletionItem;

    use super::path;
    use crate::workspace::tests::fixture;

    #[test]
    fn lists_addon_files() {
        let items = path(&fixture("project"), "addons", "valid", "");
        assert!(items.iter().any(|i| i.label == "script.sqf"), "{items:?}");
    }

    /// A half-typed `#include` path names a directory that does not exist.
    #[test]
    fn missing_directory_is_empty() {
        let empty = [] as [CompletionItem; 0];
        assert_eq!(path(&fixture("project"), "addons", "valid", "nope/"), empty);
        assert_eq!(
            path(&fixture("project"), "addons", "nosuchaddon", ""),
            empty
        );
    }
}
