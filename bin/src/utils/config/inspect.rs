use std::{path::PathBuf, sync::Arc};

use hemtt_common::config::ProjectConfig;
use hemtt_config::ConfigReport;
use hemtt_preprocessor::Processor;
use hemtt_workspace::{
    LayerType,
    reporting::{Code, WorkspaceFiles},
};

#[derive(clap::Args)]
#[allow(clippy::module_name_repetitions)]
pub struct InspectArgs {
    /// Config to inspect
    pub(crate) config: String,
}

use crate::Error;

/// Prints information about a config to stdout
///
/// # Errors
/// [`Error::Preprocessor`] if the file can not be preprocessed
pub fn inspect(file: &PathBuf) -> Result<(), Error> {
    let report = get_report(file)?;
    let workspacefiles = WorkspaceFiles::new();
    match report {
        Ok(report) => {
            println!("Config is valid!");
            if report.patches().is_empty() {
                println!(" - Contains no CfgPatches");
            } else {
                println!(" - Contains the following CfgPatches:");
                for patch in report.patches() {
                    println!(
                        "    - {}, requires {}",
                        patch.name().as_str(),
                        patch.required_version()
                    );
                }
            }
            for code in report.codes() {
                if let Some(diag) = code.diagnostic() {
                    eprintln!("{}", diag.to_string(&workspacefiles));
                }
            }
            Ok(())
        }
        Err(errors) => {
            for error in errors {
                if let Some(diag) = error.diagnostic() {
                    eprintln!("{}", diag.to_string(&workspacefiles));
                }
            }
            Ok(())
        }
    }
}

pub fn get_report(file: &PathBuf) -> Result<Result<ConfigReport, Vec<Arc<dyn Code>>>, Error> {
    assert!(file.is_file());
    let folder = PathBuf::from(&file)
        .parent()
        .expect("File has no parent")
        .to_path_buf();
    let workspace = hemtt_workspace::Workspace::builder()
        .physical(&folder, LayerType::Source)
        .finish(
            Some(ProjectConfig::test_project()),
            false,
            &hemtt_common::config::PDriveOption::Disallow,
        )?;
    let source = workspace
        .join(
            file.file_name()
                .expect("has a filename")
                .to_str()
                .expect("valid utf-8"),
        )
        .expect("File is valid");
    let processed = Processor::run(
        &source,
        &hemtt_common::config::PreprocessorOptions::default(),
    )
    .map_err(|e| e.1)?;
    Ok(hemtt_config::parse(
        Some(&ProjectConfig::test_project()),
        &processed,
    ))
}
