use std::{collections::HashMap, io::BufReader};

use hemtt_workspace::{
    position::{LineCol, Position},
    WorkspacePath,
};
use quick_xml::se::Serializer;
use serde::{Deserialize, Serialize};

pub mod analyze;
mod key;
mod package;
pub mod rapify;
mod totals;

pub use key::Key;
pub use package::Package;
pub use totals::Totals;

/// Languages in className format
static ALL_LANGUAGES: [&str; 25] = [
    "English",
    "Czech",
    "French",
    "Spanish",
    "Italian",
    "Polish",
    "Portuguese",
    "Russian",
    "German",
    "Korean",
    "Japanese",
    "Chinese",
    "Chinesesimp",
    "Turkish",
    "Swedish",
    "Slovak",
    "SerboCroatian",
    "Norwegian",
    "Icelandic",
    "Hungarian",
    "Greek",
    "Finnish",
    "Dutch",
    "Ukrainian",
    "Danish",
];

#[derive(Clone, Debug)]
pub struct Project {
    inner: InnerProject,
    path: WorkspacePath,
    keys: HashMap<String, Vec<Position>>,
    source: String,
    comments: Vec<(String, String, Option<String>)>,
}

impl Project {
    #[must_use]
    pub fn name(&self) -> &str {
        &self.inner.name
    }

    #[must_use]
    pub fn packages(&self) -> &[Package] {
        &self.inner.packages
    }

    #[must_use]
    pub const fn keys(&self) -> &HashMap<String, Vec<Position>> {
        &self.keys
    }

    #[must_use]
    pub const fn path(&self) -> &WorkspacePath {
        &self.path
    }

    #[must_use]
    pub fn source(&self) -> &str {
        &self.source
    }

    pub fn sort(&mut self) {
        self.inner.packages.sort_by(|a, b| a.name().cmp(b.name()));
        for package in &mut self.inner.packages {
            package.sort();
        }
    }

    /// Read a Project
    ///
    /// # Errors
    /// [`quick_xml::DeError`] if the reader is not a valid stringtable
    /// # Panics
    pub fn read(path: WorkspacePath) -> Result<Self, quick_xml::de::DeError> {
        let mut buffer = String::new();
        let mut reading_comments = false;
        let mut comments = Vec::new();
        let mut in_key = None;
        let source = path.read_to_string().expect("Failed to read file"); // todo proper error return
        let reader = source
            .lines()
            .map(|l| {
                let l_trim = l.trim();
                if reading_comments {
                    buffer.push('\n');
                }
                if l_trim.starts_with("<!--") {
                    reading_comments = true;
                    if !buffer.is_empty() {
                        buffer.push('\n');
                    }
                }
                if !reading_comments && !buffer.is_empty() {
                    comments.push((
                        buffer.trim().to_string(),
                        l_trim.to_string(),
                        in_key.clone(),
                    ));
                    buffer.clear();
                }
                if reading_comments {
                    buffer.push_str(l);
                    if l_trim.ends_with("-->") {
                        reading_comments = false;
                    }
                }
                if !reading_comments {
                    if l_trim.starts_with("<Key") {
                        in_key = Some(l_trim.to_string());
                    } else if l_trim.starts_with("</Key>") {
                        in_key = None;
                    }
                }
                l.replace('&', "&amp;")
            })
            .collect::<Vec<_>>();
        comments.sort();
        comments.dedup();
        let inner: InnerProject =
            quick_xml::de::from_reader(BufReader::new(reader.join("\n").as_bytes()))?;
        Ok(Self {
            keys: process_keys(&inner, &source, &path),
            inner,
            path,
            source,
            comments,
        })
    }

    /// Write a Project to a writer
    ///
    /// # Errors
    /// [`quick_xml::SeError`] if the writer fails to write
    pub fn to_writer<W: std::fmt::Write>(&self, writer: &mut W) -> Result<(), quick_xml::SeError> {
        writer.write_str(r#"<?xml version="1.0" encoding="utf-8"?>"#)?;
        writer.write_char('\n')?;
        let mut buffer = String::new();
        let mut ser = Serializer::new(&mut buffer);
        ser.indent(' ', 4);
        ser.expand_empty_elements(true);
        self.inner.serialize(ser)?;
        buffer.push('\n');

        let mut clear_next = false;
        let mut in_key = None;
        for line in buffer.lines() {
            let l_trim = line.trim();
            if clear_next {
                clear_next = false;
                in_key = None;
            }
            for (before, after, key) in &self.comments {
                if l_trim.starts_with(after) && &in_key == key {
                    let mut whitespace = line
                        .chars()
                        .take_while(|c| c.is_whitespace())
                        .collect::<String>();
                    if l_trim.starts_with("</") {
                        whitespace += "    ";
                    }
                    writer.write_str(&whitespace)?;
                    writer.write_str(before)?;
                    writer.write_char('\n')?;
                }
            }
            writer.write_str(line.replace("&amp;amp;", "&").as_str())?;
            writer.write_char('\n')?;
            if l_trim.starts_with("<Key") {
                in_key = Some(l_trim.to_string());
            } else if l_trim.starts_with("</Key>") {
                clear_next = true;
            }
        }

        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename = "Project")]
struct InnerProject {
    #[serde(rename = "@name")]
    name: String,
    #[serde(rename = "Package")]
    packages: Vec<Package>,
}

fn process_keys(
    inner: &InnerProject,
    source: &str,
    path: &WorkspacePath,
) -> HashMap<String, Vec<Position>> {
    let mut keys = HashMap::new();
    let mut all_keys: Vec<String> = Vec::with_capacity(20);
    for package in &inner.packages {
        for package_inner in package.containers() {
            for key in package_inner.keys() {
                all_keys.push(key.id().to_string());
            }
        }
        for key in package.keys() {
            all_keys.push(key.id().to_string());
        }
    }
    let mut offset = 0;
    for (linenum, line) in source.lines().enumerate() {
        for key in &all_keys {
            if let Some(pos) = line.find(&format!("\"{key}\"")) {
                keys.entry(key.to_lowercase())
                    .or_insert_with(Vec::new)
                    .push(Position::new(
                        LineCol(offset + pos + 1, (linenum + 1, pos + 2)),
                        LineCol(
                            offset + pos + 1 + key.len(),
                            (linenum + 1, pos + 2 + key.len()),
                        ),
                        path.clone(),
                    ));
            }
        }
        offset += line.chars().count() + 1;
    }
    for key in all_keys {
        keys.entry(key.to_lowercase()).or_insert_with(Vec::new);
    }
    keys
}

/// Project with determistic order of `keys` for testing
/// `keys` are sorted, but the `InnerProject` is untouched and retains original order
#[derive(Clone, Debug)]
pub struct ProjectWithSortedKeys {
    #[allow(dead_code)]
    inner: InnerProject,
    #[allow(dead_code)]
    path: WorkspacePath,
    #[allow(dead_code)]
    /// Vec of tuples that represent entries from the original: `HashMap<String, Vec<Position>>`
    keys: Vec<(String, Vec<Position>)>,
    #[allow(dead_code)]
    source: String,
    #[allow(dead_code)]
    comments: Vec<(String, String, Option<String>)>,
}

impl ProjectWithSortedKeys {
    #[must_use]
    pub fn from_project(project: &Project) -> Self {
        let mut keys: Vec<(String, Vec<Position>)> = project
            .keys
            .clone()
            .into_iter()
            .map(|h| (h.0, h.1))
            .collect();
        keys.sort_by(|a, b| b.0.cmp(&a.0));
        Self {
            inner: project.inner.clone(),
            path: project.path.clone(),
            keys,
            source: project.source.clone(),
            comments: project.comments.clone(),
        }
    }
}
