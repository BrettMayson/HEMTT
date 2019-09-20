use dialoguer::Confirmation;
use glob::glob;

use crate::{Addon, AddonList, HEMTTError, Project, Report, Stage, Task};

#[derive(Clone)]
pub struct Release {}
impl Task for Release {
    fn single(&self, addons: Vec<Result<(Report, Addon), HEMTTError>>, p: &Project, _: &Stage) -> AddonList {
        let mut can_continue = true;
        let addons: Vec<_> = addons
            .into_iter()
            .map(|d| {
                if d.is_err() {
                    can_continue = false;
                    d
                } else {
                    let (report, addon) = d.unwrap();
                    if let Some((fatal, _)) = report.stop {
                        if fatal {
                            can_continue = false;
                            println!();
                            error!(&format!("Unable to build `{}`", addon.folder().display().to_string()))
                        }
                    }
                    Ok((report, addon))
                }
            })
            .collect();

        if !can_continue && (*crate::CI || !Confirmation::new().with_text("Do you want to continue?").interact()?) {
            return Err(HEMTTError::generic(
                "Unable to release",
                "One or more addons were not built successfully",
            ));
        }

        // Prepare release directory
        let release_folder = p.release_dir()?;
        if release_folder.exists() {
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
            }
        }

        create_dir!(release_folder)?;

        for dir in &["keys"] {
            create_dir!({
                let mut d = release_folder.clone();
                d.push(dir.to_string());
                d
            })?;
        }

        for file in &p.files {
            for entry in glob(file)? {
                if let Ok(path) = entry {
                    copy_file!(path, {
                        let mut d = release_folder.clone();
                        d.push(path.file_name().unwrap().to_str().unwrap().to_owned());
                        d
                    })?;
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
