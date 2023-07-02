use std::{collections::HashMap, path::PathBuf};

use vfs::VfsPath;

#[allow(clippy::module_name_repetitions)]
/// A resolver that can find includes
pub struct Resolver<'a> {
    vfs: &'a VfsPath,
    prefixes: HashMap<String, VfsPath>,
}

impl<'a> Resolver<'a> {
    #[must_use]
    /// Create a new resolver
    pub const fn new(vfs: &'a VfsPath, prefixes: HashMap<String, VfsPath>) -> Self {
        Self { vfs, prefixes }
    }

    #[must_use]
    /// Find an include
    pub fn find_include(&self, from: &str, to: &str) -> Option<(PathBuf, String)> {
        println!("find_include: {} {}", from, to);
        let path = if to.starts_with('\\') {
            let to = to.trim_start_matches('\\');
            if let Some(path) = self
                .prefixes
                .iter()
                .find(|(prefix, _)| {
                    let prefix = prefix.trim_start_matches('\\');
                    to.starts_with(&{
                        let mut prefix = prefix.to_string();
                        prefix.push('\\');
                        prefix
                    })
                })
                .map(|(prefix, path)| {
                    let mut path = PathBuf::from(path.as_str());
                    path.push(
                        to.strip_prefix(prefix)
                            .unwrap()
                            .trim_start_matches('\\')
                            .replace('\\', "/"),
                    );
                    path
                })
            {
                path
            } else {
                let include =
                    PathBuf::from("include").join(to.trim_start_matches('\\').replace('\\', "/"));
                if include.exists() {
                    include
                } else {
                    return None;
                }
            }
        } else {
            let mut path = PathBuf::from(from).parent().unwrap().to_path_buf();
            path.push(to.replace('\\', "/"));
            path
        };
        self.vfs
            .join(path.display().to_string().trim_start_matches('/'))
            .unwrap()
            .open_file()
            .map_or(None, |mut file| {
                let mut include_content = String::new();
                file.read_to_string(&mut include_content)
                    .expect("failed to read include");
                Some((path, include_content))
            })
    }
}
