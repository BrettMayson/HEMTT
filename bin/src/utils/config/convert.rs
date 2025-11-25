use std::{io::Write as _, path::PathBuf};

use hemtt_workspace::reporting::WorkspaceFiles;

use super::{derapify::OutputFormat, inspect::get_report};
use crate::Error;

#[derive(clap::Args)]
#[allow(clippy::module_name_repetitions)]
/// Convert config files between formats
pub struct ConvertArgs {
    /// Config file to convert (.cpp, .hpp, .rvmat)
    pub(crate) file: String,
    /// Output format: debin (cpp), json, or json-pretty
    #[arg(short = 'f', long = "format", default_value = "json-pretty")]
    pub(crate) output_format: OutputFormat,
    /// Output file path
    ///
    /// If not specified, uses the input filename with appropriate extension.
    pub(crate) output: Option<String>,
}

/// Convert a config file to another format
pub fn convert(path: &PathBuf, output: Option<&str>, format: OutputFormat) -> Result<(), Error> {
    let report = get_report(path)?;
    let workspacefiles = WorkspaceFiles::new();
    match report {
        Ok(report) => {
            for code in report.codes() {
                if let Some(diag) = code.diagnostic() {
                    eprintln!("{}", diag.to_string(&workspacefiles));
                }
            }
            let config = report.into_config();
            let output = output.map_or_else(
                || {
                    let mut path = path.clone();
                    path.set_extension(format.default_extension());
                    path
                },
                PathBuf::from,
            );
            let _ = fs_err::create_dir_all(
                output
                    .parent()
                    .expect("Output file has no parent directory"),
            );
            let mut output = fs_err::File::create(output)?;
            match format {
                OutputFormat::Debin => output.write_all(config.to_string().as_bytes())?,
                OutputFormat::Json => {
                    output.write_all(serde_json::to_string(&config)?.as_bytes())?;
                }
                OutputFormat::JsonPretty => {
                    output.write_all(serde_json::to_string_pretty(&config)?.as_bytes())?;
                }
            }
            output.flush()?;
            Ok(())
        }
        Err(errors) => {
            for error in errors {
                if let Some(diag) = error.diagnostic() {
                    eprintln!("{}", diag.to_string(&workspacefiles));
                }
            }
            Err(Error::Config(String::from("Config parsing failed")))
        }
    }
}
