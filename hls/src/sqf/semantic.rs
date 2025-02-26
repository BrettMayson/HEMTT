use std::sync::Arc;

use hemtt_sqf::parser::database::Database;
use hemtt_workspace::{WorkspacePath, reporting::Symbol};
use tower_lsp::lsp_types::SemanticToken;
use tracing::warn;
use url::Url;

use super::SqfAnalyzer;

impl SqfAnalyzer {
    pub async fn process_semantic_tokens(
        &self,
        url: Url,
        text: String,
        source: WorkspacePath,
        database: Arc<Database>,
    ) {
        let Ok(tokens) = hemtt_preprocessor::parse::str(&text, &source) else {
            warn!("Failed to parse file");
            return;
        };
        self.tokens.insert(url.clone(), tokens.clone());
        let mut semantic_tokens = Vec::new();
        let mut last_line = 0;
        let mut last_col = 0;
        let mut in_string = false;
        let mut var_name = false;
        for token in tokens {
            match token.symbol() {
                Symbol::DoubleQuote => in_string = !in_string,
                Symbol::Underscore => var_name = true,
                Symbol::Word(word) => {
                    if in_string {
                        continue;
                    }
                    if var_name {
                        var_name = false;
                        continue;
                    }
                    if [
                        "call",
                        "callExtension",
                        "compile",
                        "compileFinal",
                        "exec",
                        "execFSM",
                        "execVM",
                        "private",
                        "spawn",
                        "true",
                        "false",
                    ]
                    .contains(&word.as_str())
                    {
                        continue;
                    }
                    let Some(cmd) = database.wiki().commands().get(word) else {
                        continue;
                    };
                    if cmd.groups().iter().any(|x| x == "Program Flow") {
                        continue;
                    }
                    let line = token.position().start().line() as u32 - 1;
                    let col = token.position().start().column() as u32;
                    if line != last_line {
                        last_col = 0;
                    }
                    let delta_line = line - last_line;
                    let delta_start = col - last_col;
                    let token = SemanticToken {
                        delta_line,
                        delta_start,
                        length: word.len() as u32,
                        token_type: 0,
                        token_modifiers_bitset: 0,
                    };
                    last_line = line;
                    last_col = col;
                    semantic_tokens.push(token);
                }
                _ => {
                    var_name = false;
                }
            }
        }
        Self::get()
            .semantic
            .write()
            .await
            .insert(url, semantic_tokens);
    }

    pub async fn get_tokens(&self, url: &Url) -> Option<Vec<SemanticToken>> {
        self.semantic.read().await.get(url).cloned()
    }
}
