use std::io::BufReader;

use quick_xml::se::Serializer;
use serde::{Deserialize, Serialize};

pub mod analyze;
mod key;
mod package;
mod totals;

pub use key::Key;
pub use package::Package;
pub use totals::Totals;
use tracing::error;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Project {
    #[serde(rename = "@name")]
    name: String,
    #[serde(rename = "Package")]
    packages: Vec<Package>,

    #[serde(skip)]
    meta_comments: Vec<(String, String)>,
}

impl Project {
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub fn packages(&self) -> &[Package] {
        &self.packages
    }

    pub fn sort(&mut self) {
        self.packages.sort_by(|a, b| a.name().cmp(b.name()));
        for package in &mut self.packages {
            package.sort();
        }
    }

    /// Read a Project from a reader
    ///
    /// # Errors
    /// [`quick_xml::DeError`] if the reader is not a valid stringtable
    pub fn from_reader<R: std::io::BufRead>(reader: R) -> Result<Self, quick_xml::de::DeError> {
        let mut buffer = String::new();
        let mut reading_comments = false;
        let mut comments = Vec::new();
        let Ok(reader) = reader
            .lines()
            .map(|l| {
                let Ok(l) = l else {
                    error!("Failed to read line: {:?}", l);
                    return l;
                };
                if !reading_comments && !buffer.is_empty() {
                    comments.push((buffer.trim().to_string(), l.trim().to_string()));
                    buffer.clear();
                }
                if l.trim().starts_with("<!--") {
                    reading_comments = true;
                }
                if reading_comments {
                    buffer.push_str(&l);
                    if l.trim().ends_with("-->") {
                        reading_comments = false;
                    }
                }
                Ok(l)
            })
            .collect::<Result<Vec<_>, _>>()
        else {
            return Err(quick_xml::de::DeError::Custom(
                "Failed to read lines".to_string(),
            ));
        };
        let mut this: Self =
            quick_xml::de::from_reader(BufReader::new(reader.join("\n").as_bytes()))?;
        this.meta_comments = comments;
        Ok(this)
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
        self.serialize(ser)?;
        buffer.push('\n');

        for line in buffer.lines() {
            for (before, after) in &self.meta_comments {
                if line.trim().starts_with(after) {
                    let whitespace = line
                        .chars()
                        .take_while(|c| c.is_whitespace())
                        .collect::<String>();
                    writer.write_str(&whitespace)?;
                    writer.write_str(before)?;
                    writer.write_char('\n')?;
                }
            }
            writer.write_str(line)?;
            writer.write_char('\n')?;
        }

        Ok(())
    }
}
