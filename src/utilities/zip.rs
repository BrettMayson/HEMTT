use colored::*;
use docopt::Docopt;
use pbr::ProgressBar;
use serde::Deserialize;
use walkdir;
use zip;

use std::fs::File;
use std::io::{Read, Write, Error};
use std::iter::repeat;
use std::path::{Path};

use crate::error::*;

const USAGE: &'static str = "
Usage: zip [<name>]
";

#[derive(Debug, Deserialize)]
struct Args {
    arg_name: Option<String>,
}

pub fn archive(usage: &Vec<String>) -> Result<(), Error> {
    let p = crate::project::get_project()?;
    let version = p.version.unwrap();

    let release_dir = format!("releases/{}", version);

    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.argv(usage.into_iter()).deserialize())
        .unwrap_or_else(|e| e.exit());

    let zipname = format!("{}.zip", match args.arg_name {
        Some(v) => v,
        None => format!("{}_{}", p.name.replace(" ", "_"), version)
    });
    println!(" {} {}", "Archiving".white().bold(), zipname);

    let zipsubpath = format!("releases/{}", zipname);
    let zippath = Path::new(&zipsubpath);
    let file = File::create(&zippath).unwrap_or_print();

    let dir = walkdir::WalkDir::new(&release_dir);
    let mut zip = zip::ZipWriter::new(file);
    let options = zip::write::FileOptions::default();

    let mut pb = ProgressBar::new(walkdir::WalkDir::new(&release_dir).into_iter().count() as u64);
    pb.show_speed = false;
    pb.show_time_left = false;
    pb.set_width(Some(70));

    // Zip all files and folders in all subdirectories
    let mut buffer = Vec::new();
    for entry in dir.into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();

        let name = path.strip_prefix(Path::new(&release_dir))
            .unwrap().to_str().unwrap();

        pb.message(&format!("{} - ", path.file_name().unwrap().to_str().unwrap()));
        pb.tick();

        if path.is_file() {
            zip.start_file(name, options)?;
            let mut f = File::open(path)?;

            f.read_to_end(&mut buffer)?;
            zip.write_all(&*buffer)?;
            buffer.clear();
        } else {
            zip.add_directory(name, options)?;
        }

        pb.inc();
    }
    zip.finish()?;
    pb.finish_print(&format!(" {}  {}{}", "Archived".white().bold(), zipname, crate::repeat!(" ", 55)));
    println!();

    Ok(())
}
