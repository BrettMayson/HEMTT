use std::{ops::Range, path::Path, sync::Arc};

use hemtt_common::{
    reporting::{Code, Diagnostic, Label},
    similar_values,
    workspace::{LayerType, Workspace, WorkspacePath},
};

pub struct MissionNotFound {
    project_toml: WorkspacePath,
    name: String,
    similar: Vec<String>,
    position: Option<Range<usize>>,
}

impl Code for MissionNotFound {
    fn ident(&self) -> &'static str {
        "BCLE8"
    }

    fn message(&self) -> String {
        format!("Mission `{}` not found.", self.name)
    }

    fn help(&self) -> Option<String> {
        if self.similar.is_empty() {
            None
        } else {
            Some(format!("did you mean `{}`?", self.similar.join("`, `")))
        }
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        Some({
            let mut diag = Diagnostic::simple(self);
            if let Some(position) = &self.position {
                diag = diag.with_label(
                    Label::primary(self.project_toml.clone(), position.clone())
                        .with_message("mission not found"),
                );
            }
            diag
        })
    }
}

impl MissionNotFound {
    pub fn code(launch: &str, name: String, path: &Path) -> Arc<dyn Code> {
        let presets = path.read_dir().map_or_else(
            |_| vec![],
            |files| {
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
            },
        );

        let position = std::fs::read_to_string(".hemtt/project.toml")
            .map_or(None, |content| attempt_locate(&content, launch, &name));

        Arc::new(Self {
            project_toml: {
                Workspace::builder()
                    .physical(
                        &std::env::current_dir().expect("to be in a folder"),
                        LayerType::Source,
                    )
                    .finish(None)
                    .expect("can create workspace")
                    .join(".hemtt")
                    .expect("project.toml must exist to get here")
                    .join("project.toml")
                    .expect("project.toml must exist to get here")
            },
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
        })
    }
}

fn attempt_locate(content: &str, launch: &str, mission: &str) -> Option<Range<usize>> {
    let header = format!("[hemtt.launch.{launch}]");
    let preset_line = format!("\"{mission}\"");
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
"#
        .replace("\r\n", "\n");
        assert_eq!(
            attempt_locate(&content, "server", "test"),
            Some(40..44),
            "test"
        );
        assert_eq!(
            attempt_locate(&content, "server", "test2"),
            Some(52..57),
            "test2"
        );
        assert_eq!(
            attempt_locate(&content, "server", "test3"),
            Some(65..70),
            "test3"
        );
        assert_eq!(attempt_locate(&content, "server", "test4"), None, "test4");
    }
}
