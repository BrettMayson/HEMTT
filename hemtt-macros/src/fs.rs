#[macro_export]
macro_rules! create_dir {
    ($e:expr) => {
        std::fs::create_dir_all(&$e).map_err(|source| {
            hemtt::HEMTTError::IOPath(hemtt::IOPathError {
                source,
                path: std::path::Path::new(&$e.clone()).to_path_buf(),
            })
        })
    };
}

#[macro_export]
macro_rules! open_file {
    ($e:expr) => {
        std::fs::File::open(&$e).map_err(|source| {
            hemtt::HEMTTError::IOPath(hemtt::IOPathError {
                path: std::path::PathBuf::from(&$e),
                source,
            })
        })
    };
}

#[macro_export]
macro_rules! create_file {
    ($e:expr) => {{
        let p = $e;
        std::fs::File::create(&p).map_err(|source| {
            hemtt::HEMTTError::IOPath(hemtt::IOPathError {
                path: std::path::PathBuf::from(&p),
                source,
            })
        })
    }};
}

#[macro_export]
macro_rules! copy_file {
    ($s:expr, $d:expr) => {
        std::fs::copy(&$s, &$d).map_err(|source| {
            hemtt::HEMTTError::GENERIC(
                format!("Unable to copy file: {}", source),
                format!("`{:#?}` => `{:#?}`", $s, $d),
            )
        })
    };
}

#[macro_export]
macro_rules! copy_dir {
    ($s:expr, $d:expr) => {
        fs_extra::dir::copy(&$s, &$d, &fs_extra::dir::CopyOptions::new()).map_err(|source| {
            hemtt::HEMTTError::GENERIC(
                format!("Unable to copy directory: {}", source),
                format!("`{:#?}` => `{:#?}`", $s, $d),
            )
        })
    };
}

#[macro_export]
macro_rules! rename_file {
    ($s:expr, $d:expr) => {
        std::fs::rename(&$s, &$d).map_err(|source| {
            hemtt::HEMTTError::GENERIC(
                format!("Unable to rename file: {}", source),
                format!("`{:#?}` => `{:#?}`", $s, $d),
            )
        })
    };
}

#[macro_export]
macro_rules! remove_file {
    ($s:expr) => {
        std::fs::remove_file(&$s).map_err(|source| {
            hemtt::HEMTTError::IOPath(hemtt::IOPathError {
                path: std::path::PathBuf::from(&$s),
                source,
            })
        })
    };
}
