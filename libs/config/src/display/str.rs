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
            } else {
                output.push(c);
            }
            last = c;
        }
        write!(f, "\"{output}\"")
    }
}
