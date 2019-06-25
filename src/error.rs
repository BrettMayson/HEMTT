pub trait PrintableError<T, E> {
    fn unwrap_or_print(self) -> T;
}
impl<T, E: std::fmt::Debug + std::fmt::Display> PrintableError<T, E> for Result<T, E> {
    fn unwrap_or_print(self) -> T {
        if let Err(error) = &self {
            eprintln!("{}: {}", "error", error);
            std::process::exit(1);
        }
        self.unwrap()
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct FileErrorLineNumber {
    pub file: String,
    pub content: String,
    pub error: String,
    pub line: Option<usize>,
    pub col: Option<usize>,
    pub note: Option<String>,
}

#[derive(Debug)]
pub enum HEMTTError {
    IO(std::io::Error),
    TOML(toml::ser::Error),
    GENERIC(String, String),
    LINENO(FileErrorLineNumber),
}

impl std::fmt::Display for HEMTTError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            HEMTTError::IO(ref err) => write!(f, "IO error: {}", err),
            HEMTTError::TOML(ref err) => write!(f, "TOML error: {}", err),
            HEMTTError::GENERIC(ref s, ref _v) => write!(f, "{}", s),
            HEMTTError::LINENO(ref err) => write!(f, "{}", err.error),
        }
    }
}

impl std::error::Error for HEMTTError {
    fn cause(&self) -> Option<&std::error::Error> {
        match *self {
            HEMTTError::IO(ref err) => Some(err),
            HEMTTError::TOML(ref err) => Some(err),
            HEMTTError::GENERIC(ref _s, ref _v) => Some(self),
            HEMTTError::LINENO(ref _e) => Some(self),
        }
    }
}

impl From<std::io::Error> for HEMTTError {
    fn from(err: std::io::Error) -> HEMTTError {
        HEMTTError::IO(err)
    }
}

impl From<toml::ser::Error> for HEMTTError {
    fn from(err: toml::ser::Error) -> HEMTTError {
        HEMTTError::TOML(err)
    }
}

impl From<handlebars::TemplateRenderError> for HEMTTError {
    fn from(err: handlebars::TemplateRenderError) -> HEMTTError {
        match err {
            handlebars::TemplateRenderError::RenderError(e) => {
                if let Some(_) = e.line_no {
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
            },
            handlebars::TemplateRenderError::TemplateError(e) => {
                if let Some(_) = e.line_no {
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
            },
            _ => { unimplemented!() }
        }
    }
}
