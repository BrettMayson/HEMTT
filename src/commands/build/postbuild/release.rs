use dialoguer::Confirmation;

use crate::{Addon, AddonList, HEMTTError, Project, Report, Task};

#[derive(Clone)]
pub struct Release {}
impl Task for Release {
    fn can_run(&self, _: &Addon, _: &Report, _: &Project) -> Result<bool, HEMTTError> {
        Ok(true)
    }

    fn single(&self, addons: Vec<Result<(Report, Addon), HEMTTError>>, p: &Project) -> AddonList {
        let mut can_continue = true;
        /*for addon in &addons {
            if addon.is_err() { can_continue = false; break; }
            let (report, addon) = addon.unwrap();
            if report.stop.is_some() { can_continue = false; break; }
        }*/
        let addons = addons
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
        std::fs::create_dir_all(release_folder)?;

        Ok(addons)
    }
}
