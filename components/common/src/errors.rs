#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("unknown error")]
    Unknown,
    #[error("{0:?}")]
    Std(#[from] Box<dyn std::error::Error + Sync + Send>),
    #[error("{0:?}")]
    Internal(String),
    #[error(transparent)]
    IOErr(#[from] std::io::Error),
}

pub type Result<T> = anyhow::Result<T, Error>;