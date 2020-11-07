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
pub enum ArmaConfigError {
    ParsingError {
        positives: Vec<String>,
        negatives: Vec<String>,
        position: pest::error::LineColLocation,
    },
    InvalidInput(String),
    InvalidProperty(String),
    NotProcessed,
    NotRoot,

    // Wrappers
    IO(std::io::Error),
    PATH(IOPathError),
    GENERIC(String),
}

impl ArmaConfigError {
    pub fn warn(&self) {
        warn!("{}", self);
    }
    pub fn error(&self) {
        error!("{}", self);
    }
}

impl std::fmt::Display for ArmaConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Self::GENERIC(ref err) => write!(f, "{}", err),
            Self::IO(ref err) => write!(f, "IO error: {}", err),
            Self::PATH(ref err) => write!(f, "IO error {}: {}", err.path.display(), err.source),
            Self::NotProcessed => write!(f, "Attempt to perform action on non-processed AST"),
            Self::NotRoot => write!(f, "The root of the AST is required"),
            Self::InvalidInput(ref err) => write!(f, "Invalid Input: {}", err),
            Self::InvalidProperty(ref err) => write!(f, "Invalid Property: {}", err),
            Self::ParsingError {
                ref positives,
                ref position,
                ..
            } => write!(f, "Expected {:?} at {:?}", positives, position),
        }
    }
}

impl From<std::io::Error> for ArmaConfigError {
    fn from(err: std::io::Error) -> Self {
        Self::IO(err)
    }
}

#[derive(Debug)]
pub struct IOPathError {
    pub source: std::io::Error,
    pub path: std::path::PathBuf,
}
