//! Terminal syntax highlighting for the code blocks in lint documentation
//!
//! The highlighters only add escape sequences, they never change the text:
//! with the escapes removed the output is the input.

pub mod color {
    pub const RESET: &str = "\x1b[0m";
    pub const BOLD: &str = "\x1b[1m";
    pub const DIM: &str = "\x1b[2m";
    pub const ITALIC: &str = "\x1b[3m";

    pub const KEYWORD: &str = "\x1b[35m";
    pub const COMMAND: &str = "\x1b[94m";
    pub const LOCAL: &str = "\x1b[36m";
    pub const MACRO: &str = "\x1b[93m";
    pub const STRING: &str = "\x1b[32m";
    pub const NUMBER: &str = "\x1b[33m";
    pub const CONSTANT: &str = "\x1b[33m";
    pub const COMMENT: &str = "\x1b[3;90m";
    pub const DIRECTIVE: &str = "\x1b[95m";
    pub const CLASS: &str = "\x1b[1;96m";
    pub const PROPERTY: &str = "\x1b[34m";
    pub const SECTION: &str = "\x1b[1;95m";
}

const WIKI: &str = "https://community.bistudio.com/wiki";

/// SQF words drawn as keywords, the control flow of a script
const SQF_KEYWORDS: &[&str] = &[
    "if",
    "then",
    "else",
    "exitwith",
    "while",
    "do",
    "for",
    "from",
    "to",
    "step",
    "foreach",
    "foreachreversed",
    "switch",
    "case",
    "default",
    "try",
    "catch",
    "throw",
    "waituntil",
    "private",
    "params",
    "call",
    "spawn",
    "breakout",
    "breakwith",
    "continue",
    "continuewith",
];

const CONFIG_KEYWORDS: &[&str] = &["class", "delete", "import", "enum", "__eval", "__exec"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lang {
    Sqf,
    Config,
    Toml,
    Plain,
}

impl Lang {
    #[must_use]
    /// The language of a fenced code block, from its info string
    pub fn from_fence(info: &str) -> Self {
        // the book uses extra attributes, such as ```toml,fp=...
        match info.split([',', ' ']).next().unwrap_or_default() {
            "sqf" => Self::Sqf,
            "hpp" | "cpp" | "h" | "ext" | "inc" => Self::Config,
            "toml" => Self::Toml,
            _ => Self::Plain,
        }
    }
}

/// Look up an SQF command, returning its name as the wiki spells it
pub type CommandLookup<'a> = &'a dyn Fn(&str) -> Option<String>;

pub struct Highlighter<'a> {
    lang: Lang,
    commands: CommandLookup<'a>,
    links: bool,
    in_block_comment: bool,
}

impl<'a> Highlighter<'a> {
    #[must_use]
    pub fn new(lang: Lang, commands: CommandLookup<'a>, links: bool) -> Self {
        Self {
            lang,
            commands,
            links,
            in_block_comment: false,
        }
    }

    /// Highlight one line, block comments carry over to the next line
    pub fn line(&mut self, line: &str) -> String {
        if self.lang == Lang::Plain {
            return line.to_string();
        }
        let chars = line.chars().collect::<Vec<_>>();
        let mut out = String::with_capacity(line.len() * 2);
        let mut i = 0;
        // config: the next identifier is a class name
        let mut expect_class = false;

        if self.lang == Lang::Toml && line.trim_start().starts_with('[') {
            paint(&mut out, color::SECTION, line);
            return out;
        }

        while i < chars.len() {
            let c = chars[i];
            let next = chars.get(i + 1).copied();

            if self.in_block_comment {
                i = self.block_comment(&mut out, &chars, i, i);
                continue;
            }

            let line_comment = match self.lang {
                Lang::Toml => c == '#',
                _ => c == '/' && next == Some('/'),
            };
            if line_comment {
                paint(&mut out, color::COMMENT, &collect(&chars[i..]));
                break;
            }
            if self.lang != Lang::Toml && c == '/' && next == Some('*') {
                i = self.block_comment(&mut out, &chars, i, i + 2);
                continue;
            }

            if c == '"' || c == '\'' {
                let end = self.string_end(&chars, i);
                paint(&mut out, color::STRING, &collect(&chars[i..end]));
                i = end;
                continue;
            }

            if self.lang != Lang::Toml && c == '#' && chars[..i].iter().all(|c| c.is_whitespace()) {
                let end = scan(&chars, i + 1, |c| c.is_ascii_alphanumeric() || c == '_');
                paint(&mut out, color::DIRECTIVE, &collect(&chars[i..end]));
                i = end;
                continue;
            }

            let starts_number = c.is_ascii_digit()
                || (c == '.' && next.is_some_and(|n| n.is_ascii_digit()))
                || (c == '$' && next.is_some_and(|n| n.is_ascii_hexdigit()));
            let after_word = i > 0 && is_word(chars[i - 1]);
            if starts_number && !after_word {
                let end = scan(&chars, i + 1, |c| c.is_ascii_alphanumeric() || c == '.');
                paint(&mut out, color::NUMBER, &collect(&chars[i..end]));
                i = end;
                continue;
            }

            if c.is_ascii_alphabetic() || c == '_' {
                let toml = self.lang == Lang::Toml;
                let end = scan(&chars, i + 1, |c| is_word(c) || (toml && c == '-'));
                let word = collect(&chars[i..end]);
                let following = chars[end..].iter().find(|c| !c.is_whitespace()).copied();
                match self.lang {
                    Lang::Sqf => self.sqf_word(&mut out, &word),
                    Lang::Config => {
                        expect_class = config_word(&mut out, &word, following, expect_class);
                    }
                    Lang::Toml => toml_word(&mut out, &word, following),
                    Lang::Plain => out.push_str(&word),
                }
                i = end;
                continue;
            }

            if self.lang == Lang::Config && c == ':' {
                expect_class = true;
            }
            out.push(c);
            i += 1;
        }
        out
    }

    /// Paint a block comment from `start`, looking for its end from `search`
    ///
    /// Returns the index after the comment, or the end of the line if it continues
    fn block_comment(
        &mut self,
        out: &mut String,
        chars: &[char],
        start: usize,
        search: usize,
    ) -> usize {
        let end = find(chars, search, "*/").map(|e| e + 2);
        self.in_block_comment = end.is_none();
        let end = end.unwrap_or(chars.len());
        paint(out, color::COMMENT, &collect(&chars[start..end]));
        end
    }

    /// The index after the closing quote of the string starting at `start`
    fn string_end(&self, chars: &[char], start: usize) -> usize {
        let quote = chars[start];
        let mut i = start + 1;
        while i < chars.len() {
            if self.lang == Lang::Toml && quote == '"' && chars[i] == '\\' {
                i += 2;
                continue;
            }
            if chars[i] == quote {
                // SQF and config escape a quote by doubling it
                if self.lang != Lang::Toml && chars.get(i + 1) == Some(&quote) {
                    i += 2;
                    continue;
                }
                return i + 1;
            }
            i += 1;
        }
        chars.len()
    }

    fn sqf_word(&self, out: &mut String, word: &str) {
        if word.starts_with('_') {
            paint(out, color::LOCAL, word);
            return;
        }
        let lower = word.to_ascii_lowercase();
        let style = if SQF_KEYWORDS.contains(&lower.as_str()) {
            Some(color::KEYWORD)
        } else if ["true", "false", "nil"].contains(&lower.as_str()) {
            Some(color::CONSTANT)
        } else if is_macro(word) {
            Some(color::MACRO)
        } else {
            None
        };
        let command = (self.commands)(word);
        let style = style.or_else(|| command.as_ref().map(|_| color::COMMAND));
        let mut painted = String::new();
        match style {
            Some(style) => paint(&mut painted, style, word),
            None => painted.push_str(word),
        }
        match command {
            Some(name) if self.links && !is_macro(word) => {
                out.push_str(&hyperlink(&painted, &wiki_url(&name)));
            }
            _ => out.push_str(&painted),
        }
    }
}

/// Returns whether the next identifier is a class name
fn config_word(out: &mut String, word: &str, following: Option<char>, expect_class: bool) -> bool {
    let lower = word.to_ascii_lowercase();
    if CONFIG_KEYWORDS.contains(&lower.as_str()) {
        paint(out, color::KEYWORD, word);
        return matches!(lower.as_str(), "class" | "delete" | "import");
    }
    if expect_class {
        paint(out, color::CLASS, word);
    } else if is_macro(word) {
        paint(out, color::MACRO, word);
    } else if ["true", "false"].contains(&lower.as_str()) {
        paint(out, color::CONSTANT, word);
    } else if matches!(following, Some('=' | '[')) {
        paint(out, color::PROPERTY, word);
    } else {
        out.push_str(word);
    }
    false
}

fn toml_word(out: &mut String, word: &str, following: Option<char>) {
    if matches!(following, Some('=')) {
        paint(out, color::PROPERTY, word);
    } else if ["true", "false"].contains(&word) {
        paint(out, color::CONSTANT, word);
    } else {
        out.push_str(word);
    }
}

#[must_use]
pub fn wiki_url(command: &str) -> String {
    format!("{WIKI}/{command}")
}

#[must_use]
/// Wrap text in an OSC 8 hyperlink
pub fn hyperlink(text: &str, url: &str) -> String {
    terminal_link::Link::new(text, url).to_string()
}

/// A macro by convention: all upper case, such as `QGVAR` or `ARR_2`
fn is_macro(word: &str) -> bool {
    word.len() > 1
        && word.chars().any(|c| c.is_ascii_uppercase())
        && word
            .chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_')
}

fn paint(out: &mut String, style: &str, text: &str) {
    out.push_str(style);
    out.push_str(text);
    out.push_str(color::RESET);
}

const fn is_word(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

fn scan(chars: &[char], from: usize, keep: impl Fn(char) -> bool) -> usize {
    let mut i = from;
    while i < chars.len() && keep(chars[i]) {
        i += 1;
    }
    i
}

fn find(chars: &[char], from: usize, needle: &str) -> Option<usize> {
    let needle = needle.chars().collect::<Vec<_>>();
    (from..chars.len()).find(|&i| chars[i..].starts_with(&needle))
}

fn collect(chars: &[char]) -> String {
    chars.iter().collect()
}

#[cfg(test)]
pub mod tests {
    use super::*;

    /// Remove SGR colours and OSC 8 hyperlinks
    pub fn strip(text: &str) -> String {
        let mut out = String::new();
        let mut chars = text.chars().peekable();
        while let Some(c) = chars.next() {
            if c != '\x1b' {
                out.push(c);
                continue;
            }
            match chars.next() {
                Some('[') => {
                    for c in chars.by_ref() {
                        if c.is_ascii_alphabetic() {
                            break;
                        }
                    }
                }
                Some(']') => {
                    // OSC, ends with ESC \ or BEL
                    while let Some(c) = chars.next() {
                        if c == '\x07' {
                            break;
                        }
                        if c == '\x1b' && chars.peek() == Some(&'\\') {
                            chars.next();
                            break;
                        }
                    }
                }
                _ => {}
            }
        }
        out
    }

    fn commands(word: &str) -> Option<String> {
        ["isEqualTypeAny", "isEqualType", "hint", "count"]
            .iter()
            .find(|c| c.eq_ignore_ascii_case(word))
            .map(ToString::to_string)
    }

    fn sqf(line: &str, links: bool) -> String {
        Highlighter::new(Lang::Sqf, &commands, links).line(line)
    }

    #[test]
    fn text_is_unchanged() {
        let lines = [
            r#"if (_x isEqualTypeAny [0, ""]) then { hint "a ""quoted"" word"; };"#,
            "private _value = QGVAR(enabled); // a comment",
            "_a = 0x1F + 1.5e3 + .5 + $FF; /* block */ _b = 1;",
            "#define ARR_2(a,b) a,b",
            "unterminated \"string",
            "/* starts a block",
        ];
        for links in [false, true] {
            for line in lines {
                assert_eq!(strip(&sqf(line, links)), line);
            }
        }
        let mut h = Highlighter::new(Lang::Config, &commands, true);
        for line in [
            "class CfgPatches: Parent { units[] = {}; scope = 2; };",
            "    displayName = \"$STR_test\"; // name",
            "#include \"script_component.hpp\"",
        ] {
            assert_eq!(strip(&h.line(line)), line);
        }
        let mut h = Highlighter::new(Lang::Toml, &commands, true);
        for line in [
            "[lints.sqf.s47]",
            "enabled = true # on",
            "options.x = \"a\\\"b\"",
        ] {
            assert_eq!(strip(&h.line(line)), line);
        }
    }

    #[test]
    fn sqf_tokens() {
        let out = sqf("_x isEqualTypeAny QGVAR(enabled)", false);
        assert!(out.contains(&format!("{}_x{}", color::LOCAL, color::RESET)));
        assert!(out.contains(&format!("{}isEqualTypeAny{}", color::COMMAND, color::RESET)));
        assert!(out.contains(&format!("{}QGVAR{}", color::MACRO, color::RESET)));
        // a global variable that isn't a command is left alone
        assert_eq!(sqf("myVariable", false), "myVariable");
    }

    #[test]
    fn sqf_links() {
        let out = sqf("hint _x", true);
        assert!(out.contains("https://community.bistudio.com/wiki/hint"));
        // macros and locals are never linked
        assert!(!sqf("_hint COUNT", true).contains("\x1b]8"));
    }

    #[test]
    fn block_comments_span_lines() {
        let mut h = Highlighter::new(Lang::Sqf, &commands, false);
        let _ = h.line("/* start");
        let middle = h.line("hint _x");
        assert_eq!(middle, format!("{}hint _x{}", color::COMMENT, color::RESET));
        let end = h.line("end */ hint");
        assert!(end.ends_with(&format!("{}hint{}", color::COMMAND, color::RESET)));
    }

    #[test]
    fn config_classes() {
        let mut h = Highlighter::new(Lang::Config, &commands, false);
        let out = h.line("class Child: Parent { scope = 2; };");
        assert!(out.contains(&format!("{}Child{}", color::CLASS, color::RESET)));
        assert!(out.contains(&format!("{}Parent{}", color::CLASS, color::RESET)));
        assert!(out.contains(&format!("{}scope{}", color::PROPERTY, color::RESET)));
    }

    #[test]
    fn fence() {
        assert_eq!(Lang::from_fence("sqf"), Lang::Sqf);
        assert_eq!(Lang::from_fence("hpp"), Lang::Config);
        assert_eq!(Lang::from_fence("toml,fp=.hemtt/project.toml"), Lang::Toml);
        assert_eq!(Lang::from_fence(""), Lang::Plain);
    }
}
