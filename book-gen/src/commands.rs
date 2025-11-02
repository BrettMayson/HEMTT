use clap::{Command, CommandFactory};
use mdbook::{BookItem, book::Chapter};

pub fn summary_commands() {
    let mut command_text = String::new();
    let command_cli = hemtt::Cli::command();
    let mut commands = command_cli.get_subcommands().collect::<Vec<_>>();
    commands.sort_by(|a, b| a.get_name().cmp(b.get_name()));
    for subcommand in commands {
        if ["wiki", "manage", "utils"].contains(&subcommand.get_name()) {
            continue;
        }
        let subs = subcommand.get_subcommands().collect::<Vec<_>>();
        if subs.is_empty() {
            command_text.push_str(&format!(
                "  - [{}](commands/{}.md)\n",
                subcommand.get_name(),
                subcommand.get_name()
            ));
        } else {
            command_text.push_str(&format!("  - [{}]()\n", subcommand.get_name(),));
            for sub in subs {
                command_text.push_str(&format!(
                    "    - [{}](commands/{}/{}.md)\n",
                    sub.get_name(),
                    subcommand.get_name(),
                    sub.get_name()
                ));
            }
        }
    }
    // Open SUMMARY.md and replace the commands section (between - [Commands](commands/index.md) and - [Rhai](rhai/index.md))
    let summary_path = std::path::Path::new("book/SUMMARY.md");
    let summary_content = std::fs::read_to_string(summary_path).unwrap();
    let mut new_summary_content = String::new();
    let mut in_commands_section = false;
    for line in summary_content.lines() {
        if line.trim() == "- [Commands](commands/index.md)" {
            in_commands_section = true;
            new_summary_content.push_str(line);
            new_summary_content.push('\n');
            new_summary_content.push_str(&command_text);
        } else if line.trim().starts_with("- [Rhai](") {
            in_commands_section = false;
            new_summary_content.push_str(line);
            new_summary_content.push('\n');
        } else if !in_commands_section {
            new_summary_content.push_str(line);
            new_summary_content.push('\n');
        }
    }
    std::fs::write(summary_path, new_summary_content).unwrap();
}

pub fn summary_utilities() {
    let mut utility_text = String::new();
    let command_cli = hemtt::Cli::command();
    let utilities = command_cli
        .get_subcommands()
        .find(|c| c.get_name() == "utils")
        .unwrap()
        .get_subcommands()
        .collect::<Vec<_>>();
    for utility in utilities {
        let subs = utility.get_subcommands().collect::<Vec<_>>();
        if subs.is_empty() {
            utility_text.push_str(&format!(
                "  - [{}](utilities/{}.md)\n",
                utility.get_name(),
                utility.get_name()
            ));
            continue;
        }
        utility_text.push_str(&format!("  - [{}]()\n", utility.get_name(),));
        for sub in subs {
            utility_text.push_str(&format!(
                "    - [{}](utilities/{}/{}.md)\n",
                sub.get_name(),
                utility.get_name(),
                sub.get_name()
            ));
        }
    }
    // Open SUMMARY.md and replace the utilities section (between # Utilities and # Reference)
    let summary_path = std::path::Path::new("book/SUMMARY.md");
    let summary_content = std::fs::read_to_string(summary_path).unwrap();
    let mut new_summary_content = String::new();
    let mut in_utilities_section = false;
    for line in summary_content.lines() {
        if line.trim() == "# Utilities" {
            in_utilities_section = true;
            new_summary_content.push_str(line);
            new_summary_content.push('\n');
            new_summary_content.push('\n');
            new_summary_content.push_str(&utility_text);
            new_summary_content.push('\n');
        } else if line.trim().starts_with("# Reference") {
            in_utilities_section = false;
            new_summary_content.push_str(line);
            new_summary_content.push('\n');
        } else if !in_utilities_section {
            new_summary_content.push_str(line);
            new_summary_content.push('\n');
        }
    }
    std::fs::write(summary_path, new_summary_content).unwrap();
}

pub fn run(chapter: &mut Chapter) {
    let command_cli = hemtt::Cli::command();
    let commands = command_cli.get_subcommands().collect::<Vec<_>>();
    for item in &mut chapter.sub_items {
        if let mdbook::BookItem::Chapter(chapter) = item {
            let command = commands
                .iter()
                .find(|c| *c.get_name() == chapter.name)
                .expect("command exists")
                .to_owned();

            for item in &mut chapter.sub_items {
                let BookItem::Chapter(child_chapter) = item else {
                    continue;
                };
                child_chapter.content = process_command(
                    &child_chapter.name,
                    Some(&chapter.name),
                    command
                        .get_subcommands()
                        .find(|c| *c.get_name() == child_chapter.name)
                        .expect("subcommand exists")
                        .clone(),
                );
            }
            chapter.content = process_command(&chapter.name, None, command.clone());
        }
    }
}

pub fn process_command(name: &str, nested: Option<&str>, mut command: Command) -> String {
    let mut output = format!(
        "# hemtt {}{}\n\n",
        nested.map(|s| format!("{s} ")).unwrap_or_default(),
        name,
    );

    output.push_str("<pre><code class=\"nohighlight\">");
    output.push_str(&global_options(command.render_help().to_string()));
    output.push_str("\n</code></pre>\n\n");

    if let Some(long_about) = command.get_long_about() {
        output.push_str("## Description\n\n");
        output.push_str(&long_about.to_string());
        output.push_str("\n\n");
    }

    let args = command
        .get_arguments()
        .filter(|arg| {
            !(arg.is_global_set()
                || arg.is_hide_set()
                || arg.get_short() == Some('h')
                || arg.get_long() == Some("dir")
                || arg.get_long() == Some("just"))
        })
        .collect::<Vec<_>>();
    if !args.is_empty() {
        output.push_str("## Arguments\n\n");

        for arg in args {
            let mut header = match (arg.get_short(), arg.get_long()) {
                (Some(s), Some(l)) => {
                    format!("-{s}, --{l}")
                }
                (None, Some(l)) => {
                    format!("--{l}")
                }
                (Some(s), None) => {
                    format!("-{s}")
                }
                (None, None) => String::new(),
            };
            if let Some(name) = arg
                .get_value_names()
                .map(|w| w.iter().map(|s| s.to_string()))
                .and_then(|mut l| l.next())
                && matches!(
                    arg.get_action(),
                    clap::ArgAction::Set | clap::ArgAction::Append
                )
            {
                header.push_str(&format!(" &lt;{name}&gt;"));
            }
            output.push_str(&format!("### {header}\n\n"));
            output.push_str(
                &arg.get_long_help()
                    .unwrap_or_else(|| arg.get_help().unwrap_or_default())
                    .to_string(),
            );
            if !arg.get_possible_values().is_empty() {
                output.push_str("\n\nPossible values:\n\n");
                for value in arg.get_possible_values() {
                    output.push_str(&format!(
                        "- {} - {}\n",
                        value.get_name(),
                        value.get_help().unwrap_or_default()
                    ));
                }
            }
            output.push_str("\n\n");
        }
    }
    output
}

fn global_options(usage: String) -> String {
    let mut output = String::new();
    let usage = usage.replace("<", "&lt;").replace(">", "&gt;");
    usage.lines().for_each(|line| {
        let mut line = line.to_string();

        let remove = ["      --dir &lt;DIR&gt;"];
        if remove.iter().any(|x| line.starts_with(x)) {
            return;
        }

        let links = [
            (
                "  -t, --threads",
                r#"  <a href="/commands#-t---threads">-t, --threads</a>"#,
            ),
            ("  -v...", r#"  <a href="/commands#-v">-v...</a>"#),
            (
                "      --just",
                r#"      <a href="/commands#--just">--just</a>"#,
            ),
        ];

        for (from, to) in links.iter() {
            if line.starts_with(from) {
                line = format!("{}{}", to, &line[from.len()..]);
            }
        }

        output.push_str(&line);
        output.push('\n');
    });
    output
}
