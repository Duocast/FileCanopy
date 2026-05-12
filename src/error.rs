use std::io;
use std::path::PathBuf;

use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("io error at {path:?}: {source}")]
    Io {
        path: Option<PathBuf>,
        #[source]
        source: io::Error,
    },

    #[error("scan failed: {0}")]
    Scan(String),

    #[error("duplicate analysis failed: {0}")]
    Duplicate(String),

    #[error("report generation failed: {0}")]
    Report(String),

    #[error("history store error: {0}")]
    History(String),

    #[error("scheduler error: {0}")]
    Scheduler(String),

    #[error("configuration error: {0}")]
    Config(String),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl From<io::Error> for Error {
    fn from(source: io::Error) -> Self {
        Error::Io { path: None, source }
    }
}
