use std::fmt::Write;

use arma3_wiki::model::{Command, Locality, Since, Syntax};
use hemtt_sqf::parser::database::Database;
use hemtt_workspace::reporting::Symbol;
use regex::Regex;
use tower_lsp::lsp_types::{Hover, HoverContents, MarkedString, Position};
use tracing::warn;
use url::Url;

use crate::workspace::EditorWorkspaces;

use super::SqfAnalyzer;

pub const WIKI: &str = "https://community.bistudio.com/wiki/";

impl SqfAnalyzer {
    pub async fn hover(&self, url: Url, position: Position) -> Option<Hover> {
        if !std::path::Path::new(url.path())
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("sqf"))
        {
            return None;
        }
        let Some(workspace) = EditorWorkspaces::get().guess_workspace_retry(&url).await else {
            warn!("Failed to find workspace for {:?}", url);
            return None;
        };
        let database = self.get_database(&workspace);
        let Some(tokens) = self.tokens.get(&url) else {
            warn!("No tokens found for {:?}", url);
            return None;
        };
        #[allow(clippy::int_plus_one)]
        let token = tokens.iter().find(|token| {
            let start = token.position().start();
            let end = token.position().end();
            (u32::try_from(start.line()).expect("Failed to convert start line")
                == position.line + 1)
                && (u32::try_from(end.line()).expect("Failed to convert end line")
                    == position.line + 1)
                && (u32::try_from(start.column()).expect("Failed to convert start column")
                    <= position.character)
                && (u32::try_from(end.column()).expect("Failed to convert end column")
                    >= position.character)
        })?;
        let Symbol::Word(word) = token.symbol() else {
            return None;
        };
        println!("Hover word: {word}");
        if let Some(func) = database.external_functions_get(&word.to_lowercase()) {
            return Some(hover_func(func));
        }
        database.wiki().commands().get(word)?;
        Some(hover(word, &database))
    }
}

// WIP
fn hover_func(func: &hemtt_sqf::analyze::inspector::headers::FunctionInfo) -> Hover {
    Hover {
        contents: HoverContents::Array({
            let mut contents = Vec::new();
            contents.push(MarkedString::String(format!(
                "## {}",
                func.func_name().unwrap_or(&String::new())
            )));
            {
                let mut string = String::new();
                for arg in func.params() {
                    writeln!(
                        string,
                        "- `{}`: {}{}",
                        arg.name(),
                        {
                            let typ = arg.typ().to_string();
                            if typ == "Unknown" {
                                typ
                            } else {
                                format!(
                                    "[{}](https://community.bistudio.com/wiki/{})",
                                    typ,
                                    typ.replace(' ', "_")
                                )
                            }
                        },
                        { arg.description().unwrap_or("?") }
                    )
                    .expect("Failed to write to string");
                }
                contents.push(MarkedString::String(format!("### Syntax\n{string}")));
            }
            if let Some(ret) = func.ret() {
                contents.push(MarkedString::String(format!(
                    "### Return Type\n- [{}](https://community.bistudio.com/wiki/{})",
                    ret,
                    ret.to_string().replace(' ', "_")
                )));
            }
            let example = func.example();
            if !example.is_empty() {
                contents.push(MarkedString::String(format!("### Example\n{example}")));
            }
            contents
        }),
        range: None,
    }
}

fn hover(command: &str, database: &Database) -> Hover {
    database.wiki().commands().get(command).map_or_else(
        || Hover {
            contents: HoverContents::Scalar(MarkedString::String("No documentation found".into())),
            range: None,
        },
        |command| Hover {
            contents: HoverContents::Array({
                let mut contents = Vec::new();
                contents.push(MarkedString::String(format!(
                    "## {}\n{}\n\n{}{}{}",
                    command.name(),
                    if database.wiki().is_custom_command(command.name()) {
                        "Custom Command".to_string()
                    } else {
                        format!("[BI Wiki]({WIKI}{})", command.name().replace(' ', "_"))
                    },
                    markdown_since(command.since()),
                    markdown_locality(*command.argument_loc(), "Argument"),
                    markdown_locality(*command.effect_loc(), "Effect"),
                )));
                contents.push(MarkedString::String(markdown(
                    command.name(),
                    command.description(),
                )));
                for syntax in command.syntax() {
                    contents.push(MarkedString::String(format!(
                        "### Syntax\n{}",
                        markdown_syntax(command, syntax)
                    )));
                }
                for example in command.examples() {
                    contents.push(MarkedString::String(format!(
                        "### Example\n{}",
                        markdown(command.name(), example),
                    )));
                }
                contents
            }),
            range: None,
        },
    )
}

fn markdown(name: &str, s: &str) -> String {
    let s = markdown_code(s);
    let s = markdown_feature(&s);
    markdown_links(name, &s)
}

fn markdown_links(name: &str, source: &str) -> String {
    let mut string = source.to_string();
    // [[link|text]] or [[link]]
    let regex = Regex::new(r"(?m)\[\[(.+?)\]\]").expect("Failed to compile regex");
    let result = regex.captures_iter(source);
    for mat in result {
        let link = mat.get(1).expect("Failed to get capture group 1").as_str();
        if link.contains('|') {
            let mut parts = link.split('|');
            let link = parts.next().expect("Failed to get link part");
            let text = parts.next().expect("Failed to get text part");
            string = string.replace(
                mat.get(0).expect("Failed to get full match").as_str(),
                &format!("[{}]({WIKI}{})", text, link.replace(' ', "_")),
            );
            continue;
        }
        string = string.replace(
            &format!("[[{link}]]"),
            &format!("[{link}](https://community.bistudio.com/wiki/{link})"),
        );
    }

    // {{Link|Example 5}}
    let regex = Regex::new(r"(?m)\{\{Link\|(.+?)\}\}").expect("Failed to compile regex");
    let source = string.clone();
    let result = regex.captures_iter(&source);
    for mat in result {
        let link = mat.get(1).expect("Failed to get capture group 1").as_str();
        string = string.replace(
            mat.get(0).expect("Failed to get full match").as_str(),
            &if link.starts_with('#') {
                format!(
                    "[{}](https://community.bistudio.com/wiki/{}{})",
                    link.trim_start_matches('#'),
                    name.replace(' ', "_"),
                    link.replace(' ', "_"),
                )
            } else {
                format!("[{link}](https://community.bistudio.com/wiki/{link})")
            },
        );
    }
    string
}

fn markdown_feature(source: &str) -> String {
    let mut string = source.to_string();
    let regex = Regex::new(r"(?mis)\{\{\s?Feature\s?\|\s?(.+?)\s?\|\s?(.+)\}\}")
        .expect("Failed to compile regex");
    let result = regex.captures_iter(source);
    for mat in result {
        let feature = mat.get(1).expect("Failed to get capture group 1").as_str();
        let text = mat.get(2).expect("Failed to get capture group 2").as_str();
        string = string.replace(
            mat.get(0).expect("Failed to get full match").as_str(),
            &format!(
                "### {}\n{}",
                {
                    let mut chars = feature.chars();
                    chars
                        .next()
                        .map_or_else(String::new, |f| f.to_uppercase().chain(chars).collect())
                },
                text
            ),
        );
    }
    string
}

fn markdown_code(source: &str) -> String {
    // <sqf inline>...</sqf>
    let regex = Regex::new(r"(?ms)<sqf inline>(.+?)</sqf>").expect("Failed to compile regex");
    let result = regex.captures_iter(source);
    let mut string = source.to_string();
    for mat in result {
        let text = mat.get(1).expect("Failed to get capture group 1").as_str();
        string = string.replace(
            mat.get(0).expect("Failed to get full match").as_str(),
            &format!("`{text}`"),
        );
    }

    // <sqf>...</sqf>
    let regex = Regex::new(r"(?ms)<sqf>(.+?)</sqf>").expect("Failed to compile regex");
    let result = regex.captures_iter(&string);
    let mut string = string.clone();
    for mat in result {
        let text = mat.get(1).expect("Failed to get capture group 1").as_str();
        string = string.replace(
            mat.get(0).expect("Failed to get full match").as_str(),
            &format!("```sqf\n{text}\n```"),
        );
    }

    // {{hl|text}}
    let regex = Regex::new(r"(?m)\{\{hl\|(.+?)\}\}").expect("Failed to compile regex");
    let result = regex.captures_iter(&string);
    let mut string = string.clone();
    for mat in result {
        let text = mat.get(1).expect("Failed to get capture group 1").as_str();
        string = string.replace(
            mat.get(0).expect("Failed to get full match").as_str(),
            &format!("`{text}`"),
        );
    }
    string
}

fn markdown_locality(locality: Locality, context: &str) -> String {
    format!(
        "[{} {}](https://community.bistudio.com/wiki/Multiplayer_Scripting#Locality)\n\n",
        context,
        match locality {
            Locality::Server => "Server".to_string(),
            Locality::Global => "Global".to_string(),
            Locality::Unspecified => return String::new(),
            Locality::Local => "Local".to_string(),
        }
    )
}

fn markdown_since(since: &Since) -> String {
    since.arma_3().map_or_else(String::new, |arma3| format!(
            "Since [Arma 3 {arma3}](https://community.bistudio.com/wiki/Category:Introduced_with_Arma_3_version_{arma3})\n\n"
        ))
}

fn markdown_syntax(command: &Command, syntax: &Syntax) -> String {
    let mut string = String::new();
    match syntax.call() {
        arma3_wiki::model::Call::Nular => {
            write!(string, "```sqf\n{}\n```\n", command.name()).expect("Failed to write to string");
        }
        arma3_wiki::model::Call::Unary(rhs) => {
            write!(
                string,
                "```sqf\n{} {}\n```\n",
                command.name(),
                markdown_args(&rhs.names())
            )
            .expect("Failed to write to string");
        }
        arma3_wiki::model::Call::Binary(lhs, rhs) => {
            write!(
                string,
                "```sqf\n{} {} {}\n```\n",
                markdown_args(&lhs.names()),
                command.name(),
                markdown_args(&rhs.names())
            )
            .expect("Failed to write to string");
        }
    }
    for arg in syntax.params() {
        writeln!(
            string,
            "- `{}`: {}{}",
            arg.name(),
            {
                let typ = arg.typ().to_string();
                if typ == "Unknown" {
                    typ
                } else {
                    format!(
                        "[{}](https://community.bistudio.com/wiki/{})",
                        typ,
                        typ.replace(' ', "_")
                    )
                }
            },
            {
                let desc = markdown_links(command.name(), arg.description().unwrap_or_default());
                if desc.is_empty() {
                    String::new()
                } else {
                    format!(" - {desc}")
                }
            }
        )
        .expect("Failed to write to string");
    }
    string
}

fn markdown_args(args: &[String]) -> String {
    if args.len() == 1 {
        args[0].clone()
    } else {
        format!("[{}]", args.join(", "))
    }
}

#[cfg(test)]
mod tests {
    const SOURCE: &str = r#"Set variable to given value in the variable space of given element. Can be used to broadcast variables over the network.<br>\nTo remove a variable, set it to [[nil]] (see {{Link|#Example 5}}) - note that this does not work on [[Object]] and [[createLocation|scripted]] [[Location]] namespaces (the variable will be set to [[nil]] but will remain listed by [[allVariables]]).\n{{Feature|warning|\n[[missionNamespace]], [[uiNamespace]], [[parsingNamespace]] and [[profileNamespace]] variables '''cannot''' be named as commands - e.g <sqf inline>missionNamespace setVariable ["west", 123];</sqf> conflicts with the [[west]] command and will result in a {{hl|Reserved variable in expression}} error, [[west]] being a scripting command (other namespaces do not have such limitation).\nSee also [[:Category:Scripting Commands|all available script commands]].\n}}"#;

    #[test]
    fn markdown() {
        println!("{:?}", super::markdown("setVariable", SOURCE));
    }

    #[test]
    fn markdown_feature() {
        println!("{:?}", super::markdown_feature(SOURCE));
    }

    #[test]
    fn markdown_links() {
        assert_eq!(
            super::markdown_links("setVariable", "[[west]]"),
            "[west](https://community.bistudio.com/wiki/west)"
        );
        assert_eq!(
            super::markdown_links("setVariable", "[[createLocation|scripted]]"),
            "[scripted](https://community.bistudio.com/wiki/createLocation)"
        );
        assert_eq!(
            super::markdown_links("setVariable", "See {{Link|#Example 5}}"),
            "See [Example 5](https://community.bistudio.com/wiki/setVariable#Example_5)"
        );
        println!("{:?}", super::markdown_links("setVariable", SOURCE));
    }

    #[test]
    fn markdown_code() {
        assert_eq!(
            super::markdown_code("{{hl|Reserved variable in expression}}",),
            "`Reserved variable in expression`"
        );
        assert_eq!(
            super::markdown_code(
                "{{hl|Reserved variable in expression}}\n{{hl|Reserved variable in expression}}",
            ),
            "`Reserved variable in expression`\n`Reserved variable in expression`"
        );
        assert_eq!(
            super::markdown_code("<sqf>missionNamespace setVariable [\"west\", 123];</sqf>",),
            "```sqf\nmissionNamespace setVariable [\"west\", 123];\n```"
        );
        assert_eq!(
            super::markdown_code(
                "{{hl|Reserved variable in expression}}\n{{hl|Reserved variable in expression}}",
            ),
            "`Reserved variable in expression`\n`Reserved variable in expression`"
        );
    }
}
