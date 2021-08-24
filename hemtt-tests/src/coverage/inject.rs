#[must_use]
pub fn inject(source: &str, key: &str) -> (String, Vec<i32>) {
    let mut quote = false;
    let mut line = 1;
    let mut lines = Vec::new();
    let mut result = format!(r#""hemtt_tests" callExtension ["cov", ["{}", 0]];"#, key);
    for char in source.chars() {
        result.push(char);
        if char == '"' {
            quote = !quote;
        }
        if !quote {
            if char == '\n' {
                line += 1;
            }
            if char == ';' || char == '{' {
                lines.push(line);
                result.push_str(&format!(
                    "\n\"hemtt_tests\" callExtension [\"cov\", [\"{}\", {}]];",
                    key, line
                ));
            }
        }
    }
    (result, lines)
}
