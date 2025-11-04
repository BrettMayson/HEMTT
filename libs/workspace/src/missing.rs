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
    let workspace = processed
        .sources()
        .first()
        .map(|s| s.0.clone())
        .expect("no sources");
    let expected_path = project.expected_path();
    let target_lower = target.to_ascii_lowercase();
    if !target_lower.starts_with(expected_path)
        || !target_lower.starts_with(&format!(r"\{expected_path}"))
    {
        return false;
    }

    if matches!(workspace.locate(target), Ok(Some(_))) {
        return false;
    }
    if !(target.starts_with('/') || target.starts_with('\\'))
        && matches!(workspace.locate(&format!("/{target}")), Ok(Some(_)))
    {
        return false;
    }
    true
}
