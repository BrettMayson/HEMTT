pub trait PrintableError<T, E> {
    fn unwrap_or_print(self) -> T;
}
impl<T, E: std::fmt::Debug + std::fmt::Display> PrintableError<T, E> for Result<T, E> {
    fn unwrap_or_print(self) -> T {
        if let Err(error) = &self {
            println!("{}", error);
            std::process::exit(1);
        }
        self.unwrap()
    }
}

#[derive(Debug)]
pub struct IOPathError {
    pub source: std::io::Error,
    pub path: std::path::PathBuf,
    pub message: Option<String>,
}

#[derive(Debug)]
pub struct PreprocessParseError {
    pub path: Option<String>,
    pub message: String,
    pub source: crate::preprocess::grammar::ParseError,
}

#[derive(Debug)]
pub struct PreprocessError {
    pub path: Option<String>,
    pub message: String,
    pub source: Box<HEMTTError>,
}

#[derive(Debug)]
pub struct ConfigParseError {
    pub path: Option<String>,
    pub message: String,
    pub source: crate::config::grammar::ParseError,
}

#[derive(Debug)]
pub enum HEMTTError {
    GENERIC(String),
    CONFIG(ConfigParseError),
    PARSE(PreprocessParseError),
    PREPROCESS(PreprocessError),
    IO(std::io::Error),
    IOPath(IOPathError),
    SemVer(semver::SemVerError),
}

#[macro_export]
macro_rules! aerror {
    ($e:expr) => {
        HEMTTError::GENERIC($e.to_string())
    };
    ($e:expr, $($p:expr),*) => {
        aerror!(format!($e, $($p,)*))
    };
}

impl std::fmt::Display for HEMTTError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            HEMTTError::GENERIC(ref s) => write!(f, "{}", s),
            HEMTTError::CONFIG(ref e) => write!(f, "Config: {}", e.message),
            HEMTTError::PARSE(ref e) => write!(f, "Preprocessor Parse: {}", e.message),
            HEMTTError::PREPROCESS(ref e) => write!(f, "Preprocessor: {}", e.message),
            HEMTTError::IO(ref e) => write!(f, "IO error: {}", e),
            HEMTTError::IOPath(ref e) => write!(f, "IO error: `{:#?}`\n{}", e.path, e.source),
            HEMTTError::SemVer(ref e) => write!(f, "SemVer error: `{}`", e),
        }
    }
}

impl std::error::Error for HEMTTError {
    fn cause(&self) -> Option<&dyn std::error::Error> {
        match *self {
            HEMTTError::GENERIC(ref _s) => Some(self),
            HEMTTError::CONFIG(ref e) => Some(&e.source),
            HEMTTError::PARSE(ref e) => Some(&e.source),
            HEMTTError::PREPROCESS(ref e) => Some(&e.source),
            HEMTTError::IO(ref e) => Some(e),
            HEMTTError::IOPath(ref e) => Some(&e.source),
            HEMTTError::SemVer(ref e) => Some(e),
        }
    }
}

impl From<std::io::Error> for HEMTTError {
    fn from(err: std::io::Error) -> HEMTTError {
        HEMTTError::IO(err)
    }
}

impl From<semver::SemVerError> for HEMTTError {
    fn from(err: semver::SemVerError) -> Self {
        HEMTTError::SemVer(err)
    }
}
