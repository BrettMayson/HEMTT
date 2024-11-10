use mdbook::preprocess::CmdPreprocessor;

mod commands;
mod highlight;
mod lints;

fn main() {
    if std::env::args().nth(1) == Some("supports".to_string()) {
        highlight::run();
        return;
    }

    let (_ctx, mut book) = CmdPreprocessor::parse_input(std::io::stdin()).unwrap();

    for section in &mut book.sections {
        if let mdbook::BookItem::Chapter(chapter) = section {
            if chapter.name == "Analysis" {
                lints::run(chapter);
            } else if chapter.name == "Commands" {
                commands::run(chapter);
            }
        }
    }

    serde_json::to_writer(std::io::stdout(), &book).unwrap();
}
