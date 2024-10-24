use std::io::BufReader;

use clap::ArgMatches;
use hemtt_stringtable::Project;

use crate::{context::Context, report::Report, Error};

pub fn sort(matches: &ArgMatches) -> Result<Report, Error> {
    let ctx = Context::new(None, crate::context::PreservePrevious::Remove, true)?;

    let only_lang = matches.get_flag("only-lang");

    for addon in ctx.addons() {
        let stringtable_path = ctx
            .workspace_path()
            .join(addon.folder())?
            .join("stringtable.xml")?;
        if stringtable_path.exists()? {
            match Project::from_reader(BufReader::new(stringtable_path.open_file()?)) {
                Ok(mut project) => {
                    if !only_lang {
                        project.sort();
                    }
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
