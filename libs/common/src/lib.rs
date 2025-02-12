//! HEMTT - Common Library

pub mod arma;
pub mod config;
pub mod error;
pub mod io;
pub mod math;
pub mod prefix;
pub mod steam;
pub mod strip;
pub mod version;

pub use error::Error;

mod sign_version;
pub use sign_version::BISignVersion;

#[must_use]
/// Returns up to 3 similar values from a haystack.
pub fn similar_values<'a>(search: &str, haystack: &'a [&str]) -> Vec<&'a str> {
    let lower_case_haystack = haystack
        .iter()
        .map(|v| (*v, v.to_lowercase()))
        .collect::<Vec<_>>();
    let mut similar = lower_case_haystack
        .iter()
        .map(|(v, vl)| (v, strsim::levenshtein(vl.as_str(), &search.to_lowercase())))
        .collect::<Vec<_>>();
    similar.sort_by_key(|(_, v)| *v);
    similar.retain(|s| s.1 <= 3);
    similar.truncate(3);
    similar.into_iter().map(|(n, _)| *n).collect::<Vec<_>>()
}
