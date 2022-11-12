use hemtt_error::thiserror;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("RSA Error: {0}")]
    Rsa(#[from] rsa::errors::Error),
    #[error("IO Error: {0}")]
    Io(Box<std::io::Error>),
    #[error("PBO Error: {0}")]
    Pbo(Box<hemtt_pbo::Error>),

    #[error("Invalid length while reading")]
    InvalidLength,
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::Io(Box::new(e))
    }
}

impl From<hemtt_pbo::Error> for Error {
    fn from(e: hemtt_pbo::Error) -> Self {
        Self::Pbo(Box::new(e))
    }
}
