use markdown::{Constructs, ParseOptions, mdast::Node};
use markdown_bbcode::MdToBbcode;
use regex::Regex;

pub(super) fn description(input: &str) -> Result<String, String> {
    let input = markdown_headings_to_steam(&strip_html(input));
    let mut output = Vec::new();
    MdToBbcode::new(&input, &mut output)
        .serialize()
        .map_err(|error| format!("Could not convert description to Workshop BBCode: {error:?}"))?;
    String::from_utf8(output).map_err(|error| error.to_string())
}

pub(super) fn change_note(input: &str, version: &str) -> Result<String, String> {
    let changelog =
        parse_changelog::parse(input).map_err(|error| format!("Invalid changelog: {error}"))?;
    let entry = changelog
        .get(version)
        .ok_or_else(|| format!("No changelog entry found for version {version}"))?;
    Ok(markdown_headings_to_steam(&strip_html(&format!(
        "# Version {version}\n\n{}",
        entry.notes
    ))))
}

/// Replace raw HTML blocks in markdown source with Steam-friendly text.
///
/// Steam Workshop does not render HTML,
/// so leaving it as-is would show the raw tags as garbage text on the item page.
/// Headings and paragraphs usually carry useful information (e.g. a title or requirements notice),
/// so their text content is kept and converted,
/// while purely decorative markup (badges, images, links used only for styling, etc.) is dropped.
pub(super) fn strip_html(input: &str) -> String {
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

/// Convert a raw HTML block to plain text, keeping heading and paragraph text content while dropping any other markup
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

/// Strip HTML tags from a string, keeping the text content, and converting a handful of common inline formatting tags to their bbcode equivalent
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
