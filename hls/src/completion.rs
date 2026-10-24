use std::collections::HashSet;

use hemtt_sqf::parser::database::Database;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::{
    CompletionContext, CompletionItem, CompletionItemKind, CompletionResponse, Documentation,
    MarkupContent, MarkupKind, TextDocumentPositionParams,
};
use tracing::warn;

use crate::config::ConfigAnalyzer;
use crate::sqf::SqfAnalyzer;
use crate::workspace::EditorWorkspace;

const WIKI: &str = "https://community.bistudio.com/wiki/";

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
        .map_or_default(|line| {
            let line = line.trim_end();
            if position.position.character as usize > line.len() {
                line.to_string()
            } else {
                line[..position.position.character as usize].to_string()
            }
        });
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
        _ => {
            let ext = std::path::Path::new(position.text_document.uri.path())
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or_default()
                .to_lowercase();
            if ext != "sqf" {
                return Ok(None);
            }
            let word = current_word(&text_at_cursor);
            if word.is_empty() {
                return Ok(None);
            }
            if word.starts_with('_') {
                let scope_text = text_before_cursor(&text, position.position.line, &text_at_cursor);
                let items = private_variable_completions(&scope_text, &word);
                if items.is_empty() {
                    return Ok(None);
                }
                return Ok(Some(CompletionResponse::Array(items)));
            }
            let Ok(database) = SqfAnalyzer::get().get_database(&workspace) else {
                warn!("Failed to get database for {:?}", workspace.root());
                return Ok(None);
            };
            let items = command_completions(&database, &word);
            if items.is_empty() {
                return Ok(None);
            }
            Ok(Some(CompletionResponse::Array(items)))
        }
    }
}

/// Everything before the cursor, used so completions never suggest a variable
/// before it's been declared/used.
fn text_before_cursor(text: &str, cursor_line: u32, text_at_cursor: &str) -> String {
    let mut result = String::with_capacity(text.len());
    for (i, line) in text.lines().enumerate() {
        if i == cursor_line as usize {
            break;
        }
        result.push_str(line);
        result.push('\n');
    }
    result.push_str(text_at_cursor);
    result
}

/// Removes the contents of any `{...}` block that is already closed in `text`,
/// so variables scoped to a prior, now-finished block aren't suggested. Blocks
/// still open (ancestors of the cursor) are left visible since they're still in scope.
fn strip_closed_blocks(text: &str) -> String {
    let mut stack = Vec::new();
    let mut excluded: Vec<(usize, usize)> = Vec::new();
    for (i, b) in text.bytes().enumerate() {
        match b {
            b'{' => stack.push(i),
            b'}' => {
                if let Some(start) = stack.pop() {
                    excluded.push((start + 1, i));
                }
            }
            _ => {}
        }
    }
    excluded.sort_unstable();
    let mut merged: Vec<(usize, usize)> = Vec::new();
    for (start, end) in excluded {
        if let Some(last) = merged.last_mut()
            && start <= last.1
        {
            last.1 = last.1.max(end);
            continue;
        }
        merged.push((start, end));
    }
    let mut visible = String::with_capacity(text.len());
    let mut idx = 0;
    for (start, end) in merged {
        visible.push_str(&text[idx..start]);
        idx = end;
    }
    visible.push_str(&text[idx..]);
    visible
}

/// Private (`_var`) variables visible at the cursor: used earlier in the file,
/// not inside an already-closed `{...}` block, matching `prefix`.
fn private_variable_completions(text_before_cursor: &str, prefix: &str) -> Vec<CompletionItem> {
    let visible = strip_closed_blocks(text_before_cursor);
    let re = regex::Regex::new(r"[A-Za-z_][A-Za-z0-9_]*").expect("valid regex");
    let prefix = prefix.to_lowercase();
    let mut seen = HashSet::new();
    let mut items: Vec<CompletionItem> = re
        .find_iter(&visible)
        .map(|m| m.as_str())
        .filter(|name| name.starts_with('_'))
        // a lone `_` (or `__`, etc) isn't a real variable, just what's being typed
        .filter(|name| name.chars().any(|c| c.is_ascii_alphabetic()))
        .filter(|name| name.to_lowercase().starts_with(&prefix))
        .filter(|name| seen.insert((*name).to_string()))
        .map(|name| CompletionItem {
            label: name.to_string(),
            kind: Some(CompletionItemKind::VARIABLE),
            ..Default::default()
        })
        .collect();
    items.sort_by(|a, b| a.label.cmp(&b.label));
    items
}

/// The identifier characters immediately before the cursor
fn current_word(text_at_cursor: &str) -> String {
    text_at_cursor
        .chars()
        .rev()
        .take_while(|c| c.is_alphanumeric() || *c == '_')
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect()
}

/// Commands with a dedicated snippet in `languages/sqf.snippets.json`, so they
/// aren't suggested twice (once as the snippet, once as a plain keyword).
const SNIPPET_COMMANDS: &[&str] = &[
    "if",
    "while",
    "for",
    "foreach",
    "switch",
    "try",
    "waituntil",
    "params",
    "private",
];

const NO_SUGGEST_COMMANDS: &[&str] = &["setDammage"];

/// Arma 3 commands (including any project-defined custom commands) matching `prefix`
fn command_completions(database: &Database, prefix: &str) -> Vec<CompletionItem> {
    let prefix = prefix.to_lowercase();
    let mut items: Vec<CompletionItem> = database
        .wiki()
        .commands()
        .raw()
        .values()
        .filter(|command| command.name().to_lowercase().starts_with(&prefix))
        .filter(|command| !SNIPPET_COMMANDS.contains(&command.name().to_lowercase().as_str()))
        .filter(|command| !NO_SUGGEST_COMMANDS.contains(&command.name().to_lowercase().as_str()))
        .map(|command| {
            let is_flow = command.groups().iter().any(|g| g == "Program Flow");
            CompletionItem {
                label: command.name().to_string(),
                kind: Some(if is_flow {
                    CompletionItemKind::KEYWORD
                } else {
                    CompletionItemKind::FUNCTION
                }),
                detail: Some("Arma 3 Command".to_string()),
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: format!(
                        "[BI Wiki]({WIKI}{})\n\n{}",
                        command.name().replace(' ', "_"),
                        command.description()
                    ),
                })),
                ..Default::default()
            }
        })
        .collect();
    items.sort_by(|a, b| a.label.cmp(&b.label));
    items
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
                label: dir_name,
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

    use super::{
        command_completions, current_word, path, private_variable_completions, strip_closed_blocks,
        text_before_cursor,
    };
    use crate::sqf::SqfAnalyzer;
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

    #[test]
    fn current_word_takes_trailing_identifier() {
        assert_eq!(current_word("hint \"test\"; cutRs"), "cutRs");
        assert_eq!(current_word("if (hi"), "hi");
        assert_eq!(current_word(""), "");
        assert_eq!(current_word("hint ("), "");
    }

    #[test]
    fn command_completions_matches_prefix_case_insensitively() {
        let database = SqfAnalyzer::get()
            .get_database(&fixture("plain"))
            .expect("builds");
        let items = command_completions(&database, "hint");
        assert!(items.iter().any(|i| i.label == "hint"), "{items:?}");

        let items = command_completions(&database, "HIN");
        assert!(items.iter().any(|i| i.label == "hint"), "{items:?}");
    }

    #[test]
    fn command_completions_excludes_snippet_commands() {
        let database = SqfAnalyzer::get()
            .get_database(&fixture("plain"))
            .expect("builds");
        let items = command_completions(&database, "if");
        assert!(!items.iter().any(|i| i.label == "if"), "{items:?}");

        let items = command_completions(&database, "wait");
        assert!(!items.iter().any(|i| i.label == "waitUntil"), "{items:?}");
    }

    #[test]
    fn private_variable_completions_finds_used_vars_once() {
        let text = "private _unit = objNull;\n_unitGroup = group _unit;";
        let items = private_variable_completions(text, "_uni");
        let labels: Vec<_> = items.iter().map(|i| i.label.as_str()).collect();
        assert_eq!(labels, vec!["_unit", "_unitGroup"]);
    }

    #[test]
    fn private_variable_completions_ignores_bare_underscores() {
        let empty = [] as [CompletionItem; 0];
        assert_eq!(private_variable_completions("_", "_"), empty);
        assert_eq!(private_variable_completions("__", "_"), empty);
    }

    #[test]
    fn private_variable_completions_ignores_mid_identifier_underscores() {
        let text = "[unit] call ace_advanced_throwing_fnc_canThrow;";
        let items = private_variable_completions(text, "_");
        assert!(items.is_empty(), "{items:?}");
    }

    #[test]
    fn text_before_cursor_excludes_current_and_later_lines() {
        let text = "line0\nline1\nline2";
        assert_eq!(text_before_cursor(text, 1, "li"), "line0\nli");
        assert_eq!(text_before_cursor(text, 0, "li"), "li");
    }

    #[test]
    fn strip_closed_blocks_removes_finished_sibling_scopes() {
        let text = "private _outer = 1;\nif (true) then {\n  private _inner = 2;\n};\n";
        let visible = strip_closed_blocks(text);
        assert!(visible.contains("_outer"));
        assert!(!visible.contains("_inner"));
    }

    #[test]
    fn strip_closed_blocks_keeps_open_ancestor_scope() {
        // the outer block never closes in this prefix, so it's still in scope
        let text =
            "{\n  private _outer = 1;\n  if (true) then {\n    private _inner = 2;\n  };\n  hin";
        let visible = strip_closed_blocks(text);
        assert!(visible.contains("_outer"));
        assert!(!visible.contains("_inner"));
    }

    #[test]
    fn private_variable_completions_ignores_forward_references() {
        let text = "_a = 1;\n_b = 2;\n_c = 3;";
        let scope_text = text_before_cursor(text, 1, "_b = 2;");
        let items = private_variable_completions(&scope_text, "_");
        let labels: Vec<_> = items.iter().map(|i| i.label.as_str()).collect();
        assert_eq!(labels, vec!["_a", "_b"]);
    }
}
