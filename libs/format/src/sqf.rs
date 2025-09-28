use logos::Logos;

use crate::FormatterConfig;

#[derive(Logos, Debug, PartialEq, Eq)]
pub enum SqfToken {
    // Keywords and identifiers (SQF is case-insensitive, but we’ll preserve case)
    #[regex(r"[A-Za-z_][A-Za-z0-9_]*")]
    Ident,

    // Numbers (decimal, hexadecimal, and scientific notation)
    #[regex(r"0x[0-9A-Fa-f]+|[0-9]+(\.[0-9]+)?([eE][+-]?[0-9]+)?")]
    Number,

    // Strings - SQF style with proper escaping (doubled quotes)
    #[regex(r#""([^"]|"")*""#)]
    DoubleString,
    #[regex(r"'([^']|'')*'")]
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
    #[token("^")]
    Power,

    // Comparison operators
    #[token("==")]
    Equal,
    #[token("!=")]
    NotEqual,
    #[token("<")]
    Less,
    #[token("<=")]
    LessEqual,
    #[token(">")]
    Greater,
    #[token(">=")]
    GreaterEqual,

    // Logical operators
    #[token("&&")]
    And,
    #[token("||")]
    Or,
    #[token("!")]
    Not,

    // Other common SQF operators and symbols
    #[token(">>")]
    Config,
    #[token(".")]
    Dot,
    #[token("?")]
    Question,
    #[token("@")]
    At,
    #[token("$")]
    Dollar,
    #[token("&")]
    Ampersand,
    #[token("|")]
    Pipe,
    #[token("~")]
    Tilde,
    #[token("`")]
    Backtick,
    #[token("#")]
    Hash,

    // Preprocessor lines (only at start of line with specific keywords)
    #[regex(
        r"#(include|define|ifdef|ifndef|endif|undef|if|else|elif|pragma).*?",
        priority = 2
    )]
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
/// Format SQF code according to the provided `FormatterConfig`.
/// Returns the formatted string or an error message if formatting fails.
///
/// # Errors
/// Returns an error if the input contains unexpected tokens.
pub fn format_sqf(source: &str, cfg: &FormatterConfig) -> Result<String, String> {
    let mut lexer = SqfToken::lexer(source);
    let mut output = String::new();
    let mut indent_level = 0;
    let mut need_indent = true;
    let mut consecutive_newlines = 0;
    let mut bracket_depth: i32 = 0;
    let mut paren_depth: i32 = 0;
    let mut seen_newline_since_semi = true; // Track if we've seen a newline token since the last semicolon

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
            SqfToken::Space => {
                // Skip spaces - we handle indentation ourselves
                consecutive_newlines = 0;
            }
            SqfToken::Newline => {
                consecutive_newlines += 1;
                seen_newline_since_semi = true; // We've seen a newline token
                // Only add newlines to the output if we have actual content already
                // This prevents leading newlines in the output
                if !output.is_empty() {
                    // Add a newline to the output for the first newline (unless we already have one)
                    if consecutive_newlines == 1 && !output.ends_with('\n') {
                        output.push('\n');
                    }
                    // Preserve empty lines - when we see the second newline, it represents an empty line
                    // For each newline after the first, add it to the output
                    if consecutive_newlines >= 2 {
                        output.push('\n');
                    }
                }
                need_indent = true;
            }
            SqfToken::Preprocessor => {
                consecutive_newlines = 0;
                if !output.is_empty() && !output.ends_with('\n') {
                    output.push('\n');
                }
                output.push_str(lexer.slice().trim_end());
                output.push('\n');
                need_indent = true;
            }
            SqfToken::LineComment => {
                consecutive_newlines = 0;

                // Special case: if the comment appears right after a semicolon (without any newline tokens in between)
                // then it should be treated as an end-of-line comment
                if need_indent && output.ends_with(";\n") && !seen_newline_since_semi {
                    // Remove the newline and put the comment on the same line as the semicolon
                    output.pop(); // Remove the '\n'
                    output.push(' ');
                } else if need_indent {
                    output.push_str(&cfg.indent(indent_level));
                } else {
                    output.push(' ');
                }
                output.push_str(lexer.slice().trim_end());
                output.push('\n');
                need_indent = true;
            }
            SqfToken::BlockComment => {
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
            SqfToken::LBrace => {
                consecutive_newlines = 0;
                if need_indent {
                    // At beginning of line, add proper indentation
                    output.push_str(&cfg.indent(indent_level));
                } else if cfg.space_before_brace && !output.ends_with(' ') && !output.ends_with('[')
                {
                    // In middle of line, add space if needed, but not after [
                    output.push(' ');
                }

                output.push('{');
                output.push('\n');
                indent_level += 1;
                need_indent = true;
            }
            SqfToken::RBrace => {
                consecutive_newlines = 0;
                if !need_indent {
                    output.push('\n');
                }
                indent_level = indent_level.saturating_sub(1);
                output.push_str(&cfg.indent(indent_level));
                output.push('}');
                need_indent = false;
            }
            SqfToken::Semi => {
                consecutive_newlines = 0;
                // If the last character is '}', put semicolon on same line
                if output.trim_end().ends_with('}') {
                    // Remove the trailing newline from the previous RBrace
                    let trimmed = output.trim_end();
                    output.truncate(trimmed.len());
                }
                output.push(';');
                output.push('\n');
                need_indent = true;
                seen_newline_since_semi = false; // We haven't seen a newline token since this semicolon
            }
            SqfToken::Comma => {
                consecutive_newlines = 0;
                output.push(',');

                // Check if we're inside a macro call (all caps identifier)
                let in_macro_call = is_in_macro_call(&output);

                // Context-aware comma spacing:
                // - In macro calls: preserve original spacing
                // - In arrays (bracket_depth > 0): always add space (unless in macro)
                // - In function calls (paren_depth > 0): preserve original spacing
                // - In other contexts: add space
                if in_macro_call || paren_depth > 0 {
                    // Inside macros or function calls - preserve original spacing
                    let span = lexer.span();
                    let source_after = &source[span.end..];
                    if source_after.starts_with(' ') || source_after.starts_with('\t') {
                        output.push(' ');
                    }
                } else if bracket_depth > 0 {
                    // Inside arrays - always add space for consistency
                    output.push(' ');
                } else {
                    // Other contexts - add space
                    output.push(' ');
                }

                need_indent = false;
            }
            // Minus operator - special handling for negative numbers
            SqfToken::Minus => {
                consecutive_newlines = 0;
                if need_indent {
                    output.push_str(&cfg.indent(indent_level));
                } else if is_negative_number_context(&output) {
                    // Negative number - no space before minus
                } else if !output.ends_with(' ') {
                    // Arithmetic operation - add space before
                    output.push(' ');
                }
                output.push_str(lexer.slice());

                // Check if this is likely a negative number (no space after) or arithmetic (space after)
                let remaining = lexer.remainder().trim_start();
                let should_add_space = remaining.chars().next().is_none_or(|next_char| {
                    if next_char.is_ascii_digit() {
                        // Next token starts with digit - likely negative number, no space
                        false
                    } else if next_char == '-' {
                        // Check if this looks like "- -number" which should become "- -number" (no space)
                        let after_minus = remaining.strip_prefix('-').unwrap_or("").trim_start();
                        !after_minus
                            .chars()
                            .next()
                            .is_some_and(|c| c.is_ascii_digit())
                    } else {
                        // Arithmetic operation - add space after
                        true
                    }
                });

                if should_add_space {
                    output.push(' ');
                }
                need_indent = false;
            }
            // Arithmetic and comparison operators - add spaces around them
            SqfToken::Plus
            | SqfToken::Multiply
            | SqfToken::Divide
            | SqfToken::Modulo
            | SqfToken::Power
            | SqfToken::Equal
            | SqfToken::NotEqual
            | SqfToken::Less
            | SqfToken::LessEqual
            | SqfToken::Greater
            | SqfToken::GreaterEqual
            | SqfToken::And
            | SqfToken::Or
            | SqfToken::Eq
            | SqfToken::Hash => {
                consecutive_newlines = 0;
                if need_indent {
                    output.push_str(&cfg.indent(indent_level));
                } else if !output.ends_with(' ') {
                    output.push(' ');
                }
                output.push_str(lexer.slice());
                output.push(' ');
                need_indent = false;
            }
            // Left parenthesis - add space before if preceded by certain keywords
            SqfToken::LParen => {
                consecutive_newlines = 0;
                paren_depth += 1;
                if need_indent {
                    output.push_str(&cfg.indent(indent_level));
                } else if should_add_space_before_paren(&output) && !output.ends_with(' ') {
                    output.push(' ');
                }
                output.push_str(lexer.slice());
                need_indent = false;
            }
            // Left bracket - track depth for array formatting
            SqfToken::LBracket => {
                consecutive_newlines = 0;
                if need_indent {
                    output.push_str(&cfg.indent(indent_level));
                } else if should_add_space_before_bracket(&output) && !output.ends_with(' ') {
                    // Add space before [ when preceded by certain keywords or in certain contexts
                    output.push(' ');
                }
                output.push('[');
                bracket_depth += 1;
                need_indent = false;
            }
            // Right bracket - track depth
            SqfToken::RBracket => {
                consecutive_newlines = 0;
                bracket_depth = bracket_depth.saturating_sub(1);
                output.push(']');
                need_indent = false;
            }
            // String literals - preserve them exactly but add spacing
            SqfToken::DoubleString | SqfToken::SingleString => {
                consecutive_newlines = 0;
                if need_indent {
                    output.push_str(&cfg.indent(indent_level));
                } else if !output.ends_with(' ')
                    && !output.ends_with('(')
                    && !output.ends_with('[')
                    && !output.ends_with('=')
                    && !output.ends_with('{')
                {
                    output.push(' ');
                }
                output.push_str(lexer.slice());
                need_indent = false;
            }
            // Dot operator - no spaces around it
            SqfToken::Dot => {
                consecutive_newlines = 0;
                if need_indent {
                    output.push_str(&cfg.indent(indent_level));
                }
                output.push_str(lexer.slice());
            }
            // Not operator - add space before it when following identifiers
            SqfToken::Not => {
                consecutive_newlines = 0;
                if need_indent {
                    output.push_str(&cfg.indent(indent_level));
                } else if should_add_space_before_not(&output) && !output.ends_with(' ') {
                    output.push(' ');
                }
                output.push_str(lexer.slice());
            }
            // Right parenthesis - might need space after it
            SqfToken::RParen => {
                consecutive_newlines = 0;
                paren_depth -= 1;
                if need_indent {
                    output.push_str(&cfg.indent(indent_level));
                }
                output.push_str(lexer.slice());
                need_indent = false;
            }
            // Everything else - add space before if needed
            _ => {
                consecutive_newlines = 0;
                // Special case for 'else' - should be on same line as previous '}'
                if lexer.slice() == "else" && output.trim_end().ends_with('}') {
                    // Remove the previous newline if it exists
                    if output.ends_with('\n') {
                        output.pop();
                    }
                    output.push(' ');
                    output.push_str(lexer.slice());
                    need_indent = false;
                } else if need_indent {
                    output.push_str(&cfg.indent(indent_level));
                    need_indent = false;
                    output.push_str(lexer.slice());
                } else {
                    // Check if we should add space before this token
                    let ends_with_minus = output.ends_with('-');
                    let is_number = matches!(token, SqfToken::Number);
                    let is_negative_number_context =
                        ends_with_minus && is_number && is_negative_number_context(&output);

                    let should_add_space = !(output.ends_with(' ')
                        || output.ends_with('(')
                        || output.ends_with('[')
                        || output.ends_with('=')
                        || output.ends_with('{')
                        || output.ends_with(',') && paren_depth > 0
                        // Special case: don't add space after minus when followed by a number (negative number)
                        || is_negative_number_context);

                    if should_add_space {
                        output.push(' ');
                    }
                    output.push_str(lexer.slice());
                }
            }
        }
    }

    // Post-process to format arrays nicely
    let formatted = post_process_arrays(&output, cfg);

    // Post-process to fix negative numbers (remove spaces) - do this first
    let formatted = post_process_negative_numbers(&formatted);

    // Post-process to convert simple blocks to single lines
    let formatted = post_process_single_line_blocks(&formatted);

    Ok(formatted.trim_end().to_string() + "\n")
}

fn should_add_space_before_paren(output: &str) -> bool {
    // Use naming conventions: macros (all caps) don't get spaces, functions do
    if let Some(last_word) = output.split_whitespace().last() {
        // Handle case where last_word might have non-identifier chars at the end
        // Extract just the identifier part from the end
        let mut identifier = String::new();
        for ch in last_word.chars().rev() {
            if ch.is_alphanumeric() || ch == '_' {
                identifier.insert(0, ch);
            } else {
                break;
            }
        }

        if identifier.is_empty() {
            return false;
        }

        // Check if the identifier is all uppercase (macro convention)
        if identifier
            .chars()
            .all(|c| c.is_uppercase() || c.is_numeric() || c == '_')
        {
            return false; // Macros like CSTRING() don't get spaces
        }

        // Check if it's a valid identifier that could be a function
        if identifier.chars().all(|c| c.is_alphanumeric() || c == '_') && !identifier.is_empty() {
            return true; // Functions like getText() get spaces
        }
    }

    false
}

fn should_add_space_before_not(output: &str) -> bool {
    // Don't add space if we're at the start or already have space
    if output.is_empty() || output.ends_with(' ') {
        return false;
    }

    // Don't add space if the last character is not alphanumeric or underscore
    // This covers operators, punctuation, etc.
    if let Some(last_char) = output.chars().last()
        && !(last_char.is_alphanumeric() || last_char == '_')
    {
        return false;
    }

    // Default: add space after identifiers
    true
}

fn is_in_macro_call(output: &str) -> bool {
    // Look for the last opening parenthesis and check if it's preceded by an all-caps identifier
    if let Some(last_paren_pos) = output.rfind('(') {
        // Get the text before the last opening parenthesis
        let before_paren = &output[..last_paren_pos];

        // Extract the last identifier (word) before the parenthesis
        if let Some(last_word_start) =
            before_paren.rfind(|c: char| !c.is_alphanumeric() && c != '_')
        {
            let identifier = &before_paren[last_word_start + 1..];

            // Check if the identifier is all uppercase (macro convention)
            if !identifier.is_empty()
                && identifier
                    .chars()
                    .all(|c| c.is_uppercase() || c.is_numeric() || c == '_')
                && identifier.chars().any(char::is_alphabetic)
            // Must contain at least one letter
            {
                return true;
            }
        } else {
            // No non-alphanumeric character found, so the identifier starts at the beginning
            let identifier = before_paren;
            if !identifier.is_empty()
                && identifier
                    .chars()
                    .all(|c| c.is_uppercase() || c.is_numeric() || c == '_')
                && identifier.chars().any(char::is_alphabetic)
            // Must contain at least one letter
            {
                return true;
            }
        }
    }

    false
}

fn should_add_space_before_bracket(output: &str) -> bool {
    // Don't add space if we're at the start, already have space, or after certain characters
    if output.is_empty() || output.ends_with(' ') || output.ends_with('(') || output.ends_with('[')
    {
        return false;
    }

    // Don't add space if the last character is not alphanumeric or underscore
    // This covers operators, punctuation, etc.
    if let Some(last_char) = output.chars().last()
        && !(last_char.is_alphanumeric() || last_char == '_')
    {
        return false;
    }

    // Default: add space after identifiers
    true
}

fn post_process_arrays(input: &str, cfg: &FormatterConfig) -> String {
    let mut result = String::new();
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        result.push(ch);
        if ch == '[' {
            // Found start of array, process it
            let array_content = extract_array_content(&mut chars);

            // Calculate current indentation level from the result
            let current_indent_level = calculate_current_indent_level(&result, cfg);

            let formatted_array =
                format_array_content_with_context(&array_content, cfg, current_indent_level);
            result.push_str(&formatted_array);
            result.push(']');
        }
    }

    result
}

fn extract_array_content(chars: &mut std::iter::Peekable<std::str::Chars>) -> String {
    let mut content = String::new();
    let mut bracket_depth = 1;
    let mut in_string = false;
    let mut string_delimiter = None;

    while let Some(ch) = chars.next() {
        match ch {
            '"' | '\'' if string_delimiter.is_none() => {
                content.push(ch);
                in_string = true;
                string_delimiter = Some(ch);
            }
            '"' | '\'' if Some(ch) == string_delimiter => {
                content.push(ch);
                // Check for escaped quotes (doubled)
                if chars.peek() == Some(&ch) {
                    if let Some(escaped_ch) = chars.next() {
                        content.push(escaped_ch);
                    }
                } else {
                    in_string = false;
                    string_delimiter = None;
                }
            }
            '[' if !in_string => {
                content.push(ch);
                bracket_depth += 1;
            }
            ']' if !in_string => {
                bracket_depth -= 1;
                if bracket_depth == 0 {
                    break;
                }
                content.push(ch);
            }
            _ => {
                content.push(ch);
            }
        }
    }

    content
}

fn format_array_content_with_context(
    content: &str,
    cfg: &FormatterConfig,
    base_indent_level: usize,
) -> String {
    if content.trim().is_empty() {
        return String::new();
    }

    let items = parse_array_items_simple(content);

    if items.is_empty() {
        return String::new();
    }

    // Calculate one-line format
    let one_line = items.join(", ");
    let line_length = one_line.len();

    // Use single-line format for short arrays (like config formatter)
    if line_length <= 80 && items.len() <= 10 {
        return one_line;
    }

    // Check if this array contains code blocks - if so, keep it horizontally compact
    let has_code_blocks = items.iter().any(|item| {
        let trimmed = item.trim();
        trimmed.contains('{') && trimmed.contains('}') // Contains code blocks
    });

    if has_code_blocks {
        // For arrays with code blocks, keep the original compact format
        return one_line;
    }

    // Check if this array should use multi-line formatting for nested arrays only
    let has_many_nested_arrays = items.len() > 5
        && items.iter().any(|item| {
            let trimmed = item.trim();
            trimmed.starts_with('[') && trimmed.ends_with(']') // Nested arrays
        });

    let mut result = String::new();
    result.push('\n');
    if has_many_nested_arrays {
        // Format complex arrays with each item on its own line
        for (i, item) in items.iter().enumerate() {
            result.push_str(&cfg.indent(base_indent_level + 1));
            result.push_str(item.trim());
            if i < items.len() - 1 {
                result.push(',');
            }
            result.push('\n');
        }
    } else {
        // Format simple arrays with line-length-based packing
        result.push_str(&cfg.indent(base_indent_level + 1));

        let mut current_line_length = cfg.indent(base_indent_level + 1).len();
        let mut first_item = true;

        for item in items {
            let item_length = if first_item {
                item.len()
            } else {
                2 + item.len() // ", " + item
            };

            // Start a new line if this item would make the line too long
            if !first_item && current_line_length + item_length > 80 {
                result.push_str(",\n");
                result.push_str(&cfg.indent(base_indent_level + 1));
                result.push_str(item.trim());
                current_line_length = cfg.indent(base_indent_level + 1).len() + item.len();
            } else {
                if !first_item {
                    result.push_str(", ");
                }
                result.push_str(item.trim());
                current_line_length += item_length;
            }

            first_item = false;
        }

        result.push('\n');
    }
    result.push_str(&cfg.indent(base_indent_level));
    result
}

fn calculate_current_indent_level(result: &str, _cfg: &FormatterConfig) -> usize {
    // Get the current line (everything after the last newline)
    let current_line = result.lines().last().unwrap_or("");

    // Count leading spaces/tabs and convert to indent level
    let leading_spaces = current_line.len() - current_line.trim_start().len();

    // Assume 4 spaces per indent level (standard)
    leading_spaces / 4
}

fn parse_array_items_simple(content: &str) -> Vec<String> {
    let mut items = Vec::new();
    let mut current_item = String::new();
    let mut bracket_depth = 0;
    let mut in_string = false;
    let mut string_delimiter = None;

    for ch in content.chars() {
        match ch {
            '"' | '\'' if string_delimiter.is_none() => {
                current_item.push(ch);
                in_string = true;
                string_delimiter = Some(ch);
            }
            '"' | '\'' if Some(ch) == string_delimiter => {
                current_item.push(ch);
                in_string = false;
                string_delimiter = None;
            }
            '[' | '{' | '(' if !in_string => {
                current_item.push(ch);
                bracket_depth += 1;
            }
            ']' | '}' | ')' if !in_string => {
                current_item.push(ch);
                bracket_depth -= 1;
            }
            ',' if !in_string && bracket_depth == 0 => {
                if !current_item.trim().is_empty() {
                    items.push(current_item.trim().to_string());
                }
                current_item.clear();
            }
            _ => {
                current_item.push(ch);
            }
        }
    }

    if !current_item.trim().is_empty() {
        items.push(current_item.trim().to_string());
    }

    items
}

/// Post-process to convert simple multi-line blocks to single lines
fn post_process_single_line_blocks(input: &str) -> String {
    let mut result = String::new();
    let lines: Vec<&str> = input.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];

        // Look for pattern: line ending with { followed by content line and }
        if line.trim_end().ends_with('{') {
            // Check for empty block first: { followed by }
            if i + 1 < lines.len() {
                let next_line = lines[i + 1];

                // Handle empty blocks: "prefix {" + "};" -> "prefix {};"
                if next_line.trim() == "};" {
                    let prefix = line.trim_end().strip_suffix('{').unwrap_or("").trim();
                    // Preserve original indentation
                    let indent = get_line_indent(line);
                    result.push_str(&indent);
                    result.push_str(prefix);
                    result.push_str(" {};\n");
                    i += 2; // Skip the closing brace line
                    continue;
                }
            }

            // Check if we have a simple single-line block
            if i + 2 < lines.len() {
                let content_line = lines[i + 1];
                let end_line = lines[i + 2];

                let end_line_trimmed = end_line.trim();
                if is_simple_block_content(content_line)
                    && (end_line_trimmed == "};" || end_line_trimmed.starts_with("}; "))
                {
                    // Convert to single line: preserve indentation + "prefix { content };"
                    let prefix = line.trim_end().strip_suffix('{').unwrap_or("").trim();
                    let content = content_line.trim();
                    let indent = get_line_indent(line);
                    result.push_str(&indent);
                    result.push_str(prefix);
                    result.push_str(" { ");
                    result.push_str(content);
                    result.push(' ');
                    result.push_str(end_line_trimmed); // Include the full line with comments
                    result.push('\n');
                    i += 3; // Skip the content and closing brace lines
                    continue;
                } else if is_simple_block_content(content_line) && end_line.trim() == "}" {
                    // Convert to single line without semicolon: preserve indentation + "prefix { content }"
                    let prefix = line.trim_end().strip_suffix('{').unwrap_or("").trim();
                    let content = content_line.trim();
                    let indent = get_line_indent(line);
                    result.push_str(&indent);
                    result.push_str(prefix);
                    result.push_str(" { ");
                    result.push_str(content);
                    result.push_str(" }\n");
                    i += 3; // Skip the content and closing brace lines
                    continue;
                }
            }
        }

        // Not a simple block, keep the line as-is
        result.push_str(line);
        result.push('\n');
        i += 1;
    }

    result
}

/// Extract the indentation (leading whitespace) from a line
fn get_line_indent(line: &str) -> String {
    line.chars().take_while(|c| c.is_whitespace()).collect()
}

/// Check if a line contains simple content suitable for single-line blocks
fn is_simple_block_content(line: &str) -> bool {
    let content = line.trim();

    // Should not be empty
    if content.is_empty() {
        return false;
    }

    // Should be reasonably short
    if content.len() > 60 {
        return false;
    }

    // Should not contain multiple statements (no internal semicolons)
    if content.matches(';').count() > 1 {
        return false;
    }

    // Should not contain nested braces
    if content.contains('{') || content.contains('}') {
        return false;
    }

    true
}

/// Check if we're in a context where minus should be treated as a negative number
fn is_negative_number_context(output: &str) -> bool {
    // When processing a number after a minus, check if the minus was in a negative number context
    if output.ends_with('-') {
        // Get the context before the minus
        if let Some(before_minus) = output.strip_suffix('-') {
            // Check if the context before the minus indicates negative number
            let before_context_indicates_negative = before_minus.ends_with('{')
                || before_minus.ends_with('[')
                || before_minus.ends_with('(')
                || before_minus.ends_with("= ") || before_minus.ends_with('=')
                || before_minus.ends_with("== ") || before_minus.ends_with("==")
                || before_minus.ends_with("!= ") || before_minus.ends_with("!=")
                || before_minus.ends_with("< ") || before_minus.ends_with('<')
                || before_minus.ends_with("> ") || before_minus.ends_with('>')
                || before_minus.ends_with("<= ") || before_minus.ends_with("<=")
                || before_minus.ends_with(">= ") || before_minus.ends_with(">=")
                || before_minus.ends_with("&& ") || before_minus.ends_with("&&")
                || before_minus.ends_with("|| ") || before_minus.ends_with("||")
                || before_minus.ends_with(", ")
                || before_minus.ends_with("return ")
                || before_minus.ends_with("exitWith ")
                || before_minus.ends_with("+ ")
                || before_minus.ends_with("- ")  // "- -" case
                || before_minus.ends_with("* ")
                || before_minus.ends_with("/ ")
                || before_minus.ends_with("% ");

            if before_context_indicates_negative {
                return true;
            }
        }
    }

    // Check for arithmetic operators, but exclude "digit -" pattern
    if output.ends_with("+ ")
        || output.ends_with("* ")
        || output.ends_with("/ ")
        || output.ends_with("% ")
    {
        return true;
    }

    // Special handling for minus: allow "- -" and "--" but not "digit -"
    if output.ends_with("- ") {
        // Check if it's "digit -" pattern (should NOT be negative context)
        if output.len() >= 3 {
            let before_space_minus = output.chars().nth(output.len() - 3);
            if let Some(ch) = before_space_minus
                && ch.is_ascii_digit()
            {
                return false; // "digit -" is not negative context
            }
        }
        return true; // Other "- " patterns are negative context
    }

    // Special case: double minus (arithmetic minus followed by negative minus)
    if output.ends_with("--") {
        return true;
    }

    false
}

/// Post-process to fix negative numbers by removing spaces between - and digits
fn post_process_negative_numbers(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let ch = chars[i];
        result.push(ch);

        // Look for pattern: "- " followed by digits (potential negative number)
        if ch == '-' && i + 1 < chars.len() && chars[i + 1] == ' ' {
            // Check if next non-space character is a digit
            let mut j = i + 1;
            while j < chars.len() && chars[j] == ' ' {
                j += 1;
            }

            if j < chars.len() && chars[j].is_ascii_digit() {
                // Check if we're in a negative number context by looking backward
                if is_negative_context_backward(&result) {
                    // Skip the space(s) and continue - don't add the spaces to result
                    i = j - 1; // Will be incremented at end of loop
                }
            }
        }

        i += 1;
    }

    result
}

/// Check if the context before the minus suggests this is a negative number
fn is_negative_context_backward(text_before_minus: &str) -> bool {
    let trimmed = text_before_minus.trim_end_matches('-').trim_end();

    // If the context ends with an arithmetic operator, this is likely a binary operator, not a negative number
    if trimmed.ends_with('+')
        || trimmed.ends_with('-')
        || trimmed.ends_with('*')
        || trimmed.ends_with('/')
        || trimmed.ends_with('%')
    {
        return false;
    }

    // Common contexts where negative numbers appear
    trimmed.ends_with('=')
        || trimmed.ends_with('{')
        || trimmed.ends_with('[')
        || trimmed.ends_with('(')
        || trimmed.ends_with(',')
        || trimmed.ends_with("return")
        || trimmed.ends_with("exitWith")
}
