use zip;
use walkdir;

use colored::*;

use std::fs::File;
use std::io::{Read, Write, Error};
use std::path::{Path};

pub fn archive() -> Result<(), Error> {
    let p = crate::project::get_project()?;
    let version = p.version.unwrap();

    let release_dir = format!("releases/{}", version);

    let zipname = format!("{}_{}.zip", p.name.replace(" ", "_"), version);
    println!(" {} {}", "Archiving".white().bold(), zipname);

    let zipsubpath = format!("releases/{}", zipname);
    let zippath = Path::new(&zipsubpath);
    let file = File::create(&zippath).unwrap();

    let walkdir = walkdir::WalkDir::new(&release_dir);
    let mut zip = zip::ZipWriter::new(file);
    let options = zip::write::FileOptions::default();

    // Zip all files and folders in all subdirectories
    let mut buffer = Vec::new();
    for entry in walkdir.into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();

        let name = path.strip_prefix(Path::new(&release_dir))
            .unwrap().to_str().unwrap();

        if path.is_file() {
            zip.start_file(name, options)?;
            let mut f = File::open(path)?;

            f.read_to_end(&mut buffer)?;
            zip.write_all(&*buffer)?;
            buffer.clear();
        }
    }
    zip.finish()?;

    Ok(())
}
