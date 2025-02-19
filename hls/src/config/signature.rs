use hemtt_workspace::reporting::Definition;
use tower_lsp::lsp_types::{
    ParameterInformation, ParameterLabel, SignatureHelp, SignatureHelpParams, SignatureInformation,
};
use tracing::{debug, warn};

use crate::{files::FileCache, workspace::EditorWorkspaces};

use super::ConfigAnalyzer;

impl ConfigAnalyzer {
    pub async fn signature_help(&self, params: &SignatureHelpParams) -> Option<SignatureHelp> {
        let url = &params.text_document_position_params.text_document.uri;
        let path = url.to_file_path().ok()?;
        if !matches!(
            path.extension().and_then(|ext| ext.to_str()),
            Some("hpp" | "cpp" | "ext")
        ) {
            debug!("Not a cpp file: {:?}", path);
            return None;
        }
        let Some(workspace) = EditorWorkspaces::get().guess_workspace_retry(url).await else {
            warn!("Failed to find workspace for {:?}", url);
            return None;
        };
        let source = workspace.join_url(url).ok()?;
        let processed = self.processed.get(&source.parent())?;
        let text =
            FileCache::get().text(&params.text_document_position_params.text_document.uri)?;
        let line = text
            .lines()
            .nth(params.text_document_position_params.position.line as usize)?;
        let line = line
            .chars()
            .take(params.text_document_position_params.position.character as usize)
            .collect::<String>();
        debug!("line: {:?}", line);
        let left_parens = line.chars().filter(|c| *c == '(').count();
        let right_parens = line.chars().filter(|c| *c == ')').count();
        if left_parens == right_parens {
            return None;
        }
        let (name, name_end) = find_name(&line);
        let arg = line[name_end..].chars().filter(|c| *c == ',').count();
        let (_, def) = processed.macros().get(name)?.first()?;
        let Definition::Function(def) = def else {
            return None;
        };
        def.args().iter().for_each(|t| debug!("arg: {:?}", t));
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
pub fn find_name(text: &str) -> (&str, usize) {
    if !text.contains('(') {
        return ("", 0);
    }
    let end = text.rfind('(').unwrap_or(0);
    let start = text[..end]
        .rfind(|c: char| !c.is_alphabetic() && !c.is_ascii_digit() && c != '_')
        .map(|i| i + 1)
        .unwrap_or(end);
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
    }
}
