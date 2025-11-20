use std::time::SystemTime;

use rhai::EvalAltResult;
use time::{OffsetDateTime, format_description};

/// Rhai function to get the current date/time formatted according to the given format string.
///
/// # Errors
/// If the format string is invalid or if formatting fails.
pub fn date(format: &str) -> Result<String, Box<EvalAltResult>> {
    let now: OffsetDateTime = SystemTime::now().into();
    let fmt = format_description::parse(format).map_err(|e| e.to_string())?;
    Ok(now.format(&fmt).map_err(|e| e.to_string())?)
}
