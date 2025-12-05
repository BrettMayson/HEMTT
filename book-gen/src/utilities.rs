use clap::CommandFactory;
use mdbook_preprocessor::book::{BookItem, Chapter};

use crate::commands::process_command;

pub fn run(chapter: &mut Chapter) {
    let command_cli = hemtt::Cli::command();
    let utilities = command_cli
        .get_subcommands()
        .find(|c| c.get_name() == "utils")
        .unwrap()
        .get_subcommands()
        .collect::<Vec<_>>();
    for chapter in &mut chapter.sub_items {
        let BookItem::Chapter(chapter) = chapter else {
            continue;
        };
        if chapter.sub_items.is_empty() {
            // no subcommands, process as a single utility
            let utility = utilities
                .iter()
                .find(|c| *c.get_name() == chapter.name)
                .expect("utility exists")
                .to_owned()
                .to_owned();
            chapter.content = process_command(&chapter.name, Some("utils"), utility);
            continue;
        }
        for item in &mut chapter.sub_items {
            let command = utilities
                .iter()
                .find(|c| *c.get_name() == chapter.name)
                .expect("utility exists")
                .to_owned()
                .to_owned();
            if let BookItem::Chapter(chapter) = item {
                let utility = command
                    .get_subcommands()
                    .find(|c| *c.get_name() == chapter.name)
                    .expect("utility exists")
                    .to_owned()
                    .to_owned();
                chapter.content = process_command(
                    &chapter.name,
                    Some(format!("utils {}", command.get_name()).as_str()),
                    utility,
                );
            }
        }
    }
}
