use hemtt_common::config::ProjectConfig;

use crate::reporting::Processed;

#[must_use]
/// Check if a target file is missing in the workspace
///
/// # Panics
/// Panics if there are no sources in the processed data
pub fn check_is_missing_file(target: &str, project: &ProjectConfig, processed: &Processed) -> bool {
    const ILLEGAL_CHARACTERS: &[char] = &['*', '?', '"', '<', '>', '|', ':', '%', ' '];
    if !target.contains('.') {
        return false;
    }
    if target.chars().any(|c| ILLEGAL_CHARACTERS.contains(&c)) {
        return false;
    }
    let expected_paths = project.expected_paths();
    let target_lower = target.to_ascii_lowercase();
    if !(expected_paths.iter().any(|p| target_lower.starts_with(p))) {
        return false;
    }
    if target.contains('/') {
        // always block targets containing '/' in path
        return true;
    }
    let workspace = processed
        .sources()
        .first()
        .map(|s| s.0.clone())
        .expect("no sources");
    if matches!(workspace.locate(target), Ok(Some(_))) {
        return false;
    }
    if !target.starts_with('\\') && matches!(workspace.locate(&format!(r"\{target}")), Ok(Some(_)))
    {
        return false;
    }
    true
}
