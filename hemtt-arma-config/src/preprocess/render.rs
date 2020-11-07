use super::Token;

pub fn render(source: Vec<Token>) -> String {
    let mut out = String::new();
    for token in source {
        out.push_str(&token.to_string())
    }
    out
}
