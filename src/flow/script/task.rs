use rayon::prelude::*;
use regex::Regex;
use subprocess::Exec;

use crate::error::*;
use crate::{Addon, AddonList, HEMTTError, Project, Stage, Task};

#[derive(Clone)]
pub struct Script {
    pub release: bool,
}
impl Task for Script {
    fn single(&self, addons: AddonList, p: &Project, s: &Stage) -> Result<AddonList, HEMTTError> {
        let steps = Self::get_scripts(s, p)?;

        for step in steps {
            info!("Starting Script ({}): {}", s.to_string(), p.render(&step, None)?);
            Self::execute(&step, false, &addons, p, s, self.release)?;
        }

        Ok(addons)
    }
}

impl Script {
    pub fn execute(
        command: &str,
        output: bool,
        addons: &[Result<(bool, bool, Addon), HEMTTError>],
        p: &Project,
        s: &Stage,
        release: bool,
    ) -> Result<(), HEMTTError> {
        let mut cmd = command.to_owned();
        match cmd.remove(0) {
            // HEMTT Command
            '@' => {
                let args_re = Regex::new(r##"([^=\s"]*)=(?:"([^"\\]*(\\.[^"\\]*)*)"|'([^'\\]*(\\.[^'\\]*)*)'|([^"\s]+))|"([^"\\]*(\\.[^"\\]*)*)"|'([^'\\]*(\\.[^'\\]*)*)'|([^"\s]+)"##).unwrap();
                let mut args: Vec<String> = vec!["hemtt".to_owned()];
                for mat in args_re.find_iter(&cmd) {
                    args.push(crate::render::run(mat.as_str(), Some(&s.to_string()), &p.get_variables())?);
                }
                crate::execute(&args, false)?;
            }
            // Script
            '!' => {
                let script = p.scripts.get(&cmd);
                if let Some(script) = script {
                    // false positive, remove in the future
                    #[allow(clippy::ifs_same_cond)]
                    let steps = if cfg!(windows) && !script.steps_windows.is_empty() {
                        &script.steps_windows
                    } else if cfg!(unix) && !script.steps_linux.is_empty() {
                        &script.steps_linux
                    } else {
                        &script.steps
                    };
                    if script.should_run(release) {
                        if script.foreach {
                            for step in steps {
                                let exec = |data: &Result<(bool, bool, Addon), HEMTTError>| {
                                    if let Ok((_, _, addon)) = data {
                                        let step = crate::render::run(
                                            step,
                                            Some(&format!("script:{}", &cmd)),
                                            &addon.get_variables(p),
                                        )
                                        .unwrap_or_print();
                                        Self::execute(&step, script.show_output, addons, p, s, release).unwrap_or_print();
                                    }
                                };
                                if script.parallel {
                                    addons.par_iter().for_each(exec);
                                } else {
                                    addons.iter().for_each(exec);
                                }
                            }
                        } else {
                            for step in steps {
                                let step = crate::render::run(step, Some(&format!("script:{}", &cmd)), &p.get_variables())?;
                                Self::execute(&step, script.show_output, addons, p, s, release)?;
                            }
                        }
                    } else {
                        info!("Script `{}` skipped", &cmd);
                    }
                } else {
                    error!("Script `{}` does not exist", &cmd);
                    std::process::exit(3);
                }
            }
            _ => {
                let cmd = command.to_string().replace("\\", "\\\\");
                let shell = Exec::shell(crate::render::run(command, Some(&s.to_string()), &p.get_variables())?)
                    .capture()
                    .unwrap_or_print();
                let out = &shell.stdout_str();
                if output {
                    for line in out.lines() {
                        println!("{}", line);
                    }
                }
                if !shell.success() {
                    error!("Failed to execute shell command: {}", cmd);
                    std::process::exit(2);
                }
            }
        }
        Ok(())
    }

    pub fn get_scripts(s: &Stage, p: &Project) -> Result<Vec<String>, HEMTTError> {
        Ok(match s {
            Stage::Check => &p.check,
            Stage::PreBuild => &p.prebuild,
            Stage::PostBuild => &p.postbuild,
            Stage::ReleaseBuild => &p.releasebuild,
            _ => {
                // Invalid, we should never be here
                error!("Scripts tried to run during an invalid stage, please report this");
                unimplemented!()
            }
        }
        .clone())
    }
}
