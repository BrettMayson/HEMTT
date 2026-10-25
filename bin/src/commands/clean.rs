use std::path::Path;

use crate::{Error, context::project_tmp, report::Report};

#[derive(clap::Parser)]
#[command(verbatim_doc_comment)]
/// Remove the `.hemttout` folder and temporary files for the current project
///
/// Symlinks are counted as a single file and are not followed.
pub struct Command {}

/// Files and bytes removed for one category of output
struct Usage {
    label: String,
    files: u64,
    bytes: u64,
}

/// Execute the clean command
///
/// # Errors
/// [`Error::ConfigNotFound`] if the current directory is not a HEMTT project
/// [`Error::Io`] if a folder cannot be read or removed
pub fn execute(_: &Command) -> Result<Report, Error> {
    let root = std::env::current_dir()?;
    if !root.join(".hemtt").exists() {
        return Err(Error::ConfigNotFound);
    }

    let usages = clean(&root)?;
    if usages.is_empty() {
        println!("Nothing to clean");
        return Ok(Report::new());
    }

    let mut total_files = 0;
    let mut total_bytes = 0;
    for usage in &usages {
        println!(
            "{:>12} {} ({}, {})",
            "Removed",
            usage.label,
            files(usage.files),
            human_size(usage.bytes)
        );
        total_files += usage.files;
        total_bytes += usage.bytes;
    }
    println!(
        "{:>12} {}, {} total",
        "Removed",
        files(total_files),
        human_size(total_bytes)
    );

    Ok(Report::new())
}

/// Measure, then remove, the `.hemttout` and temporary folders for the project at `root`
///
/// Returns the categories removed, largest first
///
/// # Errors
/// [`Error::Io`] if a folder cannot be read or removed
fn clean(root: &Path) -> Result<Vec<Usage>, Error> {
    let out = root.join(".hemttout");
    let tmp = project_tmp(root);
    let mut usages = Vec::new();

    if out.exists() {
        for entry in fs_err::read_dir(&out)? {
            let entry = entry?;
            let name = entry.file_name().to_string_lossy().into_owned();
            record(&mut usages, out_label(&name), &entry.path())?;
        }
    }
    if tmp.exists() {
        record(&mut usages, "binarize cache", &tmp)?;
    }

    for path in [&out, &tmp] {
        if path.exists() {
            fs_err::remove_dir_all(path)?;
        }
    }

    usages.sort_by_key(|usage| std::cmp::Reverse(usage.bytes));
    Ok(usages)
}

/// The category a top-level entry of `.hemttout` belongs to
fn out_label(name: &str) -> &'static str {
    match name {
        "paacache" => "PAA cache",
        "bincache" | "binarize.hcb" => "binarize cache",
        _ => "build output",
    }
}

/// Add the files and bytes under `path` to the category `label`
fn record(usages: &mut Vec<Usage>, label: &str, path: &Path) -> Result<(), Error> {
    let (files, bytes) = measure(path)?;
    match usages.iter_mut().find(|usage| usage.label == label) {
        Some(usage) => {
            usage.files += files;
            usage.bytes += bytes;
        }
        None => usages.push(Usage {
            label: label.to_owned(),
            files,
            bytes,
        }),
    }
    Ok(())
}

/// Count the files and bytes under `path`, without following symlinks
fn measure(path: &Path) -> Result<(u64, u64), Error> {
    let metadata = fs_err::symlink_metadata(path)?;
    if !metadata.is_dir() {
        return Ok((1, metadata.len()));
    }
    let mut totals = (0, 0);
    for entry in fs_err::read_dir(path)? {
        let (files, bytes) = measure(&entry?.path())?;
        totals.0 += files;
        totals.1 += bytes;
    }
    Ok(totals)
}

/// Format a byte count in binary units, matching `cargo clean`
fn human_size(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KiB", "MiB", "GiB", "TiB"];
    let mut unit = 0;
    let mut divisor: u64 = 1;
    while unit < UNITS.len() - 1 && bytes / divisor >= 1024 {
        divisor *= 1024;
        unit += 1;
    }
    if unit == 0 {
        return format!("{bytes}B");
    }
    let tenths = (bytes % divisor) * 10 / divisor;
    format!("{}.{tenths}{}", bytes / divisor, UNITS[unit])
}

/// Format a file count, singular for one
fn files(count: u64) -> String {
    if count == 1 {
        String::from("1 file")
    } else {
        format!("{count} files")
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use clap::Parser as _;

    use super::{clean, files, human_size};
    use crate::context::project_tmp;

    fn write(path: &Path, len: usize) {
        fs_err::create_dir_all(path.parent().expect("parent")).expect("create dirs");
        fs_err::write(path, vec![0_u8; len]).expect("write file");
    }

    #[test]
    fn removes_out_and_tmp() {
        let _directory =
            hemtt_test::directory::TemporaryDirectory::copy(&std::path::PathBuf::from(format!(
                "{}/tests/workspace_simple",
                env!("CARGO_MANIFEST_DIR")
            )));
        let root = std::env::current_dir().expect("current dir");
        fs_err::create_dir_all(root.join(".hemttout/dev")).expect("create .hemttout");
        fs_err::create_dir_all(project_tmp(&root)).expect("create project tmp");

        crate::execute(&crate::Cli::parse_from(vec!["hemtt", "clean"])).expect("clean");

        assert!(!root.join(".hemttout").exists());
        assert!(!project_tmp(&root).exists());
        assert!(root.join(".hemtt").exists());
    }

    #[test]
    fn reports_usage_by_category() {
        let _directory =
            hemtt_test::directory::TemporaryDirectory::copy(&std::path::PathBuf::from(format!(
                "{}/tests/workspace_simple",
                env!("CARGO_MANIFEST_DIR")
            )));
        let root = std::env::current_dir().expect("current dir");
        write(&root.join(".hemttout/bincache/a.bin"), 100);
        write(&root.join(".hemttout/binarize.hcb"), 50);
        write(&root.join(".hemttout/paacache/x.paa"), 200);
        write(&root.join(".hemttout/dev/y"), 10);
        write(&root.join(".hemttout/latest.log"), 5);
        write(&project_tmp(&root).join("hemtt_binarize_output/z"), 30);

        let usages = clean(&root).expect("clean");

        let find = |label: &str| {
            usages
                .iter()
                .find(|usage| usage.label == label)
                .map(|usage| (usage.files, usage.bytes))
        };
        assert_eq!(find("PAA cache"), Some((1, 200)));
        assert_eq!(find("binarize cache"), Some((3, 180)));
        assert_eq!(find("build output"), Some((2, 15)));
        assert_eq!(usages[0].label, "PAA cache");
        assert!(!root.join(".hemttout").exists());
        assert!(!project_tmp(&root).exists());
    }

    #[test]
    fn formats_sizes() {
        assert_eq!(human_size(0), "0B");
        assert_eq!(human_size(1536), "1.5KiB");
        assert_eq!(human_size(1023), "1023B");
        assert_eq!(human_size(38 * 1024 * 1024 * 1024), "38.0GiB");
        assert_eq!(files(1), "1 file");
        assert_eq!(files(2), "2 files");
    }
}
