use std::{
    path::{Path, PathBuf},
    sync::{atomic::AtomicUsize, Arc},
};

use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use crate::Error;

#[derive(clap::Args)]
pub struct Args {
    path: String,
}

/// Execute the convert command
///
/// # Errors
/// [`Error`] depending on the modules
pub fn execute(args: &Args) -> Result<(), Error> {
    let path = PathBuf::from(&args.path);
    if path.is_dir() {
        let count = Arc::new(AtomicUsize::new(0));
        let entries = walkdir::WalkDir::new(&path)
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?;
        entries
            .par_iter()
            .map(|entry| {
                if entry.file_type().is_file()
                    && entry.path().extension().unwrap_or_default() == "sqf"
                    && file(entry.path())?
                {
                    count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    info!("Fixed case in `{}`", entry.path().display());
                }
                Ok(())
            })
            .collect::<Result<Vec<_>, Error>>()?;
        info!(
            "Fixed case in {} files",
            count.load(std::sync::atomic::Ordering::Relaxed)
        );
    } else if file(&path)? {
        info!("Fixed case in `{}`", path.display());
    } else {
        info!("No changes in `{}`", path.display());
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InsideQuote {
    No,
    Yes,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InsideComment {
    No,
    Single,
    Multi,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InsideIgnoredMacro {
    No,
    Yes,
}

fn file(file: &Path) -> Result<bool, Error> {
    trace!("Fixing case in `{}`", file.display());

    let wiki = arma3_wiki::Wiki::load(false);

    // Read the file
    let content = std::fs::read_to_string(file)?;

    // create a buffer and read each word at a time, ignoring anything inside of quotes
    // break on any non-alphanumeric character

    let mut out = String::with_capacity(content.len());
    let mut buffer = String::new();
    let mut in_quotes = InsideQuote::No;
    let mut in_comment = InsideComment::No;
    let mut inside_ignored_macro = InsideIgnoredMacro::No;
    for char in content.chars() {
        match (char, in_quotes, in_comment, inside_ignored_macro) {
            (
                'a'..='z' | 'A'..='Z' | '0'..='9' | '_',
                InsideQuote::No,
                InsideComment::No,
                InsideIgnoredMacro::No,
            ) => {
                buffer.push(char);
            }
            ('"', _, InsideComment::No, _) => {
                if in_quotes == InsideQuote::No {
                    check_buffer(&mut out, &mut buffer, &wiki);
                    in_quotes = InsideQuote::Yes;
                } else {
                    in_quotes = InsideQuote::No;
                }
                out.push(char);
            }
            ('/', InsideQuote::No, InsideComment::No, _) => {
                check_buffer(&mut out, &mut buffer, &wiki);
                if out.chars().last().map_or(false, |c| c == '/') {
                    in_comment = InsideComment::Single;
                }
                out.push(char);
            }
            ('\n', InsideQuote::No, _, _) => {
                check_buffer(&mut out, &mut buffer, &wiki);
                if in_comment == InsideComment::Single {
                    in_comment = InsideComment::No;
                }
                out.push(char);
            }
            ('*', InsideQuote::No, InsideComment::No, _) => {
                check_buffer(&mut out, &mut buffer, &wiki);
                if out.chars().last().map_or(false, |c| c == '/') {
                    in_comment = InsideComment::Multi;
                }
                out.push(char);
            }
            ('/', InsideQuote::No, InsideComment::Multi, _) => {
                if out.chars().last().map_or(false, |c| c == '*') {
                    in_comment = InsideComment::No;
                }
                out.push(char);
            }
            ('(', InsideQuote::No, InsideComment::No, InsideIgnoredMacro::No) => {
                // ignore arguments to macros
                if !buffer.is_empty() && buffer.to_uppercase() == buffer {
                    inside_ignored_macro = InsideIgnoredMacro::Yes;
                }
                if matches!(buffer.as_str(), "LOG") {
                    out.push_str(&buffer);
                    buffer.clear();
                } else {
                    check_buffer(&mut out, &mut buffer, &wiki);
                }
                out.push(char);
            }
            (')' | ',', InsideQuote::No, InsideComment::No, InsideIgnoredMacro::Yes) => {
                inside_ignored_macro = InsideIgnoredMacro::No;
                out.push_str(&buffer);
                buffer.clear();
                out.push(char);
            }
            (_, InsideQuote::No, InsideComment::No, _) => {
                check_buffer(&mut out, &mut buffer, &wiki);
                out.push(char);
            }
            (_, InsideQuote::Yes, _, _)
            | (_, _, InsideComment::Single | InsideComment::Multi, _) => {
                out.push(char);
            }
        }
    }
    // flush buffer if we ended on non-whitepsace
    check_buffer(&mut out, &mut buffer, &wiki);

    // Write the file
    if content != out {
        std::fs::write(file, out)?;
        return Ok(true);
    }
    Ok(false)
}

fn check_buffer(out: &mut String, buffer: &mut String, wiki: &arma3_wiki::Wiki) {
    if buffer.is_empty() {
        return;
    }
    debug!("Checking buffer `{}`", buffer);
    if let Some(command) = wiki.commands().get(buffer) {
        out.push_str(command.name());
    } else {
        out.push_str(buffer);
    }
    buffer.clear();
}
