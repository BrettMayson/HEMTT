use regex::Regex;
use tower_lsp::lsp_types::{
    ColorInformation, ColorPresentation, ColorPresentationParams, Position, Range,
};
use tower_lsp::{jsonrpc::Result, lsp_types::Color};
use tracing::debug;
use url::Url;

use crate::files::FileCache;

pub async fn info(url: Url) -> Result<Vec<ColorInformation>> {
    let regex = Regex::new(r"(?m)\#\((\w+),(\d+),(\d+),(\d+)\)color\(([\d|\.]+),([\d|\.]+),([\d|\.]+),([\d|\.]+)(?:,(\w+))?\)").unwrap();

    let Some(text) = FileCache::get().text(&url) else {
        return Ok(vec![]);
    };

    let mut colors = Vec::new();
    for mat in regex.captures_iter(&text) {
        if !["rgb", "argb"].contains(&mat.get(1).unwrap().as_str()) {
            continue;
        }
        debug!("Found color: {:?}", mat);
        let start = offset_to_line_and_char(&text, mat.get(0).unwrap().start());
        let end = offset_to_line_and_char(&text, mat.get(0).unwrap().end());
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
                red: mat.get(5).unwrap().as_str().parse().unwrap(),
                green: mat.get(6).unwrap().as_str().parse().unwrap(),
                blue: mat.get(7).unwrap().as_str().parse().unwrap(),
                alpha: mat.get(8).unwrap().as_str().parse().unwrap(),
            },
        });
    }

    Ok(colors)
}

pub async fn presentation(params: ColorPresentationParams) -> Result<Vec<ColorPresentation>> {
    let regex = Regex::new(r"(?m)\#\((\w+),(\d+),(\d+),(\d+)\)color\(([\d|\.]+),([\d|\.]+),([\d|\.]+),([\d|\.]+)(?:,(\w+))?\)").unwrap();

    let Some(text) = FileCache::get().text(&params.text_document.uri) else {
        return Ok(vec![]);
    };

    for mat in regex.captures_iter(&text) {
        let start = offset_to_line_and_char(&text, mat.get(0).unwrap().start());
        let end = offset_to_line_and_char(&text, mat.get(0).unwrap().end());
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
            return Ok(vec![ColorPresentation {
                label: if let Some(texture_type) = mat.get(9) {
                    format!(
                        "#({},{},{},{})color({},{},{},{},{})",
                        mat.get(1).unwrap().as_str(),
                        mat.get(2).unwrap().as_str(),
                        mat.get(3).unwrap().as_str(),
                        mat.get(4).unwrap().as_str(),
                        params.color.red,
                        params.color.green,
                        params.color.blue,
                        params.color.alpha,
                        texture_type.as_str()
                    )
                } else {
                    format!(
                        "#({},{},{},{})color({},{},{},{})",
                        mat.get(1).unwrap().as_str(),
                        mat.get(2).unwrap().as_str(),
                        mat.get(3).unwrap().as_str(),
                        mat.get(4).unwrap().as_str(),
                        params.color.red,
                        params.color.green,
                        params.color.blue,
                        params.color.alpha
                    )
                },
                text_edit: None,
                additional_text_edits: None,
            }]);
        }
    }

    Ok(vec![])
}

fn offset_to_line_and_char(text: &str, offset: usize) -> (u32, u32) {
    let mut line = 0;
    let mut col = 0;
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
    (line as u32, col as u32)
}
