use colored::*;
use serde::{Serialize, Deserialize};

use std::io::Error;

use crate::error;

#[derive(Serialize, Deserialize)]
pub struct BuildScript {
    pub name: String,
    #[serde(skip_serializing_if = "is_true")]
    #[serde(default = "dft_true")]
    pub debug: bool,
    #[serde(skip_serializing_if = "is_true")]
    #[serde(default = "dft_true")]
    pub release: bool,
    #[serde(skip_serializing_if = "is_false")]
    #[serde(default)]
    pub prebuild: bool,
    #[serde(skip_serializing_if = "is_false")]
    #[serde(default)]
    pub postbuild: bool,
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
    pub fn run(&self, p: &crate::project::Project) -> Result<(), Error> {
        for step in &self.steps {
            println!("{}", step);
        }
        Ok(())
    }
}

pub fn run(commands: &Vec<String>) -> Result<(), Error> {
    for command in commands {
        let mut name = command.clone();
        match name.remove(0) {
            '@' => {
                println!("   {} {}", "Running".green().bold(), &name);
                match crate::utilities::find(&name) {
                    Some(v) => crate::utilities::run(&v),
                    None => return Err(error!("Unknown Utility: {}", &name))
                };
            },
            '!' => {
                println!("   {} {} (scripts not implemented)", "Running".yellow().bold(), &name);
            },
            _   => println!("   Command {}", command)
        }
    }
    Ok(())
}

fn is_true(v: &bool) -> bool { v.clone() }
fn is_false(v: &bool) -> bool { !v.clone() }
fn dft_true() -> bool { true }
