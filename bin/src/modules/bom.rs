use std::{
    io::Read,
    path::{Path, PathBuf},
    sync::Arc,
};

use hemtt_workspace::reporting::Code;

use crate::{context::Context, report::Report, should_check_bom};

use super::Module;

#[derive(Default)]
pub struct BOMCheck {}

/// Find all files under `root` that start with a UTF-8 BOM
///
/// Only files that `hemtt utils bom` is able to fix are considered
///
/// # Errors
/// [`crate::Error::Io`] if a file cannot be read
pub fn files_with_bom(root: &Path) -> Result<Vec<PathBuf>, crate::Error> {
    let mut found = Vec::new();
    for entry in walkdir::WalkDir::new(root)
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|e| e.file_type().is_file())
        .filter(|e| !e.path().display().to_string().contains(".hemttout"))
        .filter(|e| should_check_bom(e.path()))
    {
        let mut buffer = [0; 3];
        let mut file = fs_err::File::open(entry.path())?;
        if file.read_exact(&mut buffer).is_err() {
            continue;
        }
        if buffer == [0xEF, 0xBB, 0xBF] {
            found.push(entry.path().to_path_buf());
        }
    }
    Ok(found)
}

impl Module for BOMCheck {
    fn name(&self) -> &'static str {
        "BOM Check"
    }

    fn check(&self, ctx: &Context) -> Result<Report, crate::Error> {
        let mut report = Report::new();
        for folder in crate::BOM_FOLDERS {
            let folder = ctx.project_folder().join(folder);
            if !folder.exists() {
                continue;
            }
            for path in files_with_bom(&folder)? {
                report.push(Arc::new(BOMError {
                    file: {
                        path.display()
                            .to_string()
                            .strip_prefix(&ctx.project_folder().display().to_string())
                            .unwrap_or_default()
                            .to_string()
                    },
                }));
            }
        }
        Ok(report)
    }
}

struct BOMError {
    file: String,
}

impl Code for BOMError {
    fn ident(&self) -> &'static str {
        "BOM"
    }

    fn message(&self) -> String {
        format!("File `{}` has a BOM marker", self.file)
    }

    fn help(&self) -> Option<String> {
        Some("Run `hemtt utils bom` to remove BOM markers".to_string())
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::files_with_bom;

    const BOM: [u8; 3] = [0xEF, 0xBB, 0xBF];

    fn write_bom(path: &Path) {
        fs_err::create_dir_all(path.parent().expect("parent")).expect("create dirs");
        let mut content = BOM.to_vec();
        content.extend_from_slice(b"content\n");
        fs_err::write(path, content).expect("write file");
    }

    #[test]
    fn util_fixes_everything_check_reports() {
        let _directory = hemtt_test::directory::TemporaryDirectory::new();
        let root = std::env::current_dir().expect("current dir");
        for file in [
            "addons/main/script.sqf",
            "addons/main/defines.inc",
            "addons/main/sound/voice.lip",
            "addons/main/websource/app.csproj.nuget.g.props",
            "addons/main/data/texture.paa",
            "addons/main/README",
        ] {
            write_bom(&root.join(file));
        }

        let mut found = files_with_bom(&root.join("addons")).expect("check");
        found.sort();
        let names = found
            .iter()
            .map(|p| p.file_name().expect("name").to_string_lossy().to_string())
            .collect::<Vec<_>>();
        assert_eq!(names, ["defines.inc", "script.sqf", "voice.lip"]);

        let fixed = crate::utils::bom::remove_bom(Path::new(".")).expect("fix");
        assert_eq!(fixed, 3);
        assert!(
            files_with_bom(&root.join("addons"))
                .expect("check")
                .is_empty(),
            "files reported by the check were not fixed by the util"
        );
        assert_eq!(
            fs_err::read(root.join("addons/main/script.sqf")).expect("read"),
            b"content\n"
        );
        // files the check ignores are left alone
        assert!(
            fs_err::read(root.join("addons/main/data/texture.paa"))
                .expect("read")
                .starts_with(&BOM)
        );
    }
}
