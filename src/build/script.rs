use colored::*;
use handlebars::to_json;
use pbr::ProgressBar;
use rayon::prelude::*;
use regex::Regex;
use serde::{Serialize, Deserialize};
use subprocess::Exec;

use std::path::PathBuf;
use std::io::Error;
use std::sync::{Arc, Mutex};

use crate::error;
use crate::error::*;
use crate::state::{State, Stage};
use crate::build::PBOResult;

struct ScriptStatus {
    progressbar: ProgressBar<std::io::Stdout>,
}
impl ScriptStatus {
    pub fn new() -> Self {
        let mut obj = ScriptStatus {
            progressbar: ProgressBar::new(0)
        };
        obj.progressbar.set_width(Some(70));
        obj.progressbar.show_speed = false;
        obj
    }
    pub fn pb(&mut self) -> &mut ProgressBar<std::io::Stdout> {
        &mut self.progressbar
    }
    pub fn total(&mut self, count: u64) {
        self.progressbar.total = count;
    }
}

#[derive(Serialize, Deserialize)]
pub struct BuildScript {
    #[serde(skip_serializing_if = "crate::is_true")]
    #[serde(default = "crate::dft_true")]
    pub debug: bool,
    #[serde(skip_serializing_if = "crate::is_true")]
    #[serde(default = "crate::dft_true")]
    pub release: bool,
    #[serde(skip_serializing_if = "crate::is_false")]
    #[serde(default = "crate::dft_false")]
    pub foreach: bool,
    #[serde(skip_serializing_if = "crate::is_false")]
    #[serde(default = "crate::dft_false")]
    pub parallel: bool,
    #[serde(skip_serializing_if = "crate::is_false")]
    #[serde(default = "crate::dft_false")]
    pub show_output: bool,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default = "Vec::new")]
    pub steps: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default = "Vec::new")]
    pub steps_windows: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default = "Vec::new")]
    pub steps_linux: Vec<String>,
}

impl BuildScript {
    pub fn run(&self, p: &crate::project::Project, state: &State) -> Result<(), Error> {
        let mut steps = &self.steps;
        let mut pb = ScriptStatus::new();
        let pbm = Arc::new(Mutex::new(&mut pb));
        if cfg!(windows) {
            if !self.steps_windows.is_empty() {
                steps = &self.steps_windows;
            }
        } else {
            if !self.steps_linux.is_empty() {
                steps = &self.steps_linux;
            }
        }
        if self.foreach {
            if self.parallel {
                match state.stage {
                    Stage::PreBuild => {
                        pbm.lock().unwrap().total(state.addons.len() as u64);
                        state.addons.par_iter().for_each(|addon| {
                            self.run_pathbuf(&p, &addon, &steps, &state, &pbm).unwrap_or_print();
                            pbm.lock().unwrap().pb().inc();
                        });
                        //pbm.lock().unwrap().pb().finish_print(&nicefmt!(green, "Executed", format!("{}{}", state.addons.len(), repeat!(" ", 60))));
                        //println!();
                        finishpb!(pbm.lock().unwrap().pb(), green, "Executed", state.addons.len());
                    },
                    Stage::PostBuild | Stage::ReleaseBuild => {
                        let built = &state.result.unwrap().built;
                        pbm.lock().unwrap().total(built.len() as u64);
                        built.par_iter().for_each(|addon| {
                            self.run_pboresult(&p, &addon, &steps, &state, &pbm).unwrap_or_print();
                            pbm.lock().unwrap().pb().inc();
                        });
                        finishpb!(pbm.lock().unwrap().pb(), green, "Executed", built.len());
                    },
                    _ => {}
                }
            } else {
                match state.stage {
                    Stage::PreBuild => {
                        pbm.lock().unwrap().total(state.addons.len() as u64);
                        for addon in &state.addons {
                            self.run_pathbuf(&p, &addon, &steps, &state, &pbm)?;
                            pbm.lock().unwrap().pb().inc();
                        }
                        finishpb!(pbm.lock().unwrap().pb(), green, "Executed", state.addons.len());
                    },
                    Stage::PostBuild | Stage::ReleaseBuild => {
                        let built = &state.result.unwrap().built;
                        pbm.lock().unwrap().total(built.len() as u64);
                        for addon in built {
                            self.run_pboresult(&p, &addon, &steps, &state, &pbm)?;
                            pbm.lock().unwrap().pb().inc();
                        }
                        finishpb!(pbm.lock().unwrap().pb(), green, "Executed", built.len());
                    },
                    _ => {}
                }
            }
        } else {
            for command in steps {
                execute(&p, &p.render(&command), &state, self.show_output, None)?;
            }
        }
        Ok(())
    }

    fn run_pathbuf(&self, p: &crate::project::Project, addon: &PathBuf, steps: &Vec<String>, state: &State, pbm: &Arc<Mutex<&mut ScriptStatus>>) -> Result<(), Error> {
        if !self.show_output {
            print!("\r");
            eprint!("\r");
        }
        let mut data = p.template_data.clone();
        let name = addon.file_name().unwrap().to_str().unwrap().to_owned();
        pbm.lock().unwrap().pb().message(&format!("{}{} ", &name, repeat!(" ",
            if &name.len() > &20 {0} else {20 - &name.len()}
        )));
        data.insert("addon", to_json(name.clone()));
        data.insert("source", to_json(addon.to_str().unwrap().to_owned()));
        let mut target = addon.parent().unwrap().to_path_buf();
        if p.prefix.is_empty() {
            target.push(&format!("{}.pbo", &name));
        } else {
            target.push(&format!("{}_{}.pbo", p.prefix, &name));
        }
        data.insert("target", to_json(target.to_str().unwrap().to_owned()));
        for command in steps {
            execute(&p, &crate::template::render(&command, &data), &state, self.show_output, Some(&mut pbm.lock().unwrap()))?;
        }
        Ok(())
    }

    fn run_pboresult(&self, p: &crate::project::Project, addon: &PBOResult, steps: &Vec<String>, state: &State, pbm: &Arc<Mutex<&mut ScriptStatus>>) -> Result<(), Error> {
        if !self.show_output {
            print!("\r");
            eprint!("\r");
        }
        let mut data = p.template_data.clone();
        let name = addon.source.file_name().unwrap().to_str().unwrap().to_owned();
        pbm.lock().unwrap().pb().message(&format!("{}{} ", &name, repeat!(" ",
            if &name.len() > &20 {0} else {20 - &name.len()}
        )));
        data.insert("addon", to_json(name.clone()));
        data.insert("source", to_json(addon.source.to_str().unwrap().to_owned()));
        let mut target = addon.source.parent().unwrap().to_path_buf();
        if p.prefix.is_empty() {
            target.push(&format!("{}.pbo", &name));
        } else {
            target.push(&format!("{}_{}.pbo", p.prefix, &name));
        }
        data.insert("target", to_json(target.to_str().unwrap().to_owned()));
        data.insert("time", to_json(addon.time.to_string()));
        for command in steps {
            execute(&p, &crate::template::render(&command, &data), &state, self.show_output, Some(&mut pbm.lock().unwrap()))?;
        }
        Ok(())
    }
}

pub fn run(p: &crate::project::Project, state: &State) -> Result<(), Error> {
    let mut name = String::new();
    let mut commands: Vec<String> = Vec::new();
    match &state.stage {
        Stage::PreBuild => {
            name = "Pre Build".to_owned();
            commands = p.prebuild.clone();
        },
        Stage::PostBuild => {
            name = "Post Build".to_owned();
            commands = p.postbuild.clone();
        }
        Stage::ReleaseBuild => {
            name = "Release Build".to_owned();
            commands = p.releasebuild.clone();
        },
        _ => {
            println!("/shrug");
        }
    }
    if commands.is_empty() {return Ok(())};
    println!("  {} {}", "Starting".green().bold(), &name);
    for command in commands {
        execute(&p, &p.render(&command), &state, true, None)?;
    }
    println!("  {} {}", "Finished".green().bold(), &name);
    Ok(())
}

fn execute(p: &crate::project::Project, command: &String, state: &State, output: bool, pb: Option<&mut ScriptStatus>) -> Result<(), Error> {
    let mut name = command.clone();

    let prefix = match &pb {
        Some(_) => "\r",
        None => ""
    };

    match name.remove(0) {
        // TODO replace println with color macros, need to deal with that prefix
        '@' => {
            let re = Regex::new(r##"([^=\s"]*)=(?:"([^"\\]*(\\.[^"\\]*)*)"|'([^'\\]*(\\.[^'\\]*)*)'|([^"\s]+))|"([^"\\]*(\\.[^"\\]*)*)"|'([^'\\]*(\\.[^'\\]*)*)'|([^"\s]+)"##).unwrap();
            let mut args: Vec<String> = Vec::new();
            for mat in re.find_iter(&name) {
                args.push(mat.as_str().to_owned());
            }
            if output {println!("{}   {} {}", prefix, "Utility".green().bold(), &name)};
            match crate::utilities::find(&args[0]) {
                Some(v) => crate::utilities::run(&v, &mut args)?,
                None => return Err(error!("Unknown Utility: {}", &name))
            };
            if let Some(_) = &pb {
                &pb.unwrap().pb().tick();
            }
            if output {println!("{}      {} {}", prefix, "Done".green().bold(), &name)};
        },
        '!' => {
            if output {println!("{}    {} {}", prefix, "Script".green().bold(), &name)};
            p.script(&name, &state)?;
            if let Some(_) = &pb {
                &pb.unwrap().pb().tick();
            }
            if output {println!("{}      {} {}", prefix, "Done".green().bold(), &name)};
        },
        _   => {
            let cmd = command.clone().replace("\\", "\\\\");
            if output {println!("{} {} {}{}", prefix, "Executing".green().bold(), &cmd.bold(), repeat!(" ", 60 - &cmd.len()))};
            if let Some(_) = &pb {
                &pb.unwrap().pb().tick();
            }
            let shell = Exec::shell(&command).capture().unwrap_or_print();
            let out = &shell.stdout_str();
            if output {
                for line in out.lines() {
                    println!("{}           {}{}", prefix, line, repeat!(" ", 70 - line.len()));
                }
            }
            if !shell.success() {
                red!("Failed", &cmd);
                std::process::exit(2);
            }
        }
    }
    Ok(())
}
