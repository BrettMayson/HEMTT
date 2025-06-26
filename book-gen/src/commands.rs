use clap::{Command, CommandFactory};
use mdbook::book::Chapter;

pub fn run(chapter: &mut Chapter) {
    let commands = vec![
        ("new", hemtt::commands::new::Command::command()),
        ("check", hemtt::commands::check::Command::command()),
        ("dev", hemtt::commands::dev::Command::command()),
        ("launch", hemtt::commands::launch::Command::command()),
        ("build", hemtt::commands::build::Command::command()),
        ("release", hemtt::commands::release::Command::command()),
        ("script", hemtt::commands::script::Command::command()),
    ];

    let nested = [(
        "localization",
        vec![
            (
                "coverage",
                hemtt::commands::localization::coverage::Command::command(),
            ),
            (
                "sort",
                hemtt::commands::localization::sort::Command::command(),
            ),
        ],
    )];

    for item in &mut chapter.sub_items {
        if let mdbook::BookItem::Chapter(chapter) = item {
            if let Some((name, command)) = commands.iter().find(|(name, _)| *name == chapter.name) {
                chapter.content = process_command(name, None, command.clone());
            } else if let Some((_, commands)) =
                nested.iter().find(|(name, _)| *name == chapter.name)
            {
                for item in &mut chapter.sub_items {
                    if let mdbook::BookItem::Chapter(child_chapter) = item {
                        if let Some((name, command)) = commands
                            .iter()
                            .find(|(name, _)| *name == child_chapter.name)
                        {
                            child_chapter.content =
                                process_command(name, Some(&chapter.name), command.clone());
                        }
                    }
                }
            }
        }
    }
}

fn process_command(name: &str, nested: Option<&str>, mut command: Command) -> String {
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
            {
                if matches!(
                    arg.get_action(),
                    clap::ArgAction::Set | clap::ArgAction::Append
                ) {
                    header.push_str(&format!(" &lt;{name}&gt;"));
                }
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
