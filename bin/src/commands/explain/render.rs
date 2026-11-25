//! Render lint documentation, which is markdown, for the terminal

use std::sync::LazyLock;

use regex::Regex;

use super::highlight::{CommandLookup, Highlighter, Lang, color, hyperlink, wiki_url};

const HEADING: &str = "\x1b[1;4;94m";
const INLINE_CODE: &str = "\x1b[96m";
const LINK: &str = "\x1b[4;94m";
const INCORRECT: &str = "\x1b[1;31m";
const CORRECT: &str = "\x1b[1;32m";

static LINT_CODE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\bL-[CSL][0-9]{2}[A-Z]*\b").expect("lint code regex is valid"));

/// Find the book link for a lint code
pub type LintLookup<'a> = &'a dyn Fn(&str) -> Option<String>;

pub struct Renderer<'a> {
    /// Use OSC 8 hyperlinks, otherwise links are printed after their text
    pub links: bool,
    pub commands: CommandLookup<'a>,
    pub lints: LintLookup<'a>,
}

impl Renderer<'_> {
    #[must_use]
    pub fn markdown(&self, text: &str) -> String {
        let mut out = Vec::new();
        let mut code: Option<Highlighter> = None;
        for line in text.lines() {
            let trimmed = line.trim_start();
            if let Some(info) = trimmed.strip_prefix("```") {
                if code.take().is_some() {
                    out.push(format!("  {}╰─{}", color::DIM, color::RESET));
                } else {
                    let lang = Lang::from_fence(info);
                    let label = info.split([',', ' ']).next().unwrap_or_default();
                    out.push(format!("  {}╭─ {label}{}", color::DIM, color::RESET));
                    code = Some(Highlighter::new(lang, self.commands, self.links));
                }
                continue;
            }
            if let Some(highlighter) = &mut code {
                out.push(format!(
                    "  {}│{} {}",
                    color::DIM,
                    color::RESET,
                    highlighter.line(line)
                ));
                continue;
            }
            let hashes = trimmed.chars().take_while(|c| *c == '#').count();
            if hashes > 0 && trimmed[hashes..].starts_with(' ') {
                out.push(format!(
                    "{HEADING}{}{}",
                    trimmed[hashes..].trim(),
                    color::RESET
                ));
                continue;
            }
            if let Some(item) = trimmed
                .strip_prefix("- ")
                .or_else(|| trimmed.strip_prefix("* "))
            {
                let indent = &line[..line.len() - trimmed.len()];
                out.push(format!("{indent}  • {}", self.inline(item)));
                continue;
            }
            out.push(self.inline(line));
        }
        out.join("\n")
    }

    /// Bold, inline code, links and lint codes within a line
    #[must_use]
    pub fn inline(&self, text: &str) -> String {
        let mut out = String::new();
        let mut plain = String::new();
        let mut rest = text;
        while let Some(c) = rest.chars().next() {
            if c == '`'
                && let Some(end) = rest[1..].find('`')
            {
                out.push_str(&self.lint_codes(&std::mem::take(&mut plain)));
                out.push_str(&self.inline_code(&rest[1..=end]));
                rest = &rest[end + 2..];
                continue;
            }
            if let Some(after) = rest.strip_prefix("**")
                && let Some(end) = after.find("**")
            {
                out.push_str(&self.lint_codes(&std::mem::take(&mut plain)));
                out.push_str(&bold(&after[..end]));
                rest = &after[end + 2..];
                continue;
            }
            if c == '['
                && let Some((label, url, len)) = markdown_link(rest)
            {
                out.push_str(&self.lint_codes(&std::mem::take(&mut plain)));
                out.push_str(&self.link(label, &resolve(url)));
                rest = &rest[len..];
                continue;
            }
            plain.push(c);
            rest = &rest[c.len_utf8()..];
        }
        out.push_str(&self.lint_codes(&plain));
        out
    }

    fn inline_code(&self, code: &str) -> String {
        let painted = format!("{INLINE_CODE}{code}{}", color::RESET);
        if let Some(url) = (self.lints)(code) {
            return self.link_painted(&painted, &url);
        }
        let single_word = !code.is_empty()
            && code.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
            && !code.starts_with('_');
        if single_word && let Some(name) = (self.commands)(code) {
            return self.link_painted(&painted, &wiki_url(&name));
        }
        painted
    }

    /// Link lint codes, such as L-S14, in plain text
    fn lint_codes(&self, text: &str) -> String {
        if !self.links {
            return text.to_string();
        }
        LINT_CODE
            .replace_all(text, |caps: &regex::Captures| {
                let code = &caps[0];
                (self.lints)(code).map_or_else(|| code.to_string(), |url| hyperlink(code, &url))
            })
            .into_owned()
    }

    fn link(&self, label: &str, url: &str) -> String {
        let painted = format!("{LINK}{label}{}", color::RESET);
        if self.links {
            hyperlink(&painted, url)
        } else {
            format!("{painted} {}({url}){}", color::DIM, color::RESET)
        }
    }

    /// Link text that is already styled, only when hyperlinks are supported
    fn link_painted(&self, painted: &str, url: &str) -> String {
        if self.links {
            hyperlink(painted, url)
        } else {
            painted.to_string()
        }
    }
}

fn bold(text: &str) -> String {
    match text {
        "Incorrect" => format!("{INCORRECT}✗ Incorrect{}", color::RESET),
        "Correct" => format!("{CORRECT}✓ Correct{}", color::RESET),
        _ => format!("{}{text}{}", color::BOLD, color::RESET),
    }
}

/// Parse `[label](url)` at the start of `text`, returning the label, url and length
fn markdown_link(text: &str) -> Option<(&str, &str, usize)> {
    let close = text.find("](")?;
    let label = &text[1..close];
    if label.contains('[') {
        return None;
    }
    let url_start = close + 2;
    let url_len = text[url_start..].find(')')?;
    let url = &text[url_start..url_start + url_len];
    Some((label, url, url_start + url_len + 1))
}

/// Book links in lint docs are relative to the lints folder
fn resolve(url: &str) -> String {
    if url.starts_with("http://") || url.starts_with("https://") {
        return url.to_string();
    }
    let path = url.replace(".md", ".html");
    if let Some(path) = path.strip_prefix('/') {
        return format!("https://hemtt.dev/{path}");
    }
    let mut base = vec!["lints"];
    for part in path.split('/') {
        match part {
            ".." => {
                base.pop();
            }
            "." | "" => {}
            part => base.push(part),
        }
    }
    format!("https://hemtt.dev/{}", base.join("/"))
}

#[cfg(test)]
mod tests {
    use super::super::highlight::tests::strip;
    use super::*;

    fn commands(word: &str) -> Option<String> {
        (word.eq_ignore_ascii_case("isEqualTypeAny")).then(|| "isEqualTypeAny".to_string())
    }

    fn lints(code: &str) -> Option<String> {
        (code == "L-S14").then(|| "https://hemtt.dev/lints/sqf.html#unused".to_string())
    }

    fn renderer(links: bool) -> Renderer<'static> {
        Renderer {
            links,
            commands: &commands,
            lints: &lints,
        }
    }

    #[test]
    fn markdown() {
        let doc = "### Example\n\n**Incorrect**\n```sqf\n_x isEqualTypeAny [0];\n```\n- **one**: `isEqualTypeAny` and L-S14";
        let out = strip(&renderer(true).markdown(doc));
        assert_eq!(
            out,
            "Example\n\n✗ Incorrect\n  ╭─ sqf\n  │ _x isEqualTypeAny [0];\n  ╰─\n  • one: isEqualTypeAny and L-S14"
        );
    }

    #[test]
    fn links() {
        let r = renderer(true);
        let out = r.inline("see `isEqualTypeAny`, L-S14 and [the wiki](https://example.com)");
        assert!(out.contains("https://community.bistudio.com/wiki/isEqualTypeAny"));
        assert!(out.contains("https://hemtt.dev/lints/sqf.html#unused"));
        assert!(out.contains("https://example.com"));

        // without hyperlinks the url is printed, and only for markdown links
        let out =
            strip(&renderer(false).inline("`isEqualTypeAny` [the wiki](https://example.com)"));
        assert_eq!(out, "isEqualTypeAny the wiki (https://example.com)");
    }

    #[test]
    fn resolve_links() {
        assert_eq!(resolve("https://a.b/c"), "https://a.b/c");
        assert_eq!(
            resolve("../configuration/lints.md#sqf"),
            "https://hemtt.dev/configuration/lints.html#sqf"
        );
        assert_eq!(
            resolve("/commands/check.md"),
            "https://hemtt.dev/commands/check.html"
        );
        assert_eq!(resolve("sqf.md"), "https://hemtt.dev/lints/sqf.html");
    }

    #[test]
    fn not_a_link() {
        let out = strip(&renderer(true).inline("array[0] and [1](2"));
        assert_eq!(out, "array[0] and [1](2");
    }
}
