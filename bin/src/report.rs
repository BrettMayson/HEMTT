use std::{
    fs::OpenOptions,
    io::{BufWriter, Write},
    sync::Arc,
};

use hemtt_common::reporting::{Code, WorkspaceFiles};

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
                .append(true)
                .open(".hemttout/ci_annotations.txt")?,
        );
        let workspace_files = WorkspaceFiles::new();
        for code in self
            .warnings(WithIncludes::No)
            .iter()
            .chain(self.errors(WithIncludes::No).iter())
        {
            if let Some(diag) = code.diagnostic() {
                let annotations = diag.to_annotations(&workspace_files);
                for annotation in annotations {
                    ci_annotation.write_all(annotation.line().as_bytes())?;
                }
            }
        }
        trace!(
            "wrote {} ci annotations to .hemttout/ci_annotations.txt",
            self.warnings.len() + self.errors.len()
        );
        Ok(())
    }

    pub fn write_to_stdout(&self) {
        let with_includes = if std::env::var("HEMTT_REPORT_WITH_INCLUDES") == Ok("true".to_string()) {
            WithIncludes::Yes
        } else {
            WithIncludes::No
        };
        let workspace_files = WorkspaceFiles::new();
        for code in self
            .warnings(with_includes)
            .iter()
            .chain(self.errors(with_includes).iter())
        {
            if let Some(diag) = code.diagnostic() {
                eprintln!("{}", diag.to_string(&workspace_files));
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
    pub fn warnings(&self, includes: WithIncludes) -> Vec<Arc<dyn Code>> {
        filter_codes(&self.warnings, includes)
    }

    #[must_use]
    pub fn errors(&self, includes: WithIncludes) -> Vec<Arc<dyn Code>> {
        filter_codes(&self.errors, includes)
    }

    #[must_use]
    /// Returns `true` if there are any errors
    pub fn failed(&self) -> bool {
        !self.errors.is_empty()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WithIncludes {
    Yes,
    No,
}

fn filter_codes(codes: &[Arc<dyn Code>], includes: WithIncludes) -> Vec<Arc<dyn Code>> {
    if includes == WithIncludes::Yes {
        return codes.to_vec();
    }
    codes
        .iter()
        .filter(|c| {
            if includes == WithIncludes::Yes {
                true
            } else {
                c.diagnostic()
                    .map_or(true, |d| !d.labels.iter().any(|l| l.file().is_include()))
            }
        })
        .cloned()
        .collect::<Vec<_>>()
}
