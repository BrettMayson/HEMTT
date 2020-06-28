use armake2::ArmakeError;

pub trait PrintableError<T, E> {
    fn unwrap_or_print(self) -> T;
}
impl<T, E: std::fmt::Debug + std::fmt::Display> PrintableError<T, E> for Result<T, E> {
    fn unwrap_or_print(self) -> T {
        if let Err(error) = &self {
            error!("{}", error);
            std::process::exit(1);
        }
        self.unwrap()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FileErrorLineNumber {
    pub file: String,
    pub content: String,
    pub error: String,
    pub line: Option<usize>,
    pub col: Option<usize>,
    pub note: Option<String>,
}

#[derive(Debug)]
pub struct IOPathError {
    pub source: std::io::Error,
    pub path: std::path::PathBuf,
}

#[derive(Debug)]
pub enum HEMTTError {
    GENERIC(String, String),
    IO(std::io::Error),
    LINENO(FileErrorLineNumber),
    PATH(IOPathError),
    SIMPLE(String),
    TOML(toml::ser::Error),
}

impl HEMTTError {
    // pub fn from_armake_parse(
    //     err: armake2::config::config_grammar::ParseError,
    //     path: &str,
    //     content: Option<String>,
    // ) -> Self {
    //     let c = match content {
    //         Some(v) => v.lines().nth(err.line - 1).unwrap().to_string(),
    //         None => crate::CACHED.lock().unwrap().get_line(path, err.line).unwrap(),
    //     };
    //     Self::LINENO(FileErrorLineNumber {
    //         line: Some(err.line),
    //         col: Some(err.column),
    //         error: format!("Expected one of `{}`", {
    //             let mut v = err.expected.into_iter().collect::<Vec<&str>>();
    //             v.sort();
    //             v.join("` `")
    //         }),
    //         content: c,
    //         file: path.to_string(),
    //         note: None,
    //     })
    // }

    pub fn generic<T: Into<String>, U: Into<String>>(msg: T, info: U) -> Self {
        Self::GENERIC(msg.into(), info.into())
    }
    pub fn simple<T: Into<String>>(msg: T) -> Self {
        Self::SIMPLE(msg.into())
    }
}

impl std::fmt::Display for HEMTTError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Self::GENERIC(ref s, ref v) => write!(f, "{}: {}", s, v),
            Self::IO(ref err) => write!(f, "IO error: {}", err),
            // Self::LINENO(ref err) => write!(f, "{}\n{}", err.error, filepointer!(err)),
            Self::LINENO(ref err) => write!(f, "{}", err.error),
            Self::PATH(ref err) => write!(f, "IO error {}: {}", err.path.display(), err.source),
            Self::SIMPLE(ref s) => write!(f, "{}", s),
            Self::TOML(ref err) => write!(f, "TOML error: {}", err),
        }
    }
}

impl std::error::Error for HEMTTError {
    fn cause(&self) -> Option<&dyn std::error::Error> {
        match *self {
            Self::GENERIC(ref _s, ref _v) => Some(self),
            Self::IO(ref err) => Some(err),
            Self::LINENO(ref _e) => Some(self),
            Self::PATH(ref _err) => Some(self),
            Self::SIMPLE(ref _s) => Some(self),
            Self::TOML(ref err) => Some(err),
        }
    }
}

impl From<std::io::Error> for HEMTTError {
    fn from(err: std::io::Error) -> Self {
        Self::IO(err)
    }
}

impl From<std::string::FromUtf8Error> for HEMTTError {
    fn from(_: std::string::FromUtf8Error) -> Self {
        Self::SIMPLE("Unable to convert UTF-8 to string".to_string())
    }
}

impl From<std::num::ParseIntError> for HEMTTError {
    fn from(err: std::num::ParseIntError) -> Self {
        Self::GENERIC("Unable to parse integer".to_owned(), err.to_string())
    }
}

impl From<toml::ser::Error> for HEMTTError {
    fn from(err: toml::ser::Error) -> Self {
        Self::TOML(err)
    }
}

impl From<config::ConfigError> for HEMTTError {
    fn from(err: config::ConfigError) -> Self {
        let s = "Unable to open project config".to_string();
        match err {
            config::ConfigError::Frozen => {
                Self::GENERIC(s, "Config is frozen and no further mutations can be made".to_string())
            }
            config::ConfigError::NotFound(v) => {
                Self::GENERIC(s, format!("The property `{}` is required but wasn't found", v))
            }
            config::ConfigError::PathParse(e) => Self::GENERIC(s, e.description().to_string()),
            config::ConfigError::Message(v) => Self::GENERIC(s, v),
            config::ConfigError::FileParse { .. } => Self::GENERIC(s, "The file could not be parsed".to_string()),
            _ => Self::GENERIC(s, err.to_string()),
        }
    }
}

impl From<handlebars::TemplateRenderError> for HEMTTError {
    fn from(err: handlebars::TemplateRenderError) -> Self {
        match err {
            handlebars::TemplateRenderError::RenderError(e) => {
                if e.line_no.is_some() {
                    Self::LINENO(FileErrorLineNumber {
                        error: e.desc,
                        line: e.line_no,
                        col: e.column_no,
                        note: None,
                        file: "".to_string(),
                        content: "".to_string(),
                    })
                } else {
                    Self::GENERIC("Render error".to_string(), e.desc)
                }
            }
            handlebars::TemplateRenderError::TemplateError(e) => {
                if e.line_no.is_some() {
                    Self::LINENO(FileErrorLineNumber {
                        error: e.reason.to_string(),
                        line: e.line_no,
                        col: e.column_no,
                        note: None,
                        file: "".to_string(),
                        content: "".to_string(),
                    })
                } else {
                    Self::GENERIC("Render error".to_string(), e.reason.to_string())
                }
            }
            _ => unimplemented!(),
        }
    }
}

impl From<armake2::ArmakeError> for HEMTTError {
    fn from(err: armake2::ArmakeError) -> Self {
        // println!("\n\n\n\nconfig error: {:?}\n\n\n\n\n", err);
        // Self::LINENO(FileErrorLineNumber {
        //     line: Some(err.line),
        //     col: Some(err.column),
        //     error: format!(
        //         "Expected one of {}",
        //         err.expected.into_iter().collect::<Vec<&str>>().join(", ")
        //     ),
        //     content: "Unknown content".to_string(),
        //     file: "Unknown file".to_string(),
        //     note: None,
        // })
        match err {
            ArmakeError::GENERIC(s) => Self::SIMPLE(s),
            ArmakeError::CONFIG(c) => Self::GENERIC(
                if let Some(p) = c.path {
                    format!("Failed to parse config `{}`", p)
                } else {
                    "Failed to parse config".to_owned()
                },
                c.source.to_string(),
            ),
            ArmakeError::PARSE(c) => Self::GENERIC(
                if let Some(p) = c.path {
                    format!("Unable to parse `{}`", p)
                } else {
                    "Unable to parse".to_owned()
                },
                c.source.to_string(),
            ),
            ArmakeError::PREPROCESS(c) => Self::GENERIC(
                if let Some(p) = c.path {
                    format!("Unable to preprocess `{}`", p)
                } else {
                    "Unable to preprocess".to_owned()
                },
                c.source.to_string(),
            ),
            ArmakeError::IO(e) => Self::IO(e),
            ArmakeError::IOPath(e) => Self::PATH(IOPathError {
                source: e.source,
                path: e.path,
            }),
        }
    }
}

impl From<glob::PatternError> for HEMTTError {
    fn from(err: glob::PatternError) -> Self {
        Self::GENERIC("GLOB Pattern Error".to_owned(), err.msg.to_owned())
    }
}

impl From<zip::result::ZipError> for HEMTTError {
    fn from(err: zip::result::ZipError) -> Self {
        match err {
            zip::result::ZipError::FileNotFound => Self::SIMPLE("Unable to add file to zip, it does not exist".to_owned()),
            zip::result::ZipError::Io(e) => Self::IO(e),
            zip::result::ZipError::UnsupportedArchive(e) => Self::GENERIC("Unsupported archive".to_owned(), e.to_owned()),
            zip::result::ZipError::InvalidArchive(e) => Self::GENERIC("Invalid archive".to_owned(), e.to_owned()),
        }
    }
}
