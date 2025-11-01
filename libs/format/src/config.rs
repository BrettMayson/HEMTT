use logos::Logos;

use crate::FormatterConfig;

/// Check if we're in a context where minus should be treated as a negative number in config files
fn is_negative_number_context_config(output: &str) -> bool {
    output.ends_with('{')
        || output.ends_with('[')
        || output.ends_with('(')
        || output.ends_with("= ") || output.ends_with('=') || output.ends_with("= -")
        || output.ends_with(", ") || output.ends_with(", -")
        // Arithmetic operators - negative numbers can appear after them
        || output.ends_with("+ ") || output.ends_with("+ -")
        || output.ends_with("- ") || output.ends_with("- -")
        || output.ends_with("* ") || output.ends_with("* -")
        || output.ends_with("/ ") || output.ends_with("/ -")
        || output.ends_with("% ") || output.ends_with("% -")
}

/// Format nested braces with proper spacing
fn format_nested_braces(content: &str) -> String {
    if content.starts_with('{') && content.ends_with('}') {
        let inner = &content[1..content.len() - 1];
        let parts: Vec<&str> = inner.split(',').map(str::trim).collect();
        format!("{{{}}}", parts.join(", "))
    } else {
        content.to_string()
    }
}

/// Parse config array items from content
fn parse_config_array_items(content: &str) -> Vec<String> {
    let mut items = Vec::new();
    let mut current_item = String::new();
    let mut brace_depth = 0;
    let mut string_delimiter = None;
    let mut chars = content.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '"' if string_delimiter.is_none() => {
                current_item.push(ch);
                string_delimiter = Some(ch);
            }
            '"' if Some(ch) == string_delimiter => {
                current_item.push(ch);
                // Check if this is an escaped quote (doubled)
                if chars.peek() == Some(&ch) {
                    if let Some(escaped_ch) = chars.next() {
                        current_item.push(escaped_ch);
                    }
                } else {
                    string_delimiter = None;
                }
            }
            '{' if string_delimiter.is_none() => {
                current_item.push(ch);
                brace_depth += 1;
            }
            '}' if string_delimiter.is_none() => {
                current_item.push(ch);
                brace_depth -= 1;
            }
            ',' if string_delimiter.is_none() && brace_depth == 0 => {
                let trimmed = current_item.trim();
                if !trimmed.is_empty() {
                    // Format nested braces with proper spacing
                    let formatted = if trimmed.starts_with('{') && trimmed.ends_with('}') {
                        format_nested_braces(trimmed)
                    } else {
                        trimmed.to_string()
                    };
                    items.push(formatted);
                }
                current_item.clear();
            }
            _ => {
                current_item.push(ch);
            }
        }
    }

    // Add the last item
    let trimmed = current_item.trim();
    if !trimmed.is_empty() {
        // Format nested braces with proper spacing
        let formatted = if trimmed.starts_with('{') && trimmed.ends_with('}') {
            format_nested_braces(trimmed)
        } else {
            trimmed.to_string()
        };
        items.push(formatted);
    }

    items
}

/// Format config array content with intelligent line breaking
fn format_config_array_content(
    content: &str,
    cfg: &FormatterConfig,
    indent_level: usize,
) -> String {
    let items = parse_config_array_items(content);

    if items.is_empty() {
        return String::new();
    }

    // Calculate one-line format
    let one_line = items.join(", ");
    let line_length = one_line.len();

    // Use single-line format for short arrays
    if line_length <= 100 {
        return one_line;
    }

    // Format with line breaks - try to fit multiple items per line like original
    let mut result = String::from("\n");
    result.push_str(&cfg.indent(indent_level + 1));

    let mut current_line_length = cfg.indent(indent_level + 1).len();
    let mut first_item = true;

    for item in items {
        let item_length = if first_item {
            item.len()
        } else {
            2 + item.len() // ", " + item
        };

        // Start a new line if this item would make the line too long
        if !first_item && current_line_length + item_length > 100 {
            result.push_str(",\n");
            result.push_str(&cfg.indent(indent_level + 1));
            result.push_str(&item);
            current_line_length = cfg.indent(indent_level + 1).len() + item.len();
        } else {
            if !first_item {
                result.push_str(", ");
            }
            result.push_str(&item);
            current_line_length += item_length;
        }

        first_item = false;
    }

    result.push('\n');
    result
}

#[derive(Logos, Debug, PartialEq)]
enum Token {
    // Keywords and identifiers
    #[regex(r"[A-Za-z_][A-Za-z0-9_]*")]
    Ident,

    // Numbers (integers, decimals, and hexadecimal)
    #[regex(r"0[xX][0-9a-fA-F]+|[0-9]+(\.[0-9]+)?")]
    Number,

    // Strings
    #[regex(r#""([^"\\]|\\.)*""#)]
    DoubleString,
    #[regex(r"'([^'\\\\]|\\\\.)*'")]
    SingleString,

    // Operators and punctuation
    #[token("=")]
    Eq,
    #[token(";")]
    Semi,
    #[token(",")]
    Comma,
    #[token(":")]
    Colon,
    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token("[")]
    LBracket,
    #[token("]")]
    RBracket,

    // Arithmetic operators
    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Multiply,
    #[token("/")]
    Divide,
    #[token("%")]
    Modulo,
    #[token(">>")]
    Config,
    #[token("\\")]
    Backslash,
    #[token(".")]
    Dot,

    // Preprocessor lines
    #[regex(r"#.*?", priority = 2)]
    Preprocessor,

    // Comments
    #[regex(r"//[^\n]*?")]
    LineComment,
    #[regex(r"/\*([^*]|\*[^/])*\*/")]
    BlockComment,

    // Whitespace tokens - we need to handle these to preserve empty lines
    #[regex(r"[ \t]+")]
    Space,
    #[regex(r"\n")]
    Newline,
}

#[allow(clippy::too_many_lines)]
/// Format a configuration file according to the provided `FormatterConfig`.
/// Returns the formatted string or an error message if formatting fails.
///
/// # Errors
/// Returns an error if the input contains unexpected tokens.
pub fn format_config(source: &str, cfg: &FormatterConfig) -> Result<String, String> {
    let mut lexer = Token::lexer(source);
    let mut output = String::new();
    let mut indent_level = 0;
    let mut need_indent = true;
    let mut in_array = false;
    let mut array_content = String::new();
    let mut brace_count = 0;
    let mut consecutive_newlines = 0;
    let mut after_paren = false;
    let mut newline_since_last_token = false;

    while let Some(token) = lexer.next() {
        let Ok(token) = token else {
            let span = lexer.span();
            let error_text = lexer.source().get(span.clone()).unwrap_or("unknown");
            return Err(format!(
                "Unexpected token '{}' at position {}..{}",
                error_text, span.start, span.end
            ));
        };

        match token {
            Token::Space => {
                // Skip spaces - we handle indentation ourselves
                consecutive_newlines = 0;
            }
            Token::Newline => {
                consecutive_newlines += 1;
                newline_since_last_token = true;
                // Preserve empty lines - when we see the second newline, it represents an empty line
                if consecutive_newlines == 1 {
                    // First newline - end the current line if not already ended
                    if !output.ends_with('\n') {
                        output.push('\n');
                    }
                } else if consecutive_newlines >= 2 {
                    // Additional newlines represent empty lines
                    output.push('\n');
                }
            }
            Token::Preprocessor => {
                consecutive_newlines = 0;
                if !output.is_empty() && !output.ends_with('\n') {
                    output.push('\n');
                }
                output.push_str(&cfg.indent(indent_level));
                output.push_str(lexer.slice().trim());
                output.push('\n');
                need_indent = true;
            }
            Token::LineComment => {
                consecutive_newlines = 0;

                if need_indent || newline_since_last_token {
                    // Comment at beginning of line or after a newline
                    if newline_since_last_token && !output.ends_with('\n') {
                        // Start a new line for the comment only if we're not already on a new line
                        output.push('\n');
                    }
                    output.push_str(&cfg.indent(indent_level));
                } else {
                    // True inline comment - on same line as previous content
                    output.push(' ');
                }
                output.push_str(lexer.slice().trim_end());
                output.push('\n');
                need_indent = true;
                newline_since_last_token = false;
            }
            Token::BlockComment => {
                consecutive_newlines = 0;
                let comment = lexer.slice();
                // Check if this is a multi-line comment
                let is_multiline = comment.contains('\n');

                if is_multiline {
                    // Multi-line block comment: treat like a line comment
                    if need_indent {
                        output.push_str(&cfg.indent(indent_level));
                    } else {
                        output.push(' ');
                    }
                    output.push_str(comment.trim_end());
                    output.push('\n');
                    need_indent = true;
                } else {
                    // Inline block comment: keep on same line
                    if !need_indent {
                        output.push(' ');
                    }
                    output.push_str(comment.trim());
                    need_indent = false;
                }
            }
            Token::LBracket => {
                consecutive_newlines = 0;
                // Don't add space before [ in array declarations
                output.push('[');
                need_indent = false;
            }
            Token::RBracket => {
                consecutive_newlines = 0;
                output.push(']');
                need_indent = false;
            }
            Token::LBrace => {
                consecutive_newlines = 0;
                if in_array {
                    brace_count += 1;
                    array_content.push('{');
                } else {
                    // Check if this is starting an array by looking ahead
                    let remaining = lexer.remainder();
                    if cfg.space_before_brace && !output.ends_with(' ') {
                        output.push(' ');
                    }
                    output.push('{');
                    if is_array_content(remaining) {
                        in_array = true;
                        array_content.clear();
                        brace_count = 0;
                    } else if is_empty_class(remaining) {
                        need_indent = false;
                    } else {
                        // Check if there's a comment on the same line (only whitespace before //)
                        let trimmed_remaining = remaining.trim_start();
                        if trimmed_remaining.starts_with("//") {
                            // There's a comment right after this brace on the same line
                            // Don't add newline, let comment handler manage the line ending
                            indent_level += 1;
                            need_indent = false;
                        } else {
                            // No immediate comment, proceed normally
                            output.push('\n');
                            indent_level += 1;
                            need_indent = true;
                        }
                    }
                }
            }
            Token::RBrace => {
                consecutive_newlines = 0;
                if in_array {
                    if brace_count > 0 {
                        brace_count -= 1;
                        array_content.push('}');
                    } else {
                        // End of array, format using local array function
                        let formatted =
                            format_config_array_content(&array_content, cfg, indent_level);

                        output.push_str(&formatted);
                        output.push('}');
                        in_array = false;
                        need_indent = false;
                    }
                } else {
                    // Check if this is an empty class (no newline before the brace)
                    if output.ends_with('{') {
                        // This is an empty class like "class Name {}"
                        output.push('}');
                        need_indent = false;
                    } else {
                        // Normal class ending
                        indent_level = indent_level.saturating_sub(1);
                        output.push_str(&cfg.indent(indent_level));
                        output.push('}');
                        output.push('\n');
                        need_indent = true;
                    }
                }
            }
            Token::Semi => {
                consecutive_newlines = 0;
                newline_since_last_token = false;
                if in_array {
                    // Don't add semicolons to array content, they are handled in formatting
                } else {
                    // If the last character is '}', put semicolon on same line
                    if output.trim_end().ends_with('}') {
                        // Remove the trailing newline from the previous RBrace
                        let trimmed = output.trim_end();
                        output.truncate(trimmed.len());
                    }
                    output.push(';');

                    // Look ahead to see if there's a comment on the same line
                    let remaining = lexer.remainder();
                    let next_line_comment = remaining.trim_start().starts_with("//");

                    if next_line_comment {
                        // Don't add newline, let the comment handler decide
                        need_indent = false;
                    } else {
                        output.push('\n');
                        need_indent = true;
                    }
                    after_paren = false;
                }
            }
            Token::Comma => {
                consecutive_newlines = 0;
                if in_array {
                    array_content.push(',');
                } else {
                    output.push(',');

                    // Check if there's already a space in the source after this comma
                    let span = lexer.span();
                    let source_after = &source[span.end..];

                    // Only add space if there's already one in the source (preserve original spacing)
                    if source_after.starts_with(' ') || source_after.starts_with('\t') {
                        output.push(' ');
                    }

                    need_indent = false;
                    // Don't set after_paren = false here, we're still inside parentheses
                }
            }
            Token::Colon => {
                consecutive_newlines = 0;
                if in_array {
                    array_content.push(':');
                } else {
                    output.push(':');
                    need_indent = false;
                }
            }
            Token::LParen => {
                consecutive_newlines = 0;
                if in_array {
                    array_content.push('(');
                } else {
                    output.push('(');
                    need_indent = false;
                    after_paren = true;
                }
            }
            Token::RParen => {
                consecutive_newlines = 0;
                if in_array {
                    array_content.push(')');
                } else {
                    output.push(')');
                    need_indent = false;
                    after_paren = false;
                }
            }
            // Backslash - used in file paths, no spaces around it
            Token::Backslash => {
                consecutive_newlines = 0;
                if in_array {
                    array_content.push('\\');
                } else {
                    if need_indent {
                        output.push_str(&cfg.indent(indent_level));
                        need_indent = false;
                    }
                    output.push('\\');
                }
            }
            // Dot - used in file extensions and member access, no spaces around it
            Token::Dot => {
                consecutive_newlines = 0;
                if in_array {
                    array_content.push('.');
                } else {
                    if need_indent {
                        output.push_str(&cfg.indent(indent_level));
                        need_indent = false;
                    }
                    output.push('.');
                }
            }
            // Arithmetic operators - add spaces around them, except for negative numbers
            Token::Plus | Token::Multiply | Token::Divide | Token::Modulo | Token::Config => {
                consecutive_newlines = 0;
                if in_array {
                    array_content.push_str(lexer.slice());
                } else {
                    if need_indent {
                        output.push_str(&cfg.indent(indent_level));
                        need_indent = false;
                    } else if !output.ends_with(' ') {
                        // Add space before operator
                        output.push(' ');
                    }
                    output.push_str(lexer.slice());
                    output.push(' '); // Add space after operator
                    newline_since_last_token = false;
                    after_paren = false;
                }
            }
            Token::Minus => {
                consecutive_newlines = 0;
                if in_array {
                    array_content.push_str(lexer.slice());
                } else {
                    if need_indent {
                        output.push_str(&cfg.indent(indent_level));
                        need_indent = false;
                    } else if is_negative_number_context_config(&output) {
                        // Negative number - no space before minus
                    } else if !output.ends_with(' ') {
                        // Arithmetic operation - add space before
                        output.push(' ');
                    }
                    output.push_str(lexer.slice());

                    // Check if this should have space after (arithmetic) or not (negative number)
                    let remaining = lexer.remainder().trim_start();
                    let next_char = remaining.chars().next();
                    if next_char.is_some_and(|c| c.is_ascii_digit()) {
                        // Next token is a digit - likely negative number, no space after
                    } else {
                        // Arithmetic operation - add space after
                        output.push(' ');
                    }
                    newline_since_last_token = false;
                    after_paren = false;
                }
            }
            Token::Eq => {
                consecutive_newlines = 0;
                if in_array {
                    array_content.push_str(lexer.slice());
                } else {
                    if need_indent {
                        output.push_str(&cfg.indent(indent_level));
                        need_indent = false;
                    } else if !output.ends_with(' ') {
                        // Add space before equals
                        output.push(' ');
                    }
                    output.push_str(lexer.slice());
                    output.push(' '); // Always add space after equals
                    newline_since_last_token = false;
                    after_paren = false;
                }
            }
            _ => {
                consecutive_newlines = 0;
                if in_array {
                    array_content.push_str(lexer.slice());
                } else {
                    if need_indent {
                        output.push_str(&cfg.indent(indent_level));
                        need_indent = false;
                    } else if !after_paren && !matches!(token, Token::LBracket | Token::RBracket) {
                        // Check if we should add space before this token
                        let ends_with_minus = output.ends_with('-');
                        let is_number = matches!(token, Token::Number);
                        let is_negative_number_context = ends_with_minus
                            && is_number
                            && is_negative_number_context_config(&output);

                        let should_add_space = !(output.ends_with(' ')
                            || output.ends_with('(')
                            || output.ends_with('[')
                            || output.ends_with('=')
                            || output.ends_with('{')
                            || output.ends_with(',')
                            // Special case: don't add space after minus when followed by a number (negative number)
                            || is_negative_number_context
                            // Don't add space before braces - let LBrace handler manage spacing
                            || matches!(token, Token::LBrace));

                        if should_add_space {
                            output.push(' ');
                        }
                    }
                    output.push_str(lexer.slice());
                    newline_since_last_token = false;
                    // Don't automatically set after_paren = false here
                    // Let it be managed by specific token handlers
                }
            }
        }
    }

    Ok(output.trim_end().to_string() + "\n")
}

fn is_array_content(content: &str) -> bool {
    // Heuristic: if it contains mostly numbers, commas, and braces, it's likely an array
    let trimmed = content.trim();
    if trimmed.is_empty() {
        return false;
    }

    // Find the content up to the first unmatched closing brace
    let mut brace_depth = 0;
    let mut array_end = None;
    let chars = trimmed.char_indices();

    for (i, ch) in chars {
        match ch {
            '{' => brace_depth += 1,
            '}' => {
                if brace_depth == 0 {
                    array_end = Some(i);
                    break;
                }
                brace_depth -= 1;
            }
            _ => {}
        }
    }

    // Get just the content within the braces
    let array_content = if let Some(end) = array_end {
        &trimmed[..end]
    } else {
        return false; // No closing brace found
    };

    if array_content.is_empty() {
        return false;
    }

    // Check if the content looks like an array (numbers, letters for macros, commas, braces, quotes, parentheses, decimal points)
    let chars = array_content.chars();
    let mut has_content = false;

    for ch in chars {
        match ch {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '_' | '{' | '}' | '"' | '(' | ')' | '.' => {
                has_content = true;
            }
            ',' | ' ' | '\t' | '\n' => {
                // These are fine
            }
            _ => return false, // Something else, not an array
        }
    }

    has_content
}

fn is_empty_class(content: &str) -> bool {
    // Check if the content is empty (only whitespace before closing brace)
    let trimmed = content.trim();
    trimmed.starts_with('}')
}
