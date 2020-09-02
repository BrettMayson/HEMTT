mod status;

use crate::Command;
use hemtt::HEMTTError;

mod version;

pub struct Project;
impl Command for Project {
    fn register(&self) -> clap::App {
        clap::SubCommand::with_name("project")
            .version(*crate::VERSION)
            .about("Get the status of your project")
            .subcommand(
                clap::SubCommand::with_name("version")
                    .about("Print the project version")
                    .subcommand(
                        clap::SubCommand::with_name("inc")
                            .about("Increment the version")
                            .subcommand(
                                clap::SubCommand::with_name("major").about("Increment major"),
                            )
                            .subcommand(
                                clap::SubCommand::with_name("minor").about("Increment minor"),
                            )
                            .subcommand(
                                clap::SubCommand::with_name("patch").about("Increment patch"),
                            ),
                    ),
            )
    }

    fn run(&self, a: &clap::ArgMatches, mut p: hemtt::Project) -> Result<(), HEMTTError> {
        match a.subcommand() {
            ("version", Some(b)) => {
                version::run(&mut p, b)?;
            }
            _ => println!("Not implemented"),
        }
        Ok(())
    }
}
