mod highlight;
mod render;

use std::{io::IsTerminal, sync::Arc};

use hemtt_common::config::LintEnabled;
use hemtt_config::analyze::CONFIG_LINTS;
use hemtt_sqf::analyze::{
    LintData as SqfLintData, SQF_LINTS,
    lints::s02_event_handlers::{
        LintS02EventIncorrectCommand, LintS02EventInsufficientVersion, LintS02EventUnknown,
    },
};
use hemtt_stringtable::analyze::STRINGTABLE_LINTS;
use hemtt_workspace::{
    lint::{Lint, Lints},
    reporting::{Code, Diagnostic},
};

use crate::{Error, report::Report};

use self::{
    highlight::{color, hyperlink},
    render::Renderer,
};

const BOOK: &str = "https://hemtt.dev";

#[derive(clap::Parser)]
/// Explain a lint code
///
/// Prints the documentation for a lint, the same text found in the
/// [lints section of the book](../lints/index.md), and a link to its page.
///
/// The code can be given as it is shown in a diagnostic, without the `L-`,
/// or as the lint's name, case insensitive.
///
/// ```bash
/// hemtt explain L-S47
/// hemtt explain s47
/// hemtt explain is_equal_type_any
/// ```
pub struct Command {
    /// The code or name of the lint
    code: String,
}

#[derive(Debug, Clone)]
pub struct Explanation {
    pub code: String,
    pub ident: &'static str,
    pub description: &'static str,
    pub documentation: &'static str,
    pub severity: String,
    pub enabled: LintEnabled,
    pub minimum_severity: String,
    page: &'static str,
}

impl Explanation {
    fn new<D>(lint: &Arc<Box<dyn Lint<D>>>, prefix: &str, page: &'static str) -> Self {
        let config = lint.default_config();
        Self {
            code: format!("{prefix}{}", lint.doc_ident()),
            ident: lint.ident(),
            description: lint.description(),
            documentation: lint.documentation(),
            severity: format!("{:?}", config.severity()),
            enabled: config.enabled(),
            minimum_severity: format!("{:?}", lint.minimum_severity()),
            page,
        }
    }

    #[must_use]
    /// The full link to the lint in the book
    pub fn url(&self) -> String {
        // mdbook builds heading anchors from the heading text, which is the ident
        format!("{BOOK}/lints/{}.html#{}", self.page, self.ident)
    }

    fn matches(&self, query: &str) -> bool {
        let query = query.trim();
        if self.ident.eq_ignore_ascii_case(query) {
            return true;
        }
        let code = self.code.trim_start_matches("L-");
        let query = query
            .strip_prefix("L-")
            .or_else(|| query.strip_prefix("l-"))
            .unwrap_or(query);
        code.eq_ignore_ascii_case(query)
    }

    fn render(&self) -> String {
        let state = match self.enabled {
            LintEnabled::Enabled => "",
            LintEnabled::Disabled => " (disabled by default)",
            LintEnabled::Pedantic => " (pedantic)",
        };
        format!(
            "{code}: {ident}\n\n{description}\n\nDefault severity: {severity}{state}\nMinimum severity: {minimum}\n\n{documentation}\n\nRead more: {url}",
            code = self.code,
            ident = self.ident,
            description = self.description,
            severity = self.severity,
            minimum = self.minimum_severity,
            documentation = self.documentation.trim(),
            url = self.url(),
        )
    }

    /// Colours, highlighted code blocks and, if `renderer.links`, hyperlinks
    fn render_fancy(&self, renderer: &Renderer) -> String {
        let badge = match self.severity.as_str() {
            "Error" => "\x1b[1;97;41m",
            "Warning" => "\x1b[1;30;43m",
            "Help" => "\x1b[1;30;46m",
            _ => "\x1b[1;30;47m",
        };
        let severity = match self.severity.as_str() {
            "Error" => "\x1b[1;31m",
            "Warning" => "\x1b[1;33m",
            "Help" => "\x1b[1;36m",
            _ => color::BOLD,
        };
        let state = match self.enabled {
            LintEnabled::Enabled => String::new(),
            LintEnabled::Disabled => {
                format!(" {}(disabled by default){}", color::DIM, color::RESET)
            }
            LintEnabled::Pedantic => format!(" {}(pedantic){}", color::DIM, color::RESET),
        };
        let url = self.url();
        let ident = format!("{}{}{}", color::BOLD, self.ident, color::RESET);
        let ident = if renderer.links {
            hyperlink(&ident, &url)
        } else {
            ident
        };
        let read_more = if renderer.links {
            hyperlink(&format!("\x1b[4;94m{url}{}", color::RESET), &url)
        } else {
            format!("\x1b[4;94m{url}{}", color::RESET)
        };
        format!(
            "{badge} {code} {reset} {ident}\n{italic}{description}{reset}\n\n{dim}Severity{reset} {severity}{default}{reset}{state}  {dim}Minimum{reset} {minimum}\n\n{documentation}\n\n{dim}Read more:{reset} {read_more}",
            code = self.code,
            reset = color::RESET,
            italic = color::ITALIC,
            dim = color::DIM,
            // inline code resets the style, so italics are turned back on after it
            description = renderer
                .inline(self.description)
                .replace(color::RESET, &format!("{}{}", color::RESET, color::ITALIC)),
            default = self.severity,
            minimum = self.minimum_severity,
            documentation = renderer.markdown(self.documentation.trim()),
        )
    }
}

#[must_use]
/// Every documented lint, in the same order as the book
pub fn all() -> Vec<Explanation> {
    let mut out = Vec::new();

    let mut config = CONFIG_LINTS
        .iter()
        .filter(|l| l.display())
        .map(|l| (l.sort(), Explanation::new(l, "L-C", "config")))
        .collect::<Vec<_>>();
    config.sort_by_key(|(sort, _)| *sort);
    out.extend(config.into_iter().map(|(_, e)| e));

    // The S02 sub-lints are not registered on their own, the book adds them the same way
    let s02: Lints<SqfLintData> = vec![
        Arc::new(Box::new(LintS02EventUnknown)),
        Arc::new(Box::new(LintS02EventIncorrectCommand)),
        Arc::new(Box::new(LintS02EventInsufficientVersion)),
    ];
    let mut sqf = SQF_LINTS
        .iter()
        .filter(|l| l.display())
        .map(|l| (**l).clone())
        .chain(s02)
        .map(|l| (l.sort(), Explanation::new(&l, "L-S", "sqf")))
        .collect::<Vec<_>>();
    sqf.sort_by_key(|(sort, _)| *sort);
    out.extend(sqf.into_iter().map(|(_, e)| e));

    let mut stringtables = STRINGTABLE_LINTS
        .iter()
        .filter(|l| l.display())
        .map(|l| (l.sort(), Explanation::new(l, "L-L", "stringtables")))
        .collect::<Vec<_>>();
    stringtables.sort_by_key(|(sort, _)| *sort);
    out.extend(stringtables.into_iter().map(|(_, e)| e));

    out
}

#[must_use]
/// Find the lint for a code or name
pub fn find(query: &str) -> Option<Explanation> {
    all().into_iter().find(|e| e.matches(query))
}

/// Execute the explain command
///
/// # Errors
/// Will not return an error
pub fn execute(cmd: &Command) -> Result<Report, Error> {
    if let Some(explanation) = find(&cmd.code) {
        let fancy = hemtt_common::ansi::ansi_supported() && std::io::stdout().is_terminal();
        if fancy {
            println!("{}", fancy_render(&explanation));
        } else {
            println!("{}", explanation.render());
        }
        return Ok(Report::new());
    }

    let upper = cmd.code.trim().to_uppercase();
    if upper.starts_with("PE") || upper.starts_with("PW") {
        println!(
            "{} is a preprocessor code, they are listed at {BOOK}/lints/preprocessor.html",
            cmd.code.trim()
        );
        return Ok(Report::new());
    }

    let mut report = Report::new();
    report.push(unknown_lint(&cmd.code));
    Ok(report)
}

#[must_use]
/// An error for a query that matches no lint, suggesting similar lints
pub fn unknown_lint(query: &str) -> Arc<dyn Code> {
    let lints = all();
    let haystack = lints
        .iter()
        .flat_map(|e| [e.code.as_str(), e.ident])
        .collect::<Vec<_>>();
    let similar = hemtt_common::similar_values(query.trim(), &haystack)
        .into_iter()
        .map(ToOwned::to_owned)
        .collect();
    Arc::new(UnknownLint {
        query: query.trim().to_string(),
        similar,
    })
}

fn fancy_render(explanation: &Explanation) -> String {
    let links = supports_hyperlinks::on(supports_hyperlinks::Stream::Stdout);
    // commands are only coloured in SQF, the wiki is only loaded when needed
    let wiki = (explanation.code.starts_with("L-S")
        || explanation.documentation.contains("```sqf"))
    .then(|| hemtt_sqf::parser::database::Database::empty(false));
    let commands = |word: &str| -> Option<String> {
        let wiki = wiki.as_ref()?.wiki();
        wiki.commands().get(word).map(|c| c.name().to_string())
    };
    let lint_url = |code: &str| find(code).filter(|e| e.code == code).map(|e| e.url());
    explanation.render_fancy(&Renderer {
        links,
        commands: &commands,
        lints: &lint_url,
    })
}

struct UnknownLint {
    query: String,
    similar: Vec<String>,
}

impl Code for UnknownLint {
    fn ident(&self) -> &'static str {
        // binary, command, explain, error 1
        "BCEE1"
    }

    fn message(&self) -> String {
        format!("No lint found for `{}`", self.query)
    }

    fn help(&self) -> Option<String> {
        if self.similar.is_empty() {
            Some(format!("All lints are listed at {BOOK}/lints/index.html"))
        } else {
            Some(format!(
                "Did you mean {}?",
                self.similar
                    .iter()
                    .map(|s| format!("`{s}`"))
                    .collect::<Vec<_>>()
                    .join(", ")
            ))
        }
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        Some(Diagnostic::from_code(self))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn codes_are_unique() {
        let mut seen = HashSet::new();
        for e in all() {
            assert!(
                seen.insert(e.code.clone()),
                "duplicate lint code {}",
                e.code
            );
        }
    }

    #[test]
    fn idents_are_unique() {
        let mut seen = HashSet::new();
        for e in all() {
            assert!(seen.insert(e.ident), "duplicate lint ident {}", e.ident);
        }
    }

    #[test]
    fn every_lint_can_be_found() {
        for e in all() {
            for query in [
                e.code.clone(),
                e.code.to_lowercase(),
                e.code.trim_start_matches("L-").to_string(),
                e.ident.to_string(),
                e.ident.to_uppercase(),
            ] {
                let found = find(&query).unwrap_or_else(|| panic!("{query} not found"));
                assert_eq!(found.code, e.code, "{query} found the wrong lint");
            }
        }
    }

    #[test]
    fn known_lint() {
        let e = find("L-S47").expect("S47 exists");
        assert_eq!(e.ident, "is_equal_type_any");
        assert_eq!(
            e.url(),
            "https://hemtt.dev/lints/sqf.html#is_equal_type_any"
        );
        assert!(e.render().contains("isEqualTypeAny"));
    }

    #[test]
    fn fancy_keeps_the_text() {
        let commands = |_: &str| None;
        let lints = |code: &str| find(code).map(|e| e.url());
        let renderer = Renderer {
            links: true,
            commands: &commands,
            lints: &lints,
        };
        for e in all() {
            let out = highlight::tests::strip(&e.render_fancy(&renderer));
            assert!(out.contains(&e.code), "{}", e.code);
            assert!(out.contains(&e.url()), "{}", e.code);
            assert!(!out.contains('\x1b'), "{} leaves an escape behind", e.code);
            // every line of every code block survives highlighting
            let mut in_code = false;
            for line in e.documentation.lines() {
                if line.trim_start().starts_with("```") {
                    in_code = !in_code;
                } else if in_code {
                    assert!(out.contains(line), "{}: lost `{line}`", e.code);
                }
            }
        }
    }

    #[test]
    fn sub_lints_are_included() {
        assert!(find("L-S02IV").is_some());
        assert!(find("L-S02UE").is_some());
        assert!(find("L-S02IC").is_some());
    }

    #[test]
    fn unknown() {
        assert!(find("L-S9999").is_none());
        assert!(find("").is_none());
    }
}
