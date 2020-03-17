use std::fs;

use dialoguer::Confirmation;
use glob::glob;

use crate::{Addon, AddonList, HEMTTError, Project, Report, Stage, Task};

#[derive(Clone)]
pub struct Release {
    pub force_release: bool,
}
impl Task for Release {
    fn single(&self, addons: Vec<Result<(Report, Addon), HEMTTError>>, p: &Project, _: &Stage) -> AddonList {
        let addons: Vec<_> = addons
            .into_iter()
            .map(|d| {
                let (report, addon) = d.unwrap();
                Ok((report, addon))
            })
            .collect();

        // Prepare release directory
        let release_folder = p.release_dir()?;
        if release_folder.exists() {
            if self.force_release {
                std::fs::remove_dir_all(&release_folder)?;
            } else {
                let error = HEMTTError::generic("Release already exists", "Use `--force-release` to overwrite");
                if *crate::CI {
                    return Err(error);
                } else {
                    println!();
                    warn!("Release already exists");
                    if Confirmation::new().with_text("Do you want to continue?").interact()? {
                        std::fs::remove_dir_all(&release_folder)?;
                    } else {
                        return Err(error);
                    }
                    println!();
                }
            }
        }

        create_dir!(release_folder)?;

        for dir in &["keys"] {
            create_dir!({
                let mut d = release_folder.clone();
                d.push((*dir).to_string());
                d
            })?;
        }

        for mut file in p.files.to_owned() {
            // Mirror directory structure if path ends in slash
            let mirror_structure = if file.ends_with('/') {
                file.pop();
                true
            } else {
                false
            };

            for entry in glob(&file)? {
                if let Ok(path) = entry {
                    let mut d = release_folder.clone();
                    if mirror_structure {
                        d.push(path.parent().unwrap());
                        create_dir!(d)?;
                    }

                    if fs::metadata(&path).unwrap().is_dir() {
                        debug!("Copying dir {:#?} to {:#?}", path, d);
                        copy_dir!(path, d)?;
                    } else {
                        d.push(path.file_name().unwrap().to_str().unwrap().to_owned());
                        debug!("Copying file {:#?} to {:#?}", path, d);
                        copy_file!(path, d)?;
                    }
                }
            }
        }

        for data in &addons {
            let (_, addon) = data.as_ref().unwrap();
            addon.release(&release_folder, &p)?;
        }

        Ok(addons)
    }
}
