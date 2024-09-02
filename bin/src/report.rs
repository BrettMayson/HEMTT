use std::{
    fs::OpenOptions,
    io::{BufWriter, Write},
    sync::Arc,
};

use hemtt_workspace::reporting::{Code, Codes, Severity, WorkspaceFiles};

use crate::Error;

#[derive(Debug, Default)]
pub struct Report {
    codes: Codes,
}

impl Report {
    #[must_use]
    pub fn new() -> Self {
        Self { codes: Vec::new() }
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
            .helps(WithIncludes::No)
            .iter()
            .chain(self.warnings(WithIncludes::No).iter())
            .chain(self.errors().iter())
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
            self.codes.len()
        );
        Ok(())
    }

    pub fn write_to_stdout(&self) {
        let with_includes = if std::env::var("HEMTT_REPORT_WITH_INCLUDES") == Ok("true".to_string())
        {
            WithIncludes::Yes
        } else {
            WithIncludes::No
        };
        let workspace_files = WorkspaceFiles::new();
        for code in self
            .helps(with_includes)
            .iter()
            .chain(self.warnings(with_includes).iter())
            .chain(self.errors().iter())
        {
            if let Some(diag) = code.diagnostic() {
                eprintln!("{}", diag.to_string(&workspace_files));
            }
        }
    }

    pub fn merge(&mut self, other: Self) {
        self.codes.extend(other.codes);
    }

    pub fn push(&mut self, warning: Arc<dyn Code>) {
        self.codes.push(warning);
    }

    pub fn extend(&mut self, codes: Vec<Arc<dyn Code>>) {
        self.codes.extend(codes);
    }

    #[must_use]
    pub fn errors(&self) -> Vec<Arc<dyn Code>> {
        filter_codes(&self.codes, Severity::Error, WithIncludes::Yes)
    }

    #[must_use]
    pub fn warnings(&self, includes: WithIncludes) -> Vec<Arc<dyn Code>> {
        filter_codes(&self.codes, Severity::Warning, includes)
    }

    #[must_use]
    pub fn helps(&self, includes: WithIncludes) -> Vec<Arc<dyn Code>> {
        let mut help = filter_codes(&self.codes, Severity::Help, includes);
        help.extend(filter_codes(&self.codes, Severity::Note, includes));
        help
    }

    #[must_use]
    /// Returns `true` if there are any errors
    pub fn failed(&self) -> bool {
        !self.errors().is_empty()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WithIncludes {
    Yes,
    No,
}

fn filter_codes(
    codes: &[Arc<dyn Code>],
    severity: Severity,
    includes: WithIncludes,
) -> Vec<Arc<dyn Code>> {
    codes
        .iter()
        .filter(|c| c.severity() == severity)
        .filter(|c| {
            if includes == WithIncludes::Yes {
                true
            } else {
                !c.include()
                    && c.diagnostic()
                        .map_or(true, |d| !d.labels.iter().any(|l| l.file().is_include()))
            }
        })
        .cloned()
        .collect::<Vec<_>>()
}
