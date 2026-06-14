use serde::{Serialize, Serializer};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum VoltError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Config error: {0}")]
    Config(String),

    #[error("Service '{0}' not found")]
    ServiceNotFound(String),

    #[error("Service '{0}' is already running")]
    ServiceAlreadyRunning(String),

    #[error("Service '{0}' is not installed. Please run setup first.")]
    ServiceNotInstalled(String),

    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("Process error: {0}")]
    Process(String),

    #[error("Database error: {0}")]
    Database(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Port {0} is already in use")]
    PortInUse(u16),

    #[error("{0}")]
    Generic(String),
}

impl Serialize for VoltError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

pub type VoltResult<T> = Result<T, VoltError>;

impl From<String> for VoltError {
    fn from(s: String) -> Self {
        VoltError::Generic(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_serialization() {
        let err = VoltError::ServiceNotFound("nginx".to_string());
        let json = serde_json::to_string(&err).unwrap();
        assert_eq!(json, "\"Service 'nginx' not found\"");
    }

    #[test]
    fn test_io_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file missing");
        let volt_err: VoltError = io_err.into();
        match volt_err {
            VoltError::Io(_) => (),
            _ => panic!("Should have been an Io variant"),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceUsage {
    pub cpu: f32,
    pub memory: u64,
}
