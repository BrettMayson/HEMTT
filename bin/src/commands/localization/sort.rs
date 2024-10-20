use std::io::BufReader;

use hemtt_stringtable::Project;

use crate::{context::Context, report::Report, Error};

pub fn sort() -> Result<Report, Error> {
    let ctx = Context::new(None, crate::context::PreservePrevious::Remove, true)?;

    for addon in ctx.addons() {
        let stringtable_path = ctx
            .workspace_path()
            .join(addon.folder())?
            .join("stringtable.xml")?;
        if stringtable_path.exists()? {
            match Project::from_reader(BufReader::new(stringtable_path.open_file()?)) {
                Ok(mut project) => {
                    project.sort();
                    let out_path = ctx
                        .project_folder()
                        .join(addon.folder_pathbuf())
                        .join("stringtable.xml");
                    let mut writer = String::new();
                    if let Err(e) = project.to_writer(&mut writer) {
                        error!("Failed to write stringtable for {}", addon.folder());
                        error!("{:?}", e);
                        return Ok(Report::new());
                    }
                    if let Err(e) = std::fs::write(out_path, writer) {
                        error!("Failed to write stringtable for {}", addon.folder());
                        error!("{:?}", e);
                        return Ok(Report::new());
                    }
                }
                Err(e) => {
                    error!("Failed to read stringtable for {}", addon.folder());
                    error!("{:?}", e);
                    return Ok(Report::new());
                }
            };
        }
    }
    Ok(Report::new())
}
