use std::{
    io::{Read, Seek, Write},
    path::Path,
};

use crate::{BOM_FOLDERS, Error, should_check_bom};

#[derive(clap::Parser)]
/// Convert UTF-8 with BOM to UTF-8 without BOM
///
/// ## About BOM
///
/// A Byte Order Mark (BOM) is an invisible character at the start of text files
/// that can cause issues with Arma 3's config and script parsers.
///
/// This utility:
/// - Scans your project's `addons`, `optionals` and `include` folders for files with BOM markers
///   (or the current folder, if none of them exist)
/// - Removes them from supported file types (sqf, hpp, cpp, inc, lip, etc.)
/// - Reports how many files were modified
///
/// BOM markers are often added by Windows text editors. Run this utility if you
/// encounter unexpected parsing errors or as part of project maintenance.
pub struct Command {}

/// Execute the bom command
///
/// # Errors
/// [`Error`] depending on the modules
///
/// # Panics
/// If the args are not present from clap
pub fn execute(_: &Command) -> Result<(), Error> {
    let count = remove_bom_in_project(Path::new("."))?;
    info!("Removed BOM from {} files", count);
    Ok(())
}

/// Remove the UTF-8 BOM from the same project folders the BOM check uses
///
/// Falls back to all of `root` when none of those folders exist,
/// so the util still works outside of a HEMTT project
///
/// # Errors
/// [`Error::Io`] if a file cannot be read or written
pub fn remove_bom_in_project(root: &Path) -> Result<usize, Error> {
    let folders = BOM_FOLDERS
        .iter()
        .map(|folder| root.join(folder))
        .filter(|folder| folder.is_dir())
        .collect::<Vec<_>>();
    if folders.is_empty() {
        return remove_bom(root);
    }
    let mut count = 0;
    for folder in folders {
        count += remove_bom(&folder)?;
    }
    Ok(count)
}

/// Remove the UTF-8 BOM from all supported files under `root`
///
/// Returns the number of files that were modified
///
/// # Errors
/// [`Error::Io`] if a file cannot be read or written
pub fn remove_bom(root: &Path) -> Result<usize, Error> {
    let mut count = 0;
    // Skip hidden directories such as `.git` and `.hemttout`
    for entry in walkdir::WalkDir::new(root)
        .into_iter()
        .filter_entry(|e| e.depth() == 0 || !e.file_name().to_string_lossy().starts_with('.'))
    {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if !should_check_bom(path) {
            debug!("Skipping {}", path.display());
            continue;
        }
        let mut file = fs_err::OpenOptions::new()
            .read(true)
            .write(true)
            .open(path)?;
        let mut buf = [0; 3];
        if file.read_exact(&mut buf).is_err() {
            continue;
        }
        if buf == [0xEF, 0xBB, 0xBF] {
            let mut buf = Vec::new();
            file.read_to_end(&mut buf)?;
            file.seek(std::io::SeekFrom::Start(0))?;
            if let Err(e) = file.write_all(&buf) {
                error!("Failed to write to {}: {}", path.display(), e);
                continue;
            }
            file.set_len(buf.len() as u64)?;
            info!("Removed BOM from {}", path.display());
            count += 1;
        }
    }
    Ok(count)
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::remove_bom_in_project;

    const BOM: [u8; 3] = [0xEF, 0xBB, 0xBF];

    fn write_bom(path: &str) {
        let path = Path::new(path);
        fs_err::create_dir_all(path.parent().expect("parent")).expect("create dirs");
        let mut content = BOM.to_vec();
        content.extend_from_slice(b"content\n");
        fs_err::write(path, content).expect("write file");
    }

    fn has_bom(path: &str) -> bool {
        fs_err::read(path).expect("read").starts_with(&BOM)
    }

    #[test]
    fn only_project_folders() {
        let _directory = hemtt_test::directory::TemporaryDirectory::new();
        for file in [
            "addons/main/script.sqf",
            "optionals/extra/config.cpp",
            "include/a3/macros.hpp",
            "tools/generated.sqf",
            ".hemttout/dev/addons/main/script.sqf",
            ".git/hooks/fake.sqf",
        ] {
            write_bom(file);
        }

        assert_eq!(remove_bom_in_project(Path::new(".")).expect("remove"), 3);
        assert!(!has_bom("addons/main/script.sqf"));
        assert!(!has_bom("optionals/extra/config.cpp"));
        assert!(!has_bom("include/a3/macros.hpp"));
        // outside of the folders the BOM check looks at
        assert!(has_bom("tools/generated.sqf"));
        assert!(has_bom(".hemttout/dev/addons/main/script.sqf"));
        assert!(has_bom(".git/hooks/fake.sqf"));
    }

    #[test]
    fn outside_a_project() {
        let _directory = hemtt_test::directory::TemporaryDirectory::new();
        write_bom("loose/script.sqf");
        write_bom(".hidden/script.sqf");

        assert_eq!(remove_bom_in_project(Path::new(".")).expect("remove"), 1);
        assert!(!has_bom("loose/script.sqf"));
        assert!(has_bom(".hidden/script.sqf"));
    }
}
