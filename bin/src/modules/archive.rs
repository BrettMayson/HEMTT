use std::{
    fs::{create_dir_all, File},
    path::PathBuf,
};

use walkdir::WalkDir;
use zip::{write::SimpleFileOptions, ZipWriter};

use crate::{context::Context, error::Error, progress::progress_bar, report::Report};

enum Entry {
    File(String, PathBuf),
    Directory(String),
}

/// Creates the release zips
///
/// # Errors
/// [`Error`] depending on the modules
/// [`Error::Zip`] if the zip fails to create
/// [`Error::Io`] if the zip fails to write
/// [`Error::Version`] if the version is invalid
///
/// # Panics
/// If we are somehow not in the HEMTT folder
pub fn release(ctx: &Context) -> Result<Report, Error> {
    let output = ctx.project_folder().join("releases");
    trace!("using releases folder: {:?}", output.display());
    if !output.exists() {
        create_dir_all(&output)?;
    }
    let output = output
        .join(format!("{}-latest", ctx.config().prefix()))
        .with_extension("zip");
    let options = SimpleFileOptions::default().compression_level(Some(9));

    debug!("creating release at {:?}", output.display());
    let mut to_write = Vec::new();
    for entry in WalkDir::new(ctx.build_folder().expect("build folder exists")) {
        let Ok(entry) = entry else {
            continue;
        };
        let path = entry.path();
        if path.is_dir() {
            let path = path
                .strip_prefix(ctx.build_folder().expect("build folder exists"))
                .expect("We are in the HEMTT folder, the prefix should always exist")
                .display()
                .to_string();
            if path.is_empty() {
                continue;
            }
            let dir = format!(
                "@{}/{}",
                ctx.config().hemtt().release().folder(),
                path.replace('\\', "/")
            );
            trace!("zip: creating directory {:?}", dir);
            to_write.push(Entry::Directory(dir));
            continue;
        }
        let name = path
            .strip_prefix(ctx.build_folder().expect("build folder exists"))
            .expect("We are in the HEMTT folder, the prefix should always exist");
        let file = format!(
            "@{}/{}",
            ctx.config().hemtt().release().folder(),
            name.display().to_string().replace('\\', "/")
        );
        trace!("zip: adding file {:?}", file);
        to_write.push(Entry::File(file, path.to_owned()));
    }
    let progress = progress_bar(to_write.len() as u64).with_message("Creating release");
    let mut zip = ZipWriter::new(File::create(&output)?);
    for entry in to_write {
        match entry {
            Entry::File(file, path) => {
                zip.start_file(file, options)?;
                std::io::copy(&mut File::open(path)?, &mut zip)?;
            }
            Entry::Directory(dir) => {
                zip.add_directory(dir, options)?;
            }
        }
        progress.inc(1);
    }
    progress.finish_and_clear();
    zip.finish()?;
    info!("Created release: {}", output.display());
    std::fs::copy(&output, {
        let mut output = output.clone();
        output.set_file_name(format!(
            "{}-{}.zip",
            ctx.config().prefix(),
            ctx.config().version().get(ctx.workspace_path().vfs())?
        ));
        info!("Created release: {}", output.display());
        output
    })?;
    Ok(Report::new())
}
