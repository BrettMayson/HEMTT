use quick_xml::se::Serializer;
use serde::{Deserialize, Serialize};

pub mod analyze;
mod key;
mod package;
mod totals;

pub use key::Key;
pub use package::Package;
pub use totals::Totals;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Project {
    #[serde(rename = "@name")]
    name: String,
    #[serde(rename = "Package")]
    packages: Vec<Package>,
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
    /// [`quick_xml::de::DeError`] if the reader is not a valid stringtable
    pub fn from_reader<R: std::io::BufRead>(reader: R) -> Result<Self, quick_xml::de::DeError> {
        quick_xml::de::from_reader(reader)
    }

    /// Write a Project to a writer
    ///
    /// # Errors
    /// [`quick_xml::ser::Error`] if the writer fails to write
    pub fn to_writer<W: std::fmt::Write>(
        &self,
        writer: &mut W,
    ) -> Result<(), quick_xml::de::DeError> {
        // If this write fails, the serializer will also throw an error
        let _ = writer.write_str(r#"<?xml version="1.0" encoding="utf-8"?>"#);
        let _ = writer.write_char('\n');
        let mut ser = Serializer::new(writer);
        ser.indent(' ', 4);
        self.serialize(ser)
    }
}
