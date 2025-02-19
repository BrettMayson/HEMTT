use hemtt_workspace::reporting::Definition;
use tower_lsp::lsp_types::{
    ParameterInformation, ParameterLabel, SignatureHelp, SignatureHelpParams, SignatureInformation,
};
use tracing::{debug, warn};

use crate::{files::FileCache, workspace::EditorWorkspaces};

use super::PreprocessorAnalyzer;

impl PreprocessorAnalyzer {
    pub async fn signature_help(&self, params: &SignatureHelpParams) -> Option<SignatureHelp> {
        debug!("signature_help: {:?}", params);
        let url = &params.text_document_position_params.text_document.uri;
        let path = url.to_file_path().ok()?;
        #[derive(Debug, PartialEq, Eq)]
        enum Kind {
            Config,
            Sqf,
        }
        let kind = match path.extension().and_then(|ext| ext.to_str()) {
            Some("hpp" | "cpp" | "ext") => Kind::Config,
            Some("sqf") => Kind::Sqf,
            _ => return None,
        };
        let Some(workspace) = EditorWorkspaces::get().guess_workspace_retry(url).await else {
            warn!("Failed to find workspace for {:?}", url);
            return None;
        };
        let source = workspace.join_url(url).ok()?;
        let processed = self.processed.get(&if kind == Kind::Config {
            source.parent()
        } else {
            source.clone()
        })?;
        let text =
            FileCache::get().text(&params.text_document_position_params.text_document.uri)?;
        let line = text
            .lines()
            .nth(params.text_document_position_params.position.line as usize)?;
        let line = line
            .chars()
            .take(params.text_document_position_params.position.character as usize)
            .collect::<String>();
        let left_parens = line.chars().filter(|c| *c == '(').count();
        let right_parens = line.chars().filter(|c| *c == ')').count();
        if left_parens == right_parens {
            return None;
        }
        let (name, name_end) = find_name(&line);
        if name.is_empty() {
            return None;
        }
        let mut arg = line[name_end..].chars().filter(|c| *c == ',').count();

        // special handle ARR_*
        let re = regex::Regex::new(r"ARR_(\d+)").unwrap();
        let mut matches: Vec<_> = re.captures_iter(&line).collect();
        if name.starts_with("ARR_") {
            matches.pop();
        }
        for m in matches {
            arg -= m.get(1).unwrap().as_str().parse::<usize>().unwrap() - 1;
        }

        let (_, def) = processed.macros.get(name)?.first()?;
        let Definition::Function(def) = def else {
            return None;
        };
        Some(SignatureHelp {
            signatures: vec![SignatureInformation {
                label: format!(
                    "{}({})",
                    name,
                    def.args()
                        .iter()
                        .map(|t| t.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
                documentation: None,
                parameters: Some(
                    def.args()
                        .iter()
                        .map(|t| ParameterInformation {
                            label: ParameterLabel::Simple(t.to_string()),
                            documentation: None,
                        })
                        .collect(),
                ),
                active_parameter: Some(arg as u32),
            }],
            active_signature: Some(0),
            active_parameter: None,
        })
    }
}

/// Find the name of the function in the text
/// picture[] = {ARR_2 // ""
/// picture[] = {ARR_2(1, // ARR_2
/// picture[] = QUOTE(Hello // QUOTE
/// picture[] = QUOTE(ARR_2 // QUOTE
/// picture[] = QUOTE(ARR_2(1, // ARR_2
/// picture[] = QUOTE([ARR_2(1,2)] call // QUOTE
pub fn find_name(text: &str) -> (&str, usize) {
    if !text.contains('(') {
        return ("", 0);
    }
    // find the last open paren, without a closing paren
    let mut parens = 0;
    let end = text
        .char_indices()
        .rev()
        .find_map(|(i, c)| {
            if c == ')' {
                parens += 1;
            } else if c == '(' {
                if parens == 0 {
                    return Some(i);
                }
                parens -= 1;
            }
            None
        })
        .unwrap_or(0);
    let start = text[..end]
        .rfind(|c: char| !c.is_alphabetic() && !c.is_ascii_digit() && c != '_')
        .map(|i| i + 1)
        .unwrap_or(0);
    (&text[start..end], end)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_name() {
        assert_eq!(find_name("picture[] = {ARR_2").0, "");
        assert_eq!(find_name("picture[] = {ARR_2(1").0, "ARR_2");
        assert_eq!(find_name("picture[] = QUOTE(Hello").0, "QUOTE");
        assert_eq!(find_name("picture[] = QUOTE(ARR_2").0, "QUOTE");
        assert_eq!(find_name("picture[] = QUOTE(ARR_2(1").0, "ARR_2");
        assert_eq!(find_name("picture[] = QUOTE([ARR_2(1,2)] call").0, "QUOTE");
        assert_eq!(find_name("GVAR(").0, "GVAR");
    }
}
