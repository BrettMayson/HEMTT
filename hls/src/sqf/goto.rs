use tower_lsp::lsp_types::{GotoDefinitionParams, GotoDefinitionResponse, Location, Range};

use crate::{files::FileCache, workspace::EditorWorkspaces};

use super::SqfAnalyzer;

impl SqfAnalyzer {
    /// Clicking a `FUNC`/`QFUNC`/`EFUNC`/`QEFUNC` call jumps straight to the
    /// function's implementation file, instead of the macro's `#define`.
    ///
    /// Falls through to the normal macro-definition lookup (in `main.rs`) if
    /// the cursor isn't on one of these calls, or the conventional
    /// `functions/fnc_Name.sqf` file doesn't exist.
    pub async fn goto_definition(
        &self,
        params: &GotoDefinitionParams,
    ) -> Option<GotoDefinitionResponse> {
        let url = &params.text_document_position_params.text_document.uri;
        if !std::path::Path::new(url.path())
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("sqf"))
        {
            return None;
        }
        let workspace = EditorWorkspaces::get().guess_workspace_retry(url).await?;
        let source = workspace.join_url(url).ok()?;
        let text = FileCache::get().text(url)?;
        let position = params.text_document_position_params.position;
        let line = text.lines().nth(position.line as usize)?;
        let (name, args) = macro_call_at(line, position.character as usize)?;

        let parts: Vec<&str> = source.as_str().split('/').collect();
        if parts.len() < 3 {
            return None;
        }
        let folder = parts[1];
        let current_addon = parts[2];

        let (addon, func_name) = match name {
            "FUNC" | "QFUNC" => (current_addon, args.first()?.as_str()),
            "EFUNC" | "QEFUNC" => (args.first()?.as_str(), args.get(1)?.as_str()),
            _ => return None,
        };
        if addon.is_empty() || func_name.is_empty() {
            return None;
        }

        let file_path = format!("/{folder}/{addon}/functions/fnc_{func_name}.sqf");
        let path = workspace.root().join(&file_path).ok()?;
        if !path.exists().unwrap_or(false) {
            return None;
        }
        Some(GotoDefinitionResponse::Scalar(Location {
            uri: workspace.to_url(&path),
            range: Range::default(),
        }))
    }
}

/// If `character` is inside a `FUNC(...)`/`QFUNC(...)`/`EFUNC(...)`/`QEFUNC(...)`
/// call on `line`, returns the macro name and its trimmed, comma-separated arguments.
fn macro_call_at(line: &str, character: usize) -> Option<(&str, Vec<String>)> {
    let re = regex::Regex::new(r"(FUNC|QFUNC|EFUNC|QEFUNC)\(([^)]*)\)").expect("valid regex");
    for caps in re.captures_iter(line) {
        let whole = caps.get(0).expect("group 0 always matches");
        if whole.start() <= character && character <= whole.end() {
            let name = caps.get(1).expect("group 1 always matches").as_str();
            let args = caps
                .get(2)
                .expect("group 2 always matches")
                .as_str()
                .split(',')
                .map(|s| s.trim().to_string())
                .collect();
            return Some((name, args));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::macro_call_at;

    #[test]
    fn finds_func_call_under_cursor() {
        let line = "[_unit] call FUNC(doSomething);";
        let cursor = line.find("doSomething").expect("in test string") + 2;
        assert_eq!(
            macro_call_at(line, cursor),
            Some(("FUNC", vec!["doSomething".to_string()]))
        );
    }

    #[test]
    fn finds_efunc_call_with_both_args() {
        let line = "[_unit] call EFUNC(main,doSomething);";
        let cursor = line.find("doSomething").expect("in test string");
        assert_eq!(
            macro_call_at(line, cursor),
            Some(("EFUNC", vec!["main".to_string(), "doSomething".to_string()]))
        );
    }

    #[test]
    fn ignores_cursor_outside_any_call() {
        let line = "[_unit] call FUNC(doSomething);";
        assert_eq!(macro_call_at(line, 0), None);
    }
}
