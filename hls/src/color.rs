use regex::Regex;
use tower_lsp::lsp_types::Color;
use tower_lsp::lsp_types::{
    ColorInformation, ColorPresentation, ColorPresentationParams, Position, Range,
};
use tracing::debug;
use url::Url;

use crate::files::FileCache;

pub fn info(url: &Url) -> Vec<ColorInformation> {
    let regex = Regex::new(r"(?m)\#\((\w+),(\d+),(\d+),(\d+)\)color\(([\d|\.]+),([\d|\.]+),([\d|\.]+),([\d|\.]+)(?:,(\w+))?\)").expect("Failed to compile color regex");

    let Some(text) = FileCache::get().text(url) else {
        return vec![];
    };

    let mut colors = Vec::new();
    for mat in regex.captures_iter(&text) {
        if !["rgb", "argb"].contains(&mat.get(1).expect("Failed to get color type").as_str()) {
            continue;
        }
        debug!("Found color: {:?}", mat);
        let start =
            offset_to_line_and_char(&text, mat.get(0).expect("Failed to get match").start());
        let end = offset_to_line_and_char(&text, mat.get(0).expect("Failed to get match").end());
        colors.push(ColorInformation {
            range: Range {
                start: Position {
                    line: start.0,
                    character: start.1,
                },
                end: Position {
                    line: end.0,
                    character: end.1,
                },
            },
            color: Color {
                red: mat
                    .get(5)
                    .expect("Failed to get red component")
                    .as_str()
                    .parse()
                    .expect("Failed to parse red component"),
                green: mat
                    .get(6)
                    .expect("Failed to get green component")
                    .as_str()
                    .parse()
                    .expect("Failed to parse green component"),
                blue: mat
                    .get(7)
                    .expect("Failed to get blue component")
                    .as_str()
                    .parse()
                    .expect("Failed to parse blue component"),
                alpha: mat
                    .get(8)
                    .expect("Failed to get alpha component")
                    .as_str()
                    .parse()
                    .expect("Failed to parse alpha component"),
            },
        });
    }

    colors
}

pub fn presentation(params: &ColorPresentationParams) -> Vec<ColorPresentation> {
    let regex = Regex::new(r"(?m)\#\((\w+),(\d+),(\d+),(\d+)\)color\(([\d|\.]+),([\d|\.]+),([\d|\.]+),([\d|\.]+)(?:,(\w+))?\)").expect("Failed to compile color regex");

    let Some(text) = FileCache::get().text(&params.text_document.uri) else {
        return vec![];
    };

    for mat in regex.captures_iter(&text) {
        let start =
            offset_to_line_and_char(&text, mat.get(0).expect("Failed to get match").start());
        let end = offset_to_line_and_char(&text, mat.get(0).expect("Failed to get match").end());
        let range = Range {
            start: Position {
                line: start.0,
                character: start.1,
            },
            end: Position {
                line: end.0,
                character: end.1,
            },
        };
        if range == params.range {
            return vec![ColorPresentation {
                label: mat.get(9).map_or_else(
                    || {
                        format!(
                            "#({},{},{},{})color({},{},{},{})",
                            mat.get(1).expect("Failed to get color type").as_str(),
                            mat.get(2).expect("Failed to get first component").as_str(),
                            mat.get(3).expect("Failed to get second component").as_str(),
                            mat.get(4).expect("Failed to get third component").as_str(),
                            params.color.red,
                            params.color.green,
                            params.color.blue,
                            params.color.alpha
                        )
                    },
                    |texture_type| {
                        format!(
                            "#({},{},{},{})color({},{},{},{},{})",
                            mat.get(1).expect("Failed to get color type").as_str(),
                            mat.get(2).expect("Failed to get first component").as_str(),
                            mat.get(3).expect("Failed to get second component").as_str(),
                            mat.get(4).expect("Failed to get third component").as_str(),
                            params.color.red,
                            params.color.green,
                            params.color.blue,
                            params.color.alpha,
                            texture_type.as_str()
                        )
                    },
                ),
                text_edit: None,
                additional_text_edits: None,
            }];
        }
    }

    vec![]
}

fn offset_to_line_and_char(text: &str, offset: usize) -> (u32, u32) {
    let mut line: u32 = 0;
    let mut col: u32 = 0;
    for (i, c) in text.chars().enumerate() {
        if i == offset {
            break;
        }
        if c == '\n' {
            line += 1;
            col = 0;
        } else {
            col += 1;
        }
    }
    (line, col)
}
