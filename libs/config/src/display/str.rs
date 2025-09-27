use crate::Str;

impl std::fmt::Display for Str {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // replace \n with `" \n "`, but not \\\n
        let mut last = '\0';
        let mut output = String::new();
        for c in self.value.chars() {
            if c == '\\' && last == '\\' {
                output.push(c);
            } else if c == '\n' {
                output.push_str("\" \\n \"");
            } else if c == '"' {
                output.push_str("\"\"");
            } else {
                output.push(c);
            }
            last = c;
        }
        write!(f, "\"{output}\"")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_newline() {
        let s = Str::test_new("Hello\nWorld");
        assert_eq!(s.to_string(), "\"Hello\" \\n \"World\"");
    }

    #[test]
    fn test_backslash_newline() {
        let s = Str::test_new("Hello\\\nWorld");
        assert_eq!(s.to_string(), "\"Hello\\\" \\n \"World\"");
    }

    #[test]
    fn test_quotes() {
        let s = Str::test_new("value = \"hello\"");
        assert_eq!(s.to_string(), "\"value = \"\"hello\"\"\"");
    }
}
