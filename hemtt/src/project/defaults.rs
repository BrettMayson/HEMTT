use std::path::PathBuf;

pub fn default_include() -> Vec<PathBuf> {
    let mut includes = vec![];

    if PathBuf::from("./include").exists() {
        includes.push(PathBuf::from("./include"));
    }

    includes
}

pub fn default_version() -> semver::Version {
    semver::Version::new(0, 1, 0)
}

pub fn default_mainprefix() -> String {
    String::from("z")
}

pub const fn default_reuse_private_key() -> Option<bool> {
    None
}

pub const fn default_folder_optionals() -> Option<bool> {
    Some(true)
}

pub const fn default_sig_version() -> u32 {
    3
}
