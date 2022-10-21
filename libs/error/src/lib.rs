use std::{io::BufRead, path::Path};

use colored::Colorize;
use hemtt_tokens::{position::Position, Token};

pub struct AppError {
    pub brief: String,
    pub details: Option<String>,
    pub help: Option<String>,
    pub source: Option<Source>,
}

pub enum DisplayStyle {
    Info,
    Warning,
    Error,
}

impl AppError {
    pub fn short(&self) -> &str {
        &self.brief
    }

    pub fn long(&self, style: DisplayStyle) -> String {
        format!(
            "{}{}\n{}{}{}",
            match style {
                DisplayStyle::Info => format!("{}: ", "info".bright_blue()).bold(),
                DisplayStyle::Warning => format!("{}: ", "warning".bright_yellow()).bold(),
                DisplayStyle::Error => format!("{}: ", "error".bright_red()).bold(),
            },
            self.brief.bold(),
            self.details.clone().unwrap_or_default(),
            self.help.clone().unwrap_or_default(),
            self.source().unwrap_or_default()
        )
    }

    pub fn source(&self) -> Option<String> {
        let source = self.source.as_ref()?;
        Some(format!(
            "   {} {}:{}:{}\n{}\n",
            "-->".blue(),
            source.position.path(),
            source.position.start().1 .0,
            source.position.start().1 .1,
            {
                let bar = "    |".blue();
                let mut lines = String::new();
                for (i, line) in source.lines.iter().enumerate() {
                    let linenum = source.position.start().1 .0 + i;
                    lines.push_str(&format!(
                        "{: >3} {} {}\n",
                        linenum.to_string().blue(),
                        "|".blue(),
                        line
                    ));
                }
                lines.push_str(&format!(
                    "{} {:>offset$} {}",
                    bar,
                    "^".red(),
                    source.note.red(),
                    offset = source.position.start().1 .1
                ));
                format!("{}\n{}", bar, lines)
            }
        ))
    }
}

impl<E> From<E> for AppError
where
    E: PrettyError,
{
    fn from(e: E) -> Self {
        Self {
            brief: e.brief(),
            details: e.details(),
            help: e.help(),
            source: e.source(),
        }
    }
}

pub trait PrettyError: ToString {
    fn brief(&self) -> String {
        self.to_string()
    }
    fn details(&self) -> Option<String> {
        None
    }
    fn help(&self) -> Option<String> {
        None
    }
    fn source(&self) -> Option<Source> {
        None
    }
}

pub struct Source {
    pub lines: Vec<String>,
    pub position: Position,
    pub note: String,
}

pub fn read_lines_from_file(
    path: &Path,
    start: usize,
    end: usize,
) -> Result<Vec<String>, std::io::Error> {
    let file = std::fs::File::open(path)?;
    let reader = std::io::BufReader::new(file);
    let mut lines = reader.lines();
    for _ in 1..start {
        lines.next().unwrap().unwrap();
    }
    let mut ret = Vec::new();
    for _ in 0..=(end - start) {
        if let Some(x) = lines.next() {
            ret.push(x.unwrap().trim_end().to_string());
        }
    }
    Ok(ret)
}

pub fn make_source(token: &Token, note: String) -> Source {
    Source {
        lines: read_lines_from_file(
            Path::new(token.source().path()),
            token.source().start().1 .0,
            token.source().end().1 .0,
        )
        .unwrap(),
        position: token.source().clone(),
        note,
    }
}
