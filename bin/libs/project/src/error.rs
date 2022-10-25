#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Toml Error: {0}")]
    Toml(#[from] toml::de::Error),
}
