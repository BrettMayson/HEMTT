use std::sync::Arc;

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
        println!("{}", explanation.render());
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

    let lints = all();
    let haystack = lints
        .iter()
        .flat_map(|e| [e.code.as_str(), e.ident])
        .collect::<Vec<_>>();
    let similar = hemtt_common::similar_values(cmd.code.trim(), &haystack)
        .into_iter()
        .map(ToOwned::to_owned)
        .collect();
    let mut report = Report::new();
    report.push(Arc::new(UnknownLint {
        query: cmd.code.trim().to_string(),
        similar,
    }));
    Ok(report)
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
