//! Every lint and preprocessor code must be exercised by at least one test.
//!
//! This scans the source tree for codes and the test sources and fixtures
//! for references to them, so a new lint can't be added without a test.
//! It does not run anything, it only checks that a test exists.

use std::path::{Path, PathBuf};

/// Codes that can't be tested portably, with the reason
const EXCEPTIONS: &[(&str, &str)] = &[];

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace root")
        .to_path_buf()
}

/// Lowercase codes such as `s13` or `pe12`, from file and folder names like `s13_undefined.rs`
fn codes_in(dir: &Path, prefixes: &[&str]) -> Vec<String> {
    let mut codes: Vec<String> = fs_err::read_dir(dir)
        .expect("read lint folder")
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let name = entry.file_name().to_string_lossy().to_lowercase();
            let (code, _) = name.split_once('_')?;
            let prefix = prefixes.iter().find(|p| code.starts_with(*p))?;
            code[prefix.len()..]
                .chars()
                .all(|c| c.is_ascii_digit())
                .then(|| code.to_string())
        })
        .collect();
    codes.sort();
    codes.dedup();
    assert!(!codes.is_empty(), "no codes found in {}", dir.display());
    codes
}

/// The names of everything under the test folders, and the text of the test sources
fn test_corpus() -> String {
    let mut corpus = String::new();
    for dir in ["bin/tests", "hls/tests", "libs"] {
        for entry in walkdir::WalkDir::new(root().join(dir))
            .into_iter()
            .filter_map(Result::ok)
        {
            let path = entry.path();
            let in_tests = path
                .components()
                .any(|c| c.as_os_str() == "tests" || c.as_os_str() == "snapshots");
            if !in_tests {
                continue;
            }
            corpus.push_str(&path.to_string_lossy().to_lowercase());
            corpus.push('\n');
            if path.extension().is_some_and(|e| e == "rs") {
                corpus.push_str(
                    &fs_err::read_to_string(path)
                        .expect("read test source")
                        .to_lowercase(),
                );
                corpus.push('\n');
            }
        }
    }
    corpus
}

/// Is the code referenced, as `s13_...` (fixture names) or `l-s13` (diagnostic output)
fn is_tested(corpus: &str, code: &str) -> bool {
    [format!("{code}_"), format!("l-{code}\"")]
        .iter()
        .chain(std::iter::once(&format!("[l-{code}]")))
        .any(|needle| {
            corpus.match_indices(needle.as_str()).any(|(i, _)| {
                // `s1_` must not match inside `s21_`
                i == 0
                    || !corpus.as_bytes()[i - 1].is_ascii_alphanumeric()
                    || needle.starts_with('[')
            })
        })
}

#[test]
fn every_code_has_a_test() {
    let corpus = test_corpus();
    let sources: &[(&str, &[&str])] = &[
        ("libs/sqf/src/analyze/lints", &["s"]),
        ("libs/config/src/analyze/lints", &["c"]),
        ("libs/stringtable/src/analyze/lints", &["l"]),
        ("libs/preprocessor/src/codes", &["pe", "pw"]),
    ];
    let mut untested = Vec::new();
    let mut found = 0;
    for (dir, prefixes) in sources {
        for code in codes_in(&root().join(dir), prefixes) {
            found += 1;
            if EXCEPTIONS.iter().any(|(c, _)| *c == code) {
                continue;
            }
            if !is_tested(&corpus, &code) {
                untested.push(format!("{code} (from {dir})"));
            }
        }
    }
    assert!(found > 100, "only found {found} codes, is the scan broken?");
    assert!(
        untested.is_empty(),
        "these codes have no test, add a fixture for each:\n  {}",
        untested.join("\n  ")
    );
}

#[test]
fn exceptions_are_still_needed() {
    let corpus = test_corpus();
    for (code, reason) in EXCEPTIONS {
        assert!(
            !is_tested(&corpus, code),
            "`{code}` now has a test, remove it from EXCEPTIONS ({reason})"
        );
    }
}
