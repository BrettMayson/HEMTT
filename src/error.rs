use colored::*;

pub trait PrintableError<T, E> {
    fn unwrap_or_print(self) -> T;
}
impl<T, E: std::fmt::Debug + std::fmt::Display> PrintableError<T, E> for Result<T, E> {
    fn unwrap_or_print(self) -> T {
        if let Err(error) = &self {
            error!(format!("{}", error));
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
    pub fn from_armake_parse(
        err: armake2::config::config_grammar::ParseError,
        path: &str,
        content: Option<String>,
    ) -> HEMTTError {
        let c = match content {
            Some(v) => v.lines().nth(err.line - 1).unwrap().to_string(),
            None => crate::CACHED.lock().unwrap().get_line(path, err.line).unwrap(),
        };
        HEMTTError::LINENO(FileErrorLineNumber {
            line: Some(err.line),
            col: Some(err.column),
            error: format!("Expected one of `{}`", {
                let mut v = err.expected.into_iter().collect::<Vec<&str>>();
                v.sort();
                v.join("` `")
            }),
            content: c,
            file: path.to_string(),
            note: None,
        })
    }

    pub fn generic<T: Into<String>, U: Into<String>>(msg: T, info: U) -> HEMTTError {
        HEMTTError::GENERIC(msg.into(), info.into())
    }
    pub fn simple<T: Into<String>>(msg: T) -> HEMTTError {
        HEMTTError::SIMPLE(msg.into())
    }
}

impl std::fmt::Display for HEMTTError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            HEMTTError::GENERIC(ref s, ref v) => write!(f, "{}\n    {}", s.bold(), v),
            HEMTTError::IO(ref err) => write!(f, "IO error: {}", err),
            HEMTTError::LINENO(ref err) => write!(f, "{}\n{}", err.error, filepointer!(err)),
            HEMTTError::PATH(ref err) => write!(f, "IO error {}: {}", err.path.display(), err.source),
            HEMTTError::SIMPLE(ref s) => write!(f, "{}", s),
            HEMTTError::TOML(ref err) => write!(f, "TOML error: {}", err),
        }
    }
}

impl std::error::Error for HEMTTError {
    fn cause(&self) -> Option<&dyn std::error::Error> {
        match *self {
            HEMTTError::GENERIC(ref _s, ref _v) => Some(self),
            HEMTTError::IO(ref err) => Some(err),
            HEMTTError::LINENO(ref _e) => Some(self),
            HEMTTError::PATH(ref _err) => Some(self),
            HEMTTError::SIMPLE(ref _s) => Some(self),
            HEMTTError::TOML(ref err) => Some(err),
        }
    }
}

impl From<std::io::Error> for HEMTTError {
    fn from(err: std::io::Error) -> HEMTTError {
        HEMTTError::IO(err)
    }
}

impl From<std::string::FromUtf8Error> for HEMTTError {
    fn from(_: std::string::FromUtf8Error) -> HEMTTError {
        HEMTTError::SIMPLE("Unable to convert UTF-8 to string".to_string())
    }
}

impl From<toml::ser::Error> for HEMTTError {
    fn from(err: toml::ser::Error) -> HEMTTError {
        HEMTTError::TOML(err)
    }
}

impl From<config::ConfigError> for HEMTTError {
    fn from(err: config::ConfigError) -> HEMTTError {
        let s = "Unable to open project config".to_string();
        match err {
            config::ConfigError::Frozen => {
                HEMTTError::GENERIC(s, "Config is frozen and no further mutations can be made".to_string())
            }
            config::ConfigError::NotFound(v) => {
                HEMTTError::GENERIC(s, format!("The property `{}` is required but wasn't found", v))
            }
            config::ConfigError::PathParse(e) => HEMTTError::GENERIC(s, e.description().to_string()),
            config::ConfigError::Message(v) => HEMTTError::GENERIC(s, v),
            config::ConfigError::FileParse { .. } => HEMTTError::GENERIC(s, "The file could not be parsed".to_string()),
            _ => HEMTTError::GENERIC(s, err.to_string()),
        }
    }
}

impl From<handlebars::TemplateRenderError> for HEMTTError {
    fn from(err: handlebars::TemplateRenderError) -> HEMTTError {
        match err {
            handlebars::TemplateRenderError::RenderError(e) => {
                if e.line_no.is_some() {
                    HEMTTError::LINENO(FileErrorLineNumber {
                        error: e.desc,
                        line: e.line_no,
                        col: e.column_no,
                        note: None,
                        file: "".to_string(),
                        content: "".to_string(),
                    })
                } else {
                    HEMTTError::GENERIC("Render error".to_string(), e.desc)
                }
            }
            handlebars::TemplateRenderError::TemplateError(e) => {
                if e.line_no.is_some() {
                    HEMTTError::LINENO(FileErrorLineNumber {
                        error: e.reason.to_string(),
                        line: e.line_no,
                        col: e.column_no,
                        note: None,
                        file: "".to_string(),
                        content: "".to_string(),
                    })
                } else {
                    HEMTTError::GENERIC("Render error".to_string(), e.reason.to_string())
                }
            }
            _ => unimplemented!(),
        }
    }
}

impl From<armake2::config::config_grammar::ParseError> for HEMTTError {
    fn from(err: armake2::config::config_grammar::ParseError) -> HEMTTError {
        println!("\n\n\n\nconfig error: {:?}\n\n\n\n\n", err);
        HEMTTError::LINENO(FileErrorLineNumber {
            line: Some(err.line),
            col: Some(err.column),
            error: format!("Expected one of {}", err.expected.into_iter().collect::<Vec<&str>>().join(", ")),
            content: "Unknown content".to_string(),
            file: "Unknown file".to_string(),
            note: None,
        })
    }
}
