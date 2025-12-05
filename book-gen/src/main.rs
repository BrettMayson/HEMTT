use mdbook_preprocessor::{book::BookItem, parse_input};

mod commands;
mod highlight;
mod lints;
mod utilities;

fn main() {
    if std::env::args().nth(1) == Some("supports".to_string()) {
        highlight::run();
        commands::summary_commands();
        commands::summary_utilities();
        return;
    }

    let (_ctx, mut book) = parse_input(std::io::stdin()).unwrap();

    for section in &mut book.items {
        if let BookItem::Chapter(chapter) = section {
            if chapter.name == "Lints" {
                lints::run(chapter);
            } else if chapter.name == "Commands" {
                commands::run(chapter);
            } else if chapter.name == "Utilities" {
                utilities::run(chapter);
            }
        }
    }

    serde_json::to_writer(std::io::stdout(), &book).unwrap();
}
