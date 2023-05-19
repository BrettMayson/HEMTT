use std::time::SystemTime;

use rhai::EvalAltResult;
use time::{format_description, OffsetDateTime};

pub fn date(format: &str) -> Result<String, Box<EvalAltResult>> {
    let now: OffsetDateTime = SystemTime::now().into();
    let fmt = format_description::parse(format).map_err(|e| e.to_string())?;
    Ok(now.format(&fmt).map_err(|e| e.to_string())?)
}
