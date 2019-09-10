<<<<<<< HEAD
use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
=======
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::collections::HashMap;
>>>>>>> mission_generate, translation

use armake2::pbo::PBO;
use regex::Regex;

use crate::{Command, HEMTTError};

pub struct MissionGenerate {}
impl Command for MissionGenerate {
    fn register(&self) -> clap::App {
        clap::SubCommand::with_name("mission_generate")
            .about("Generate `pbos` for multiple maps from a single mission")
            .arg(clap::Arg::with_name("mission"))
            .arg(
                clap::Arg::with_name("maps")
                    .multiple(true)
                    .takes_value(true)
                    .default_value("maps.txt"),
            )
    }

    fn require_project(&self) -> bool {
        false
    }

    fn run_no_project(&self, args: &clap::ArgMatches) -> Result<(), HEMTTError> {
        // Match maps ignoring comments
        let l_re = Regex::new(r"(?m)(.+?)\s?(?:/.+?)?$").unwrap();
        let m_re = Regex::new(r"(?m)(.+?)\.(.+)").unwrap();

        let mission_source = args.value_of("mission").unwrap();
        let mission_file = format!("{}.pbo", mission_source);
        let mission_map = if m_re.is_match(&mission_source) {
            let cap = m_re.captures(&mission_source).unwrap();
            cap.get(2).unwrap().as_str()
        } else {
            error!("Mission folder must have a map defined");
            std::process::exit(1);
        };

        let maps: Vec<_> = args.values_of("maps").unwrap().collect();
        let maps = if maps.len() == 1 && PathBuf::from(maps[0]).exists() {
            let mut new_maps = Vec::new();
            let file = open_file!(maps[0])?;
            for line in BufReader::new(file).lines() {
                let line = line.unwrap();
                let m = l_re.captures(&line).unwrap();
                let line = m.get(1).unwrap().as_str();
                if line == "" {
                    continue;
                }
                new_maps.push(line.to_owned());
            }
            new_maps
        } else {
            maps.into_iter().map(|m| m.to_owned()).collect()
        };

        create_dir!("missions")?;

        let mut out = create_file!(&mission_file)?;
        let mut pbo = PBO::from_directory(PathBuf::from(&mission_source), false, &[], &[]).unwrap();
        pbo.header_extensions = HashMap::new();
        pbo.write(&mut out).unwrap();

        for map in maps {
            let m = mission_source.replace(mission_map, &map);
            copy_file!(mission_file, format!("missions{}{}.pbo", std::path::MAIN_SEPARATOR, m))?;
            println!("{}", m);
        }

        Ok(())
    }
}
