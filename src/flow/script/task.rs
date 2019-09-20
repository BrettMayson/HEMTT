use regex::Regex;
use subprocess::Exec;

use crate::error::*;
use crate::{Addon, AddonList, HEMTTError, Project, Report, Stage, Task};

#[derive(Clone)]
pub struct Script {}
impl Task for Script {
    fn single(&self, addons: Vec<Result<(Report, Addon), HEMTTError>>, p: &Project, s: &Stage) -> AddonList {
        let steps = Script::get_scripts(s, p)?;

        for step in steps {
            println!("{}: `{}`", s, step);
            Script::execute(&step, false)?;
        }

        Ok(addons)
    }
}

impl Script {
    pub fn execute(command: &str, output: bool) -> Result<(), HEMTTError> {
        let mut cmd = command.to_owned();
        match cmd.remove(0) {
            // HEMTT Command
            '@' => {
                let args_re = Regex::new(r##"([^=\s"]*)=(?:"([^"\\]*(\\.[^"\\]*)*)"|'([^'\\]*(\\.[^'\\]*)*)'|([^"\s]+))|"([^"\\]*(\\.[^"\\]*)*)"|'([^'\\]*(\\.[^'\\]*)*)'|([^"\s]+)"##).unwrap();
                let mut args: Vec<String> = vec!["hemtt".to_owned()];
                for mat in args_re.find_iter(&cmd) {
                    args.push(mat.as_str().to_owned());
                }
                crate::execute(&args, false)?;
            }
            // Script
            '!' => {}
            _ => {
                let cmd = command.to_string().replace("\\", "\\\\");
                let shell = Exec::shell(&command).capture().unwrap_or_print();
                let out = &shell.stdout_str();
                if output {
                    for line in out.lines() {
                        println!("-   {}", line);
                    }
                }
                if !shell.success() {
                    errormessage!("Failed to execute shell command", cmd);
                    std::process::exit(2);
                }
            }
        }
        Ok(())
    }

    pub fn get_scripts(s: &Stage, p: &Project) -> Result<Vec<String>, HEMTTError> {
        Ok(match s {
            Stage::Check => &p.check,
            Stage::Build => {
                println!("Build scripts do not exist yet");
                unimplemented!()
            }
            Stage::PreBuild => &p.prebuild,
            Stage::PostBuild => &p.postbuild,
            Stage::ReleaseBuild => &p.releasebuild,
            _ => {
                // Invalid, we should never be here
                println!("Scripts tried to run during an invalid stage, please report this");
                unimplemented!()
            }
        }
        .clone())
    }
}
