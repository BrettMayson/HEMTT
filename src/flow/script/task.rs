#[derive(Clone)]
pub struct ModTime {}
impl Task for ModTime {
    fn can_run(&self, _: &Addon, _: &Report, _: &Project) -> Result<bool, HEMTTError> {
        Ok(true)
    }

    fn parallel(&self, addon: &Addon, _: &Report, p: &Project, _pb: &ProgressBar) -> Result<Report, HEMTTError> {
        let mut report = Report::new();
        let modified = modtime(&addon.folder())?;
        let target = addon.target(p);
        if target.exists() {
            if let Ok(time) = std::fs::metadata(&target).unwrap().modified() {
                if time >= modified {
                    report.stop = Some((
                        false,
                        HEMTTError::GENERIC("The PBO already exists".to_owned(), target.display().to_string()),
                    ));
                }
            }
        }
        Ok(report)
    }
}
