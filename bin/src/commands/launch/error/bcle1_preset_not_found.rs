use std::{ops::Range, path::Path, sync::Arc};

use ariadne::{ColorGenerator, Fmt, Label, Report, ReportKind, Source};
use hemtt_common::{
    reporting::{simple, Code},
    similar_values,
};

use crate::Error;

pub struct PresetNotFound {
    name: String,
    similar: Vec<String>,
    position: Option<Range<usize>>,
}

impl Code for PresetNotFound {
    fn ident(&self) -> &'static str {
        "BCLE1"
    }

    fn message(&self) -> String {
        format!("Preset `{}` not found.", self.name)
    }

    fn help(&self) -> Option<String> {
        if self.similar.is_empty() {
            None
        } else {
            Some(format!("Did you mean `{}`?", self.similar.join("`, `")))
        }
    }

    fn report(&self) -> Option<String> {
        if let Some(position) = &self.position {
            let color = ColorGenerator::default().next();
            let mut out = Vec::new();
            let mut report =
                Report::build(ReportKind::Error, ".hemtt/project.toml", position.start)
                    .with_code(self.ident())
                    .with_message(self.message())
                    .with_label(
                        Label::new((".hemtt/project.toml", position.clone()))
                            .with_color(color)
                            .with_message(format!(
                                "Preset `{}` not found.",
                                (&self.name).fg(color)
                            )),
                    );
            if let Some(help) = self.help() {
                report = report.with_help(help);
            }
            report
                .finish()
                .write_for_stdout(
                    (
                        ".hemtt/project.toml",
                        Source::from(
                            std::fs::read_to_string(".hemtt/project.toml")
                                .expect("can not find position if file is not readable"),
                        ),
                    ),
                    &mut out,
                )
                .unwrap();
            Some(String::from_utf8(out).unwrap_or_default())
        } else {
            Some(simple(self, ariadne::ReportKind::Error, self.help()))
        }
    }

    fn ci(&self) -> Vec<hemtt_common::reporting::Annotation> {
        vec![]
    }
}

impl PresetNotFound {
    pub fn code(launch: &str, name: String, path: &Path) -> Result<Arc<dyn Code>, Error> {
        let presets = if let Ok(files) = path.read_dir() {
            files
                .filter_map(|x| {
                    x.ok().and_then(|x| {
                        if x.file_type().ok()?.is_file() {
                            x.file_name()
                                .to_str()
                                .map(|s| s.trim_end_matches(".html").to_string())
                        } else {
                            None
                        }
                    })
                })
                .collect::<Vec<String>>()
        } else {
            vec![]
        };

        let position = if let Ok(content) = std::fs::read_to_string(".hemtt/project.toml") {
            attempt_locate(&content, launch, &name)
        } else {
            None
        };

        Ok(Arc::new(Self {
            similar: similar_values(
                &name,
                &presets
                    .iter()
                    .map(std::string::String::as_str)
                    .collect::<Vec<&str>>(),
            )
            .iter()
            .map(std::string::ToString::to_string)
            .collect(),
            name,
            position,
        }))
    }
}

fn attempt_locate(content: &str, launch: &str, preset: &str) -> Option<Range<usize>> {
    let header = format!("[hemtt.launch.{}]", launch);
    let preset_line = format!("\"{}\"", preset);
    let mut in_section = false;
    let mut in_presets = false;
    let mut offset = 0;
    for line in content.lines() {
        if line.starts_with(&header) {
            in_section = true;
        } else if in_section && line.starts_with('[') {
            in_section = false;
            in_presets = false;
        } else if in_section && line.starts_with("presets") {
            in_presets = true;
        } else if in_presets && line.contains(&preset_line) {
            let start = offset + line.find(&preset_line).unwrap();
            let end = start + preset_line.len();
            return Some(start + 1..end - 1);
        }
        offset += line.len() + 1;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_attempt_locate() {
        let content = r#"
[hemtt.launch.server]
presets = [
    "test",
    "test2",
    "test3",
]
"#;
        assert_eq!(
            attempt_locate(content, "server", "test"),
            Some(39..45),
            "test"
        );
        assert_eq!(
            attempt_locate(content, "server", "test2"),
            Some(51..58),
            "test2"
        );
        assert_eq!(
            attempt_locate(content, "server", "test3"),
            Some(64..71),
            "test3"
        );
        assert_eq!(attempt_locate(content, "server", "test4"), None, "test4");
    }
}
