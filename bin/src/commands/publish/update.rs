use std::path::PathBuf;

use hemtt_common::version::Version;
use markdown::{Constructs, ParseOptions, mdast::Node};
use markdown_bbcode::MdToBbcode;
use regex::Regex;
use steamworks::{PublishedFileId, UGC};

use crate::{Error, commands::publish::APP_ID, report::Report, utils::bytes_to_human_readable};

/// Everything that would be sent to Steam for a create or update
struct Preview {
    report: Report,
    /// The existing published item ID, if any. [`None`] means a new item will be created.
    id: Option<PublishedFileId>,
    content_path: PathBuf,
    content_size: u64,
    title: String,
    version: Version,
    description: Option<String>,
    changelog: String,
}

/// Build the release and gather everything that would be sent to Steam
fn prepare(cmd: &super::Command, id: Option<PublishedFileId>) -> Result<Preview, Error> {
    let mut executor = crate::commands::release::executor(&cmd.release, &cmd.build)?;
    let version = executor
        .ctx()
        .config()
        .version()
        .get(executor.ctx().workspace_path().vfs())?;
    let report = executor.run()?;

    let Some(content_path) = executor.ctx().build_folder().cloned() else {
        panic!("Build folder not found");
    };
    let content_size = dir_size(&content_path);

    let title = executor.ctx().config().name().to_string();

    let description =
        if let Some(description) = executor.ctx().config().hemtt().publish().description() {
            let content =
                markdown_headings_to_steam(&strip_html(&fs_err::read_to_string(description)?));
            let mut buf = Vec::new();
            MdToBbcode::new(&content, &mut buf)
                .serialize()
                .expect("valid markdown");
            Some(String::from_utf8(buf).expect("valid UTF-8"))
        } else {
            None
        };

    let changelog = if let Some(changelog) = executor.ctx().config().hemtt().publish().changelog() {
        let content = fs_err::read_to_string(changelog)?;
        let changelog = parse_changelog::parse(&content).expect("valid changelog");
        let Some(change) = changelog.get(version.to_string().as_str()) else {
            error!("No changelog entry found for version {}", version);
            std::process::exit(1);
        };

        markdown_headings_to_steam(&strip_html(&format!(
            "# Version {}\n\n{}",
            version, change.notes
        )))
    } else {
        format!("Version {version}")
    };

    Ok(Preview {
        report,
        id,
        content_path,
        content_size,
        title,
        version,
        description,
        changelog,
    })
}

pub fn execute(cmd: &super::Command, ugc: &UGC, create: bool) -> Result<Report, Error> {
    let Ok(id) = super::get_id() else {
        error!(
            "Failed to get published file ID, add it to meta.cpp or run `hemtt publish` to create a new item."
        );
        std::process::exit(1);
    };

    let preview = prepare(cmd, Some(id))?;

    let mut handle = ugc
        .start_item_update(APP_ID, id)
        .content_path(&preview.content_path);
    if create {
        handle = handle
            .title(&preview.title)
            .tags(vec!["Mod"], true)
            .add_key_value_tag("bis_platform", "-")
            .add_key_value_tag("bis_displayName", &preview.title);
    }
    if let Some(description) = &preview.description {
        handle = handle.description(description);
    }
    let _upload_handle = handle.submit(
        Some(&preview.changelog),
        |upload_result| match upload_result {
            Ok((published_id, needs_to_agree_to_terms)) => {
                info!("Uploaded item with id {:?}", published_id);
                if needs_to_agree_to_terms {
                    warn!("You need to agree to the terms of use before you can upload any files");
                }
            }
            Err(e) => {
                error!("Error uploading item: {:?}", e);
            }
        },
    );
    if supports_hyperlinks::on(supports_hyperlinks::Stream::Stdout) {
        let text = id.0.to_string();
        let url = format!(
            "https://steamcommunity.com/sharedfiles/filedetails/?id={}",
            id.0
        );
        let link = terminal_link::Link::new(&text, &url);
        println!("Updated on Steam Workshop: {link}");
    } else {
        println!(
            "Updated on Steam Workshop: https://steamcommunity.com/sharedfiles/filedetails/?id={}",
            id.0
        );
    }
    Ok(preview.report)
}

/// Build the release and print a preview of what would be submitted to Steam,
/// without connecting to Steam or uploading anything.
pub fn dry_run(cmd: &super::Command, id: Option<PublishedFileId>) -> Result<Report, Error> {
    let preview = prepare(cmd, id)?;

    info!("Dry run, nothing was submitted to Steam");
    println!();
    match preview.id {
        Some(id) => {
            if supports_hyperlinks::on(supports_hyperlinks::Stream::Stdout) {
                let text = id.0.to_string();
                let url = format!(
                    "https://steamcommunity.com/sharedfiles/filedetails/?id={}",
                    id.0
                );
                let link = terminal_link::Link::new(&text, &url);
                println!("Workshop Item: {link} (existing item will be updated)");
            } else {
                println!("Workshop Item: #{} (existing item will be updated)", id.0);
            }
        }
        None => println!("Workshop Item: none found, a new item would be created"),
    }
    println!("Version: {}", preview.version);
    println!(
        "Content: {} ({})",
        preview.content_path.display(),
        bytes_to_human_readable(preview.content_size)
    );
    if preview.id.is_none() {
        println!("Title: {}", preview.title);
        println!("Tags: Mod");
    }
    match &preview.description {
        Some(description) => println!("\nDescription:\n{description}"),
        None => println!("\nDescription: unchanged"),
    }
    println!("\nChangelog:\n{}", preview.changelog);

    Ok(preview.report)
}

/// Sum the size in bytes of all files within `path`
fn dir_size(path: &std::path::Path) -> u64 {
    walkdir::WalkDir::new(path)
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .filter_map(|entry| entry.metadata().ok())
        .map(|metadata| metadata.len())
        .sum()
}

/// Replace raw HTML blocks in markdown source with Steam-friendly text.
///
/// Steam Workshop does not render HTML, so leaving it as-is would show the
/// raw tags as garbage text on the item page. Headings and paragraphs
/// usually carry useful information (e.g. a title or requirements notice),
/// so their text content is kept and converted, while purely decorative
/// markup (badges, images, links used only for styling, etc.) is dropped.
fn strip_html(input: &str) -> String {
    let options = ParseOptions {
        constructs: Constructs::gfm(),
        ..Default::default()
    };
    let Ok(root) = markdown::to_mdast(input, &options) else {
        return input.to_string();
    };

    let mut ranges = Vec::new();
    collect_html_ranges(&root, &mut ranges);
    if ranges.is_empty() {
        return input.to_string();
    }
    ranges.sort_by_key(|(start, _)| *start);

    let mut output = String::with_capacity(input.len());
    let mut last = 0;
    for (start, end) in ranges {
        if start < last {
            // nested/overlapping range already covered
            continue;
        }
        output.push_str(&input[last..start]);
        output.push_str(&html_block_to_text(&input[start..end]));
        last = end;
    }
    output.push_str(&input[last..]);
    output
}

/// Recursively collect the byte ranges of every raw HTML node in a markdown AST
fn collect_html_ranges(node: &Node, ranges: &mut Vec<(usize, usize)>) {
    if let Node::Html(html) = node {
        if let Some(position) = &html.position {
            ranges.push((position.start.offset, position.end.offset));
        }
        return;
    }
    if let Some(children) = node.children() {
        for child in children {
            collect_html_ranges(child, ranges);
        }
    }
}

/// Convert a raw HTML block to plain text, keeping heading and paragraph
/// text content while dropping any other markup
fn html_block_to_text(html: &str) -> String {
    use std::fmt::Write as _;

    let block_re = Regex::new(r"(?is)<h([1-6])[^>]*>(.*?)</h[1-6]\s*>|<p[^>]*>(.*?)</p>")
        .expect("valid regex");

    let mut output = String::new();
    for cap in block_re.captures_iter(html) {
        if let Some(level) = cap.get(1) {
            let level = level.as_str().parse::<usize>().unwrap_or(1).min(3);
            let text = strip_inline_tags(cap.get(2).map_or("", |m| m.as_str()));
            if !text.is_empty() {
                let _ = writeln!(output, "[h{level}]{text}[/h{level}]");
            }
        } else if let Some(paragraph) = cap.get(3) {
            let text = strip_inline_tags(paragraph.as_str());
            if !text.is_empty() {
                output.push_str(&text);
                output.push('\n');
            }
        }
    }
    output
}

/// Strip HTML tags from a string, keeping the text content, and converting
/// a handful of common inline formatting tags to their bbcode equivalent
fn strip_inline_tags(input: &str) -> String {
    const BR_PLACEHOLDER: &str = "\u{0}BR\u{0}";

    let bold_open = Regex::new(r"(?i)<(?:strong|b)(?:\s[^>]*)?>").expect("valid regex");
    let bold_close = Regex::new(r"(?i)</(?:strong|b)\s*>").expect("valid regex");
    let italic_open = Regex::new(r"(?i)<(?:em|i)(?:\s[^>]*)?>").expect("valid regex");
    let italic_close = Regex::new(r"(?i)</(?:em|i)\s*>").expect("valid regex");
    let br = Regex::new(r"(?i)<br\s*/?>").expect("valid regex");
    let tag = Regex::new(r"(?is)<[^>]+>").expect("valid regex");
    let whitespace = Regex::new(r"\s+").expect("valid regex");

    let text = bold_open.replace_all(input, "[b]");
    let text = bold_close.replace_all(&text, "[/b]");
    let text = italic_open.replace_all(&text, "[i]");
    let text = italic_close.replace_all(&text, "[/i]");
    let text = br.replace_all(&text, BR_PLACEHOLDER);
    let text = tag.replace_all(&text, "");
    let text = whitespace.replace_all(&text, " ");
    text.replace(BR_PLACEHOLDER, "\n").trim().to_string()
}

pub fn markdown_headings_to_steam(input: &str) -> String {
    input
        .lines()
        .map(|line| {
            let hashes = line.chars().take_while(|&c| c == '#').count();

            if (1..=usize::MAX).contains(&hashes) && line.chars().nth(hashes) == Some(' ') {
                let level = hashes.min(3);
                format!("[h{level}]{}[/h{level}]", &line[hashes + 1..])
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}
