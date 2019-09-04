#[macro_export]
macro_rules! create_dir {
    ($e:expr) => {
        std::fs::create_dir_all(&$e).map_err(|source| crate::HEMTTError::PATH(crate::IOPathError {
            source,
            path: std::path::Path::new(&$e.clone()).to_path_buf(),
        }))
    };
}

#[macro_export]
macro_rules! open_file {
    ($e:expr) => {
        File::open(&$e).map_err(|source| {
            HEMTTError::PATH(crate::IOPathError {
                path: PathBuf::from(&$e),
                source,
            })
        })
    };
}
