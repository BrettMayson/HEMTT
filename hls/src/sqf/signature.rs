use arma3_wiki::model::{Call, Command, Syntax};
use tower_lsp::lsp_types::{
    ParameterInformation, ParameterLabel, SignatureHelp, SignatureHelpParams, SignatureInformation,
};

use crate::{files::FileCache, workspace::EditorWorkspaces};

use super::SqfAnalyzer;

impl SqfAnalyzer {
    /// Signature help for an Arma 3 command near the cursor.
    ///
    /// Only the parameter *names* are used, not their types, since the wiki's
    /// type data isn't reliably parsed yet.
    pub async fn signature_help(&self, params: &SignatureHelpParams) -> Option<SignatureHelp> {
        let url = &params.text_document_position_params.text_document.uri;
        if !std::path::Path::new(url.path())
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("sqf"))
        {
            return None;
        }
        let workspace = EditorWorkspaces::get().guess_workspace_retry(url).await?;
        let database = self.get_database(&workspace).ok()?;
        let text = FileCache::get().text(url)?;
        let position = params.text_document_position_params.position;
        let line = text.lines().nth(position.line as usize)?;
        let line: String = line.chars().take(position.character as usize).collect();
        let word = nearest_command_word(&line)?;
        let command = database.wiki().commands().get(word)?;
        if command.syntax().is_empty() {
            return None;
        }
        Some(SignatureHelp {
            signatures: command
                .syntax()
                .iter()
                .map(|syntax| signature_information(command, syntax))
                .collect(),
            active_signature: Some(0),
            active_parameter: None,
        })
    }
}

/// `lhs` is always already fully typed by the time `nearest_command_word`
/// resolves to a binary command (the command comes after the `lhs` in text),
/// so the parameter currently being typed is always the `rhs`.
fn signature_information(command: &Command, syntax: &Syntax) -> SignatureInformation {
    let (label, parameters, active_parameter) = match syntax.call() {
        Call::Nular => (command.name().to_string(), Vec::new(), None),
        Call::Unary(rhs) => {
            let rhs_label = format_args_group(&rhs.names());
            (
                format!("{} {rhs_label}", command.name()),
                vec![parameter_information(&rhs_label)],
                Some(0),
            )
        }
        Call::Binary(lhs, rhs) => {
            let lhs_label = format_args_group(&lhs.names());
            let rhs_label = format_args_group(&rhs.names());
            (
                format!("{lhs_label} {} {rhs_label}", command.name()),
                vec![
                    parameter_information(&lhs_label),
                    parameter_information(&rhs_label),
                ],
                Some(1),
            )
        }
    };
    SignatureInformation {
        label,
        documentation: None,
        parameters: Some(parameters),
        active_parameter,
    }
}

fn parameter_information(label: &str) -> ParameterInformation {
    ParameterInformation {
        label: ParameterLabel::Simple(label.to_string()),
        documentation: None,
    }
}

/// A single name is shown bare; multiple (an array argument) are wrapped in
/// `[...]` so it's clear the command expects an array there.
fn format_args_group(names: &[String]) -> String {
    if names.len() == 1 {
        names[0].clone()
    } else {
        format!("[{}]", names.join(", "))
    }
}

/// The nearest command-like word before the cursor: the word immediately
/// before the cursor if it's already a full word, otherwise the first
/// alphabetic word found scanning backwards, skipping over quoted strings
/// and non-alphabetic tokens (numbers, punctuation) along the way. Stops at
/// the start of the current statement (`;`, `{`, `}`) so a previous
/// statement's (or the enclosing block's) command is never picked up.
fn nearest_command_word(line: &str) -> Option<&str> {
    let bytes = line.as_bytes();
    let mut i = line.len();
    while i > 0 {
        let b = bytes[i - 1];
        if b == b';' || b == b'{' || b == b'}' {
            return None;
        }
        if b == b'"' || b == b'\'' {
            let quote = b;
            i -= 1;
            while i > 0 && bytes[i - 1] != quote {
                i -= 1;
            }
            i = i.saturating_sub(1);
            continue;
        }
        if is_word_byte(b) {
            let end = i;
            while i > 0 && is_word_byte(bytes[i - 1]) {
                i -= 1;
            }
            let word = &line[i..end];
            if word.starts_with(|c: char| c.is_ascii_alphabetic()) {
                return Some(word);
            }
            continue;
        }
        i -= 1;
    }
    None
}

const fn is_word_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

#[cfg(test)]
mod tests {
    use super::{format_args_group, nearest_command_word};

    #[test]
    fn format_args_group_wraps_multiple_names_in_brackets() {
        assert_eq!(format_args_group(&["unit".to_string()]), "unit");
        assert_eq!(
            format_args_group(&["name".to_string(), "value".to_string()]),
            "[name, value]"
        );
    }

    #[test]
    fn finds_word_immediately_before_cursor() {
        assert_eq!(nearest_command_word("hint"), Some("hint"));
        assert_eq!(nearest_command_word("_unit setDamage"), Some("setDamage"));
    }

    #[test]
    fn skips_numbers_and_strings_while_scanning_back() {
        assert_eq!(
            nearest_command_word("_unit setDamage 0.5"),
            Some("setDamage")
        );
        assert_eq!(nearest_command_word("hint \"hello world\""), Some("hint"));
    }

    #[test]
    fn empty_line_has_no_word() {
        assert_eq!(nearest_command_word(""), None);
        assert_eq!(nearest_command_word("   "), None);
    }

    /// Regression: signature help shouldn't bleed into the next statement.
    #[test]
    fn stops_at_statement_and_block_boundaries() {
        assert_eq!(nearest_command_word("hint \"a\"; "), None);
        assert_eq!(nearest_command_word("if (true) then { "), None);
        assert_eq!(nearest_command_word("};"), None);
    }
}
