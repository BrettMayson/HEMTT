use std::sync::Arc;

use hemtt_common::config::LintEnabled;
use hemtt_config::analyze::CONFIG_LINTS;
use hemtt_sqf::analyze::{
    LintData, SQF_LINTS,
    lints::s02_event_handlers::{
        LintS02EventIncorrectCommand, LintS02EventInsufficientVersion, LintS02EventUnknown,
    },
};
use hemtt_stringtable::analyze::STRINGTABLE_LINTS;
use hemtt_workspace::lint::{Lint, Lints};
use mdbook::book::Chapter;

pub fn run(chapter: &mut Chapter) {
    for item in &mut chapter.sub_items {
        if let mdbook::BookItem::Chapter(chapter) = item {
            eprintln!("Processing chapter: {}", chapter.name);
            if chapter.name == "Config" {
                config(chapter);
            }
            if chapter.name == "SQF" {
                sqf(chapter);
            }
            if chapter.name == "Stringtables" {
                stringtables(chapter);
            }
        }
    }
}

fn config(chapter: &mut Chapter) {
    let mut output = String::from("# Lints - Config\n\n");
    let mut lint_text: Vec<(u32, String)> = Vec::new();
    for lint in CONFIG_LINTS.iter().filter(|l| l.display()) {
        lint_text.push((lint.sort(), get_text(&**lint, "L-C")));
    }
    lint_text.sort_by(|a, b| a.0.cmp(&b.0));
    for (_, text) in lint_text {
        output.push_str(&text);
    }
    chapter.content = output;
}

fn sqf(chapter: &mut Chapter) {
    let mut output = String::from("# Lints - SQF\n\n");
    let mut lint_text: Vec<(u32, String)> = Vec::new();
    let lints = SQF_LINTS
        .iter()
        .filter(|l| l.display())
        .map(|l| (**l).clone())
        .chain({
            let lints: Lints<LintData> = vec![
                Arc::new(Box::new(LintS02EventUnknown)),
                Arc::new(Box::new(LintS02EventIncorrectCommand)),
                Arc::new(Box::new(LintS02EventInsufficientVersion)),
            ];
            lints.into_iter()
        })
        .collect::<Vec<_>>();
    for lint in lints {
        lint_text.push((lint.sort(), get_text(&lint, "L-S")));
    }
    lint_text.sort_by(|a, b| a.0.cmp(&b.0));
    for (_, text) in lint_text {
        output.push_str(&text);
    }
    chapter.content = output;
}

fn stringtables(chapter: &mut Chapter) {
    let mut output = String::from("# Lints - Stringtables\n\n");
    let mut lint_text: Vec<(u32, String)> = Vec::new();
    for lint in STRINGTABLE_LINTS.iter().filter(|l| l.display()) {
        lint_text.push((lint.sort(), get_text(&**lint, "L-L")));
    }
    lint_text.sort_by(|a, b| a.0.cmp(&b.0));
    for (_, text) in lint_text {
        output.push_str(&text);
    }
    chapter.content = output;
}

fn get_text<D>(lint: &Arc<Box<dyn Lint<D>>>, prefix: &str) -> String {
    let mut text = String::new();
    text.push_str(&format!("\n***\n## {}\n", lint.ident()));
    text.push_str(&format!("Code: **{prefix}{}**  \n", lint.doc_ident()));
    text.push_str(&format!(
        "Default Severity: **{:?}** {}  \n",
        lint.default_config().severity(),
        match lint.default_config().enabled() {
            LintEnabled::Enabled => "",
            LintEnabled::Disabled => "(Disabled)",
            LintEnabled::Pedantic => "(Pedantic)",
        },
    ));
    text.push_str(&format!(
        "Minimum Severity: {:?}  \n",
        lint.minimum_severity()
    ));
    text.push_str(&format!("\n{}\n", lint.description()));
    text.push_str(&format!("\n{}\n", lint.documentation()));
    text
}
