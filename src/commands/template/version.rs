use std::fs::File;
use std::io::{BufReader, BufRead};
use std::path::{Path, PathBuf};

use crate::{HEMTTError};
use super::Template;

impl Template {
    pub fn get_version(&self) -> Result<String, HEMTTError> {
        if PathBuf::from("./hemtt/template/scripts/get_version.lua").exists() {
            Ok(self.eval_file("./hemtt/template/scripts/get_version.lua", |_| {}))
        } else {
            if Path::new("addons/main/script_version.hpp").exists() {
                let f = BufReader::new(File::open("addons/main/script_version.hpp")?);
                let (mut major, mut minor, mut patch, mut build) = (0, 0, 0, String::new());
                for line in f.lines() {
                    let line = line?;
                    let mut split = line.split(' ');
                    let define = split.next().unwrap();
                    if define != "#define" { continue; }
                    let key = split.next().unwrap();
                    let value = split.next().unwrap();
                    match key {
                        "MAJOR" => {
                            major = value.parse().map_err(|_| HEMTTError::GENERIC(
                                "Unable to interpret version number part".to_owned(),
                                value.to_owned()
                            ))?;
                        },
                        "MINOR" => {
                            minor = value.parse().map_err(|_| HEMTTError::GENERIC(
                                "Unable to interpret version number part".to_owned(),
                                value.to_owned()
                            ))?;
                        },
                        "PATCHLVL" | "PATCH" => {
                            patch = value.parse().map_err(|_| HEMTTError::GENERIC(
                                "Unable to interpret version number part".to_owned(),
                                value.to_owned()
                            ))?;
                        },
                        "BUILD" => {
                            build = String::from(value);
                        },
                        _ => {}
                    }
                }
                Ok(
                    if build.is_empty() {
                        format!("{}.{}.{}", major, minor, patch)
                    } else {
                        format!("{}.{}.{}.{}", major, minor, patch, build)
                    }
                )
            } else {
                Err(HEMTTError::generic(
                    "No way to determine the version number was detected",
                    if cfg!(windows) {
                        "Use `cmd /C \"set APP_VERSION={} && hemtt ...\"` to specify a version for this build"
                    } else {
                        "Use `APP_VERSION={} hemtt ...` to specify a version for this build"
                    },
                ))
            }
        }
    }
}
