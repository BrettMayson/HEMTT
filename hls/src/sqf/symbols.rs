use tower_lsp::lsp_types::{DocumentSymbol, Position, Range, SymbolKind};

use crate::files::FileCache;

use super::SqfAnalyzer;

impl SqfAnalyzer {
    /// An outline of the private variables declared in a `.sqf` file, via
    /// `private _x = ...;`, `private ["_a", "_b"]`, or `params [...]`.
    #[allow(clippy::unused_self)]
    pub fn document_symbols(&self, url: &url::Url) -> Option<Vec<DocumentSymbol>> {
        if !std::path::Path::new(url.path())
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("sqf"))
        {
            return None;
        }
        let text = FileCache::get().text(url)?;
        let symbols = collect_private_variables(&text);
        if symbols.is_empty() {
            None
        } else {
            Some(symbols)
        }
    }
}

#[allow(deprecated)] // `DocumentSymbol::deprecated` has no builder, must be named
#[allow(clippy::cast_possible_truncation)]
fn variable_symbol(name: &str, line: u32, start: usize, end: usize) -> DocumentSymbol {
    let range = Range::new(
        Position::new(line, start as u32),
        Position::new(line, end as u32),
    );
    DocumentSymbol {
        name: name.to_string(),
        detail: None,
        kind: SymbolKind::VARIABLE,
        tags: None,
        deprecated: None,
        range,
        selection_range: range,
        children: None,
    }
}

fn collect_private_variables(text: &str) -> Vec<DocumentSymbol> {
    #[allow(clippy::cast_possible_truncation)]
    const fn line_number(line_idx: usize) -> u32 {
        line_idx as u32
    }

    let private_re = regex::Regex::new(r"\bprivate\s+(_[A-Za-z0-9_]+)").expect("valid regex");
    let array_decl_re =
        regex::Regex::new(r"\b(?:private|params)\s*\[([^\]]*)\]").expect("valid regex");
    let string_item_re = regex::Regex::new(r#""(_[A-Za-z0-9_]+)""#).expect("valid regex");

    let mut symbols = Vec::new();
    for (line_idx, line) in text.lines().enumerate() {
        let line_idx = line_number(line_idx);
        for caps in private_re.captures_iter(line) {
            let m = caps.get(1).expect("group 1 always matches");
            symbols.push(variable_symbol(m.as_str(), line_idx, m.start(), m.end()));
        }
        for caps in array_decl_re.captures_iter(line) {
            let inner = caps.get(1).expect("group 1 always matches");
            for item in string_item_re.captures_iter(inner.as_str()) {
                let m = item.get(1).expect("group 1 always matches");
                symbols.push(variable_symbol(
                    m.as_str(),
                    line_idx,
                    inner.start() + m.start(),
                    inner.start() + m.end(),
                ));
            }
        }
    }
    symbols
}

#[cfg(test)]
mod tests {
    use tower_lsp::lsp_types::DocumentSymbol;

    use super::collect_private_variables;

    #[test]
    fn finds_private_assignment() {
        let text = "private _unit = objNull;";
        let symbols = collect_private_variables(text);
        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].name, "_unit");
    }

    #[test]
    fn finds_private_and_params_arrays() {
        let text = "params [\"_a\", \"_b\"];\nprivate [\"_c\"];";
        let names: Vec<_> = collect_private_variables(text)
            .into_iter()
            .map(|s| s.name)
            .collect();
        assert_eq!(names, vec!["_a", "_b", "_c"]);
    }

    #[test]
    fn empty_file_has_no_symbols() {
        let empty: [DocumentSymbol; 0] = [];
        assert_eq!(collect_private_variables(""), empty);
    }
}
