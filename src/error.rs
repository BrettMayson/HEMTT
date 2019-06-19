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

#[derive(Debug)]
pub enum HEMTTError {
    IO(std::io::Error),
    TOML(toml::ser::Error),
}

impl std::fmt::Display for HEMTTError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            // Both underlying errors already impl `Display`, so we defer to
            // their implementations.
            HEMTTError::IO(ref err) => write!(f, "IO error: {}", err),
            HEMTTError::TOML(ref err) => write!(f, "TOML error: {}", err),
        }
    }
}

impl std::error::Error for HEMTTError {
    fn cause(&self) -> Option<&std::error::Error> {
        match *self {
            // N.B. Both of these implicitly cast `err` from their concrete
            // types (either `&io::Error` or `&num::ParseIntError`)
            // to a trait object `&Error`. This works because both error types
            // implement `Error`.
            HEMTTError::IO(ref err) => Some(err),
            HEMTTError::TOML(ref err) => Some(err),
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
