use super::TokenPos;

pub fn render(source: Vec<TokenPos>) -> String {
    let mut out = String::new();
    for token in source {
        out.push_str(&token.to_string())
    }
    out
}
