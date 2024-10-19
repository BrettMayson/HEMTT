use std::sync::Arc;

use arma3_wiki::Wiki;
use hemtt_config::CONFIG_LINTS;
use hemtt_sqf::analyze::{
    lints::s02_event_handlers::{
        LintS02EventIncorrectCommand, LintS02EventInsufficientVersion, LintS02EventUnknown,
    },
    SqfLintData, SQF_LINTS,
};
use hemtt_workspace::lint::Lints;
use mdbook::{book::Chapter, preprocess::CmdPreprocessor};

fn main() {
    if std::env::args().nth(1) == Some("supports".to_string()) {
        highlight();
        return;
    }

    let (_ctx, mut book) = CmdPreprocessor::parse_input(std::io::stdin()).unwrap();

    for section in &mut book.sections {
        if let mdbook::BookItem::Chapter(chapter) = section {
            if chapter.name == "Analysis" {
                for item in &mut chapter.sub_items {
                    if let mdbook::BookItem::Chapter(ref mut chapter) = item {
                        if chapter.name == "Config" {
                            config(chapter);
                        }
                        if chapter.name == "SQF" {
                            sqf(chapter);
                        }
                    }
                }
            }
        }
    }

    serde_json::to_writer(std::io::stdout(), &book).unwrap();
}

fn config(chapter: &mut Chapter) {
    let mut output = String::from("# Lints - Conifg\n\n");
    let mut lint_text: Vec<(u32, String)> = Vec::new();
    for lint in CONFIG_LINTS.iter() {
        let mut text = String::new();
        text.push_str(&format!("\n***\n## {}\n", lint.ident()));
        text.push_str(&format!("Code: **L-C{}**  \n", lint.doc_ident()));
        text.push_str(&format!(
            "Default Severity: **{:?}** {}  \n",
            lint.default_config().severity(),
            if lint.default_config().enabled() {
                ""
            } else {
                "(Disabled)"
            },
        ));
        text.push_str(&format!(
            "Minimum Severity: {:?}  \n",
            lint.minimum_severity()
        ));
        text.push_str(&format!("\n{}\n", lint.description()));
        text.push_str(&format!("\n{}\n", lint.documentation()));
        lint_text.push((lint.sort(), text));
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
        .map(|l| (**l).clone())
        .chain({
            let lints: Lints<SqfLintData> = vec![
                Arc::new(Box::new(LintS02EventUnknown)),
                Arc::new(Box::new(LintS02EventIncorrectCommand)),
                Arc::new(Box::new(LintS02EventInsufficientVersion)),
            ];
            lints.into_iter()
        })
        .collect::<Vec<_>>();
    for lint in lints {
        let mut text = String::new();
        text.push_str(&format!("\n***\n## {}\n", lint.ident()));
        text.push_str(&format!("Code: **L-S{}**  \n", lint.doc_ident()));
        text.push_str(&format!(
            "Default Severity: **{:?}** {}  \n",
            lint.default_config().severity(),
            if lint.default_config().enabled() {
                ""
            } else {
                "(Disabled)"
            },
        ));
        text.push_str(&format!(
            "Minimum Severity: {:?}  \n",
            lint.minimum_severity()
        ));
        text.push_str(&format!("\n{}\n", lint.description()));
        text.push_str(&format!("\n{}\n", lint.documentation()));
        lint_text.push((lint.sort(), text));
    }
    lint_text.sort_by(|a, b| a.0.cmp(&b.0));
    for (_, text) in lint_text {
        output.push_str(&text);
    }
    chapter.content = output;
}

fn highlight() {
    let wiki = Wiki::load(true);

    let mut flow = Vec::with_capacity(500);
    let mut commands = Vec::with_capacity(3000);

    for command in wiki.commands().raw().values() {
        let name = command.name();
        if name.contains(' ') || name.contains('%') || name.contains('_') || name.contains('+') {
            continue;
        }
        if !name.is_ascii() {
            continue;
        }
        let dest = if command.groups().iter().any(|x| x == "Program Flow") {
            &mut flow
        } else {
            &mut commands
        };
        dest.push(command.name());
    }

    // Remove special commands
    commands.retain(|x| {
        ![
            "call",
            "callExtension",
            "compile",
            "compileFinal",
            "exec",
            "execFSM",
            "execVM",
            "private",
            "spawn",
        ]
        .contains(x)
    });

    let highlight = std::fs::read_to_string("book-lints/highlight.js").unwrap();

    let highlight = highlight.replace("$FLOW$", &format!("'{}'", flow.join("','")));
    let highlight = highlight.replace("$COMMANDS$", &format!("'{}'", commands.join("','")));

    std::fs::write("book/highlight.js", highlight).unwrap();
}
