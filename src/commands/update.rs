use crate::error::PrintableError;
use crate::{Command, HEMTTError};

pub struct Update {}
impl Command for Update {
    fn register(&self) -> clap::App {
        clap::SubCommand::with_name("update").about("Update HEMTT to the latest stable release")
    }

    fn require_project(&self) -> bool {
        false
    }

    fn run_no_project(&self, _: &clap::ArgMatches) -> Result<(), HEMTTError> {
        if cfg!(debug_assertions) {
            println!("You are running a debug version, which can not be updated");
            return Ok(());
        }

        let target = self_update::get_target();
        let status = self_update::backends::github::Update::configure()
            .repo_owner("SynixeBrett")
            .repo_name("HEMTT")
            .target(&target)
            .bin_name(if cfg!(windows) { "hemtt.exe" } else { "hemtt" })
            .show_download_progress(true)
            .current_version(env!("CARGO_PKG_VERSION"))
            .build()
            .unwrap_or_print()
            .update()
            .unwrap_or_print();
        println!("\nUsing Version: {}", status.version());
        Ok(())
    }
}
