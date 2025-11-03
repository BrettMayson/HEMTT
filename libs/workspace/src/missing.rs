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
    let mut prefix = String::new();
    for char in target.chars() {
        if char == '/' || char == '\\' {
            if prefix.is_empty() {
                continue;
            }
            break;
        }
        prefix.push(char);
    }
    if let Some(mainprefix) = &project.mainprefix()
        && prefix != mainprefix.to_lowercase()
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
