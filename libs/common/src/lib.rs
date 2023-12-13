//! HEMTT - Common Library

pub mod addons;
pub mod arma;
pub mod error;
pub mod io;
pub mod math;
pub mod position;
pub mod prefix;
pub mod project;
pub mod reporting;
pub mod version;
pub mod workspace;

mod sign_version;
pub use sign_version::BISignVersion;

#[must_use]
pub fn similar_values<'a>(search: &str, haystack: &'a [&str]) -> Vec<&'a str> {
    let mut similar = haystack
        .iter()
        .map(|v| (v, strsim::levenshtein(v, search)))
        .collect::<Vec<_>>();
    similar.sort_by_key(|(_, v)| *v);
    similar.retain(|s| s.1 <= 3);
    if similar.len() > 3 {
        similar.truncate(3);
    }
    similar.into_iter().map(|(n, _)| *n).collect::<Vec<_>>()
}
