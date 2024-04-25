use arma3_wiki::{
    model::{Command, Locality, Since, Syntax},
    Wiki,
};
use regex::Regex;
use tower_lsp::lsp_types::{Hover, HoverContents, MarkedString, Position};
use tracing::debug;
use url::Url;

use super::{locate::Locate, SqfCache};

const WIKI: &str = "https://community.bistudio.com/wiki/";

impl SqfCache {
    pub fn hover(&self, uri: Url, position: Position) -> Option<Hover> {
        let files = self.files.read().unwrap();
        if let Some((processed, _, statements)) = files.get(&uri) {
            if let Some(expression) = (processed, statements).locate_expression(uri, position) {
                match expression {
                    hemtt_sqf::Expression::NularCommand(command, _) => {
                        return Some(hover(command.as_str()))
                    }
                    hemtt_sqf::Expression::UnaryCommand(command, _, _) => {
                        return Some(hover(command.as_str()))
                    }
                    hemtt_sqf::Expression::BinaryCommand(command, _, _, _) => {
                        return Some(hover(command.as_str()))
                    }
                    _ => return None,
                }
            }
        } else {
            debug!("No cached file for uri: {} in {:?}", uri, files.keys());
        }
        None
    }
}

fn hover(command: &str) -> Hover {
    let wiki = Wiki::load();
    wiki.commands().get(command).map_or_else(
        || Hover {
            contents: HoverContents::Scalar(MarkedString::String("No documentation found".into())),
            range: None,
        },
        |command| Hover {
            contents: HoverContents::Array({
                let mut contents = Vec::new();
                contents.push(MarkedString::String(format!(
                    "## {}\n[BI Wiki]({WIKI}{})\n\n{}{}{}",
                    command.name(),
                    command.name().replace(' ', "_"),
                    markdown_since(command.since()),
                    markdown_locality(command.argument_loc(), "Argument"),
                    markdown_locality(command.effect_loc(), "Effect"),
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
    let s = markdown_feature(s);
    markdown_links(name, s)
}

fn markdown_links(name: &str, source: String) -> String {
    let mut string = source.clone();
    // [[link|text]] or [[link]]
    let regex = Regex::new(r"(?m)\[\[(.+?)\]\]").unwrap();
    let result = regex.captures_iter(&source);
    for mat in result {
        let link = mat.get(1).unwrap().as_str();
        if link.contains('|') {
            let mut parts = link.split('|');
            let link = parts.next().unwrap();
            let text = parts.next().unwrap();
            string = string.replace(
                mat.get(0).unwrap().as_str(),
                &format!("[{}]({WIKI}{})", text, link.replace(' ', "_")),
            );
            continue;
        }
        string = string.replace(
            &format!("[[{}]]", link),
            &format!("[{}](https://community.bistudio.com/wiki/{})", link, link),
        );
    }

    // {{Link|#Example 5}}
    let regex = Regex::new(r"(?m)\{\{Link\|#(.+?)\}\}").unwrap();
    let source = string.clone();
    let result = regex.captures_iter(&source);
    for mat in result {
        let link = mat.get(1).unwrap().as_str();
        string = string.replace(
            mat.get(0).unwrap().as_str(),
            &format!(
                "[{}](https://community.bistudio.com/wiki/{}{})",
                link,
                name.replace(' ', "_"),
                link.replace(' ', "_"),
            ),
        );
    }
    string
}

fn markdown_feature(source: String) -> String {
    let mut string = source.clone();
    let regex = Regex::new(r"(?mis)\{\{\s?Feature\s?\|\s?(.+?)\s?\|\s?(.+)\}\}").unwrap();
    let result = regex.captures_iter(&source);
    for mat in result {
        let feature = mat.get(1).unwrap().as_str();
        let text = mat.get(2).unwrap().as_str();
        string = string.replace(
            mat.get(0).unwrap().as_str(),
            &format!(
                "### {}\n{}",
                {
                    let mut chars = feature.chars();
                    match chars.next() {
                        None => String::new(),
                        Some(f) => f.to_uppercase().chain(chars).collect(),
                    }
                },
                text
            ),
        );
    }
    string
}

fn markdown_code(source: &str) -> String {
    // <sqf inline>...</sqf>
    let regex = Regex::new(r"(?ms)<sqf inline>(.+?)</sqf>").unwrap();
    let result = regex.captures_iter(source);
    let mut string = source.to_string();
    for mat in result {
        let text = mat.get(1).unwrap().as_str();
        string = string.replace(mat.get(0).unwrap().as_str(), &format!("`{}`", text));
    }

    // <sqf>...</sqf>
    let regex = Regex::new(r"(?ms)<sqf>(.+?)</sqf>").unwrap();
    let result = regex.captures_iter(&string);
    let mut string = string.clone();
    for mat in result {
        let text = mat.get(1).unwrap().as_str();
        string = string.replace(
            mat.get(0).unwrap().as_str(),
            &format!("```sqf\n{}\n```", text),
        );
    }

    // {{hl|text}}
    let regex = Regex::new(r"(?m)\{\{hl\|(.+?)\}\}").unwrap();
    let result = regex.captures_iter(&string);
    let mut string = string.clone();
    for mat in result {
        let text = mat.get(1).unwrap().as_str();
        string = string.replace(mat.get(0).unwrap().as_str(), &format!("`{}`", text));
    }
    string
}

fn markdown_locality(locality: &Locality, context: &str) -> String {
    format!(
        "[{} {}](https://community.bistudio.com/wiki/Multiplayer_Scripting#Locality)\n\n",
        context,
        match locality {
            Locality::Server => "Server".to_string(),
            Locality::Global => "Global".to_string(),
            Locality::Unspecified => return "".to_string(),
            Locality::Local => "Local".to_string(),
        }
    )
}

fn markdown_since(since: &Since) -> String {
    if let Some(arma3) = since.arma_3() {
        format!("Since [Arma 3 {}](https://community.bistudio.com/wiki/Category:Introduced_with_Arma_3_version_{})\n\n", arma3, arma3)
    } else {
        "".to_string()
    }
}

fn markdown_syntax(command: &Command, syntax: &Syntax) -> String {
    let mut string = String::new();
    match syntax.call() {
        arma3_wiki::model::Call::Nular => {
            string.push_str(&format!("```sqf\n{}\n```\n", command.name()));
        }
        arma3_wiki::model::Call::Unary(rhs) => {
            string.push_str(&format!(
                "```sqf\n{} {}\n```\n",
                command.name(),
                markdown_args(&rhs.names())
            ));
        }
        arma3_wiki::model::Call::Binary(lhs, rhs) => {
            string.push_str(&format!(
                "```sqf\n{} {} {}\n```\n",
                markdown_args(&lhs.names()),
                command.name(),
                markdown_args(&rhs.names())
            ));
        }
    }
    for arg in syntax.params() {
        string.push_str(&format!(
            "- `{}`: {}{}\n",
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
                let desc = markdown_links(
                    command.name(),
                    arg.description().unwrap_or_default().to_string(),
                );
                if desc.is_empty() {
                    "".to_string()
                } else {
                    format!(" - {}", desc)
                }
            }
        ));
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
        println!("{:?}", super::markdown_feature(SOURCE.to_string()));
    }

    #[test]
    fn markdown_links() {
        assert_eq!(
            super::markdown_links("setVariable", "[[west]]".to_string()),
            "[west](https://community.bistudio.com/wiki/west)"
        );
        assert_eq!(
            super::markdown_links("setVariable", "[[createLocation|scripted]]".to_string()),
            "[scripted](https://community.bistudio.com/wiki/createLocation)"
        );
        assert_eq!(
            super::markdown_links("setVariable", "See {{Link|#Example 5}}".to_string()),
            "See [Example 5](https://community.bistudio.com/wiki/setVariableExample_5"
        );
        println!(
            "{:?}",
            super::markdown_links("setVariable", SOURCE.to_string())
        );
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
