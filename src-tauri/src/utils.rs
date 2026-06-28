use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum VoltError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
    #[error("Process error: {0}")]
    Process(String),
    #[error("Configuration error: {0}")]
    Config(String),
    #[error("Service error: {0}")]
    Service(String),
    #[error("Permission denied: {0}")]
    Permission(String),
    #[error("Custom error: {0}")]
    Custom(String),
}

impl Serialize for VoltError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

pub type VoltResult<T> = Result<T, VoltError>;
