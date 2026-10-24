use tower_lsp::lsp_types::FoldingRange;

use crate::files::FileCache;

use super::SqfAnalyzer;

impl SqfAnalyzer {
    /// Fold each multi-line `{...}`/`[...]` block.
    #[allow(clippy::unused_self)]
    pub fn folding_ranges(&self, url: &url::Url) -> Option<Vec<FoldingRange>> {
        if !std::path::Path::new(url.path())
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("sqf"))
        {
            return None;
        }
        let text = FileCache::get().text(url)?;
        let ranges = collect_brace_folds(&text);
        if ranges.is_empty() {
            None
        } else {
            Some(ranges)
        }
    }
}

fn collect_brace_folds(text: &str) -> Vec<FoldingRange> {
    #[allow(clippy::cast_possible_truncation)]
    const fn line_number(line_idx: usize) -> u32 {
        line_idx as u32
    }

    let mut stack: Vec<(char, u32)> = Vec::new();
    let mut ranges = Vec::new();
    for (line_idx, line) in text.lines().enumerate() {
        let line_idx = line_number(line_idx);
        for c in line.chars() {
            match c {
                '{' | '[' => stack.push((c, line_idx)),
                '}' | ']' => {
                    if let Some((open_char, open_line)) = stack.pop() {
                        let matches = (open_char == '{' && c == '}') || (open_char == '[' && c == ']');
                        if matches && line_idx > open_line {
                            ranges.push(FoldingRange {
                                start_line: open_line,
                                end_line: line_idx,
                                ..Default::default()
                            });
                        }
                    }
                }
                _ => {}
            }
        }
    }
    ranges
}

#[cfg(test)]
mod tests {
    use tower_lsp::lsp_types::FoldingRange;

    use super::collect_brace_folds;

    #[test]
    fn folds_multiline_block() {
        let text = "if (true) then {\n\thint \"a\";\n};";
        let ranges = collect_brace_folds(text);
        assert_eq!(ranges.len(), 1);
        assert_eq!(ranges[0].start_line, 0);
        assert_eq!(ranges[0].end_line, 2);
    }

    #[test]
    fn ignores_single_line_blocks() {
        let text = "private _x = {1};";
        let empty: [FoldingRange; 0] = [];
        assert_eq!(collect_brace_folds(text), empty);
    }

    #[test]
    fn folds_nested_blocks() {
        let text = "{\n\tif (true) then {\n\t\thint \"a\";\n\t};\n};";
        let ranges = collect_brace_folds(text);
        assert_eq!(ranges.len(), 2);
    }
}
