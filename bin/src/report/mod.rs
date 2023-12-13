use std::{
    fs::OpenOptions,
    io::{BufWriter, Write},
    sync::Arc,
};

use hemtt_common::reporting::Code;

use crate::Error;

#[derive(Debug, Default)]
pub struct Report {
    warnings: Vec<Arc<dyn Code>>,
    errors: Vec<Arc<dyn Code>>,
}

impl Report {
    #[must_use]
    pub fn new() -> Self {
        Self {
            warnings: Vec::new(),
            errors: Vec::new(),
        }
    }

    /// Write the report to the `ci_annotations.txt` file for GitHub Actions
    ///
    /// # Errors
    /// [`std::io::Error`] if the file cannot be opened
    pub fn write_ci_annotations(&self) -> Result<(), Error> {
        trace!("writing ci annotations to .hemttout/ci_annotations.txt");
        let mut ci_annotation = BufWriter::new(
            OpenOptions::new()
                .write(true)
                .append(true)
                .open(".hemttout/ci_annotations.txt")?,
        );
        for code in self.warnings.iter().chain(self.errors.iter()) {
            for annotation in code.ci() {
                ci_annotation.write_all(annotation.line().as_bytes())?;
            }
        }
        trace!("wrote ci annotations to .hemttout/ci_annotations.txt");
        Ok(())
    }

    pub fn write_to_stdout(&self) {
        for code in self.warnings.iter().chain(self.errors.iter()) {
            if let Some(report) = code.report() {
                eprintln!("{report}");
            }
        }
    }

    pub fn merge(&mut self, other: Self) {
        self.warnings.extend(other.warnings);
        self.errors.extend(other.errors);
    }

    pub fn warn(&mut self, warning: Arc<dyn Code>) {
        self.warnings.push(warning);
    }

    pub fn error(&mut self, error: Arc<dyn Code>) {
        self.errors.push(error);
    }

    pub fn add_warnings(&mut self, warnings: Vec<Arc<dyn Code>>) {
        self.warnings.extend(warnings);
    }

    pub fn add_errors(&mut self, errors: Vec<Arc<dyn Code>>) {
        self.errors.extend(errors);
    }

    #[must_use]
    pub fn warnings(&self) -> &[Arc<dyn Code>] {
        &self.warnings
    }

    #[must_use]
    pub fn errors(&self) -> &[Arc<dyn Code>] {
        &self.errors
    }

    #[must_use]
    pub fn failed(&self) -> bool {
        !self.errors.is_empty()
    }
}
