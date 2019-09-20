use colored::*;
use docopt::Docopt;
use pbr::ProgressBar;
use serde::Deserialize;
use walkdir;
use zip;

use std::fs::File;
use std::io::{Error, BufReader, BufWriter, copy};
use std::path::{Path};

use crate::{HEMTTError, Command, Project};

pub struct Zip {}
impl Command for Zip {
    fn register(&self) -> clap::App {
        clap::SubCommand::with_name("zip").about("Get translation info from `stringtable.xml` files")
            .arg(clap::Arg::with_name("name").help("Name of the archive").default_value(""))
    }

    fn run_project(&self, args: &clap::ArgMatches, mut p: Project) -> Result<(), HEMTTError> {
        archive(args.value_of("name").unwrap(), p)?
    }
}

pub fn archive(name: &str, mut p: Project) -> Result<(), HEMTTError> {
    let version = p.version()?;

    let release_dir = format!("releases/{}", version);

    let zipname = format!("{}.zip", match name {
        "" => format!("{}_{}", p.name.replace(" ", "_"), version)
        _ => v,
    });
    println!(" {} {}", "Archiving".white().bold(), zipname);

    let zipsubpath = format!("releases/{}", zipname);
    let zippath = Path::new(&zipsubpath);
    let file = BufWriter::new(create_file!(&zippath)?);

    let dir = walkdir::WalkDir::new(&release_dir);
    let mut zip = zip::ZipWriter::new(file);
    let options = zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    let mut pb = ProgressBar::new(walkdir::WalkDir::new(&release_dir).into_iter().count() as u64);
    pb.show_speed = false;
    pb.show_time_left = false;
    pb.set_width(Some(70));

    // Zip all files and folders in all subdirectories
    for entry in dir.into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        let name = path.strip_prefix(Path::new(&release_dir)).unwrap();

        pb.message(&format!("{} - ", path.file_name().unwrap().to_str().unwrap()));
        pb.tick();

        // Write file or directory explicitly
        // Some unzip tools unzip files with directory paths correctly, some do not!
        if path.is_file() {
            zip.start_file_from_path(name, options)?;

            let mut f = BufReader::new(open_file!(path)?);

            // Copy directly, without any buffer, as we have no use for the intermediate data
            copy(&mut f, &mut zip)?;
        } else if !name.as_os_str().is_empty() {
            // Only if not root! Avoids path spec / warning
            // and mapname conversion failed error on unzip
            zip.add_directory_from_path(name, options)?;
        }

        pb.inc();
    }
    zip.finish()?;
    pb.finish_print(&format!(" {}  {}{}", "Archived".white().bold(), zipname, crate::repeat!(" ", 55)));
    println!();

    Ok(())
}
