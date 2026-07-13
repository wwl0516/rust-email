use thiserror::Error;

#[derive(Error, Debug)]
pub enum MailError {
    #[error("connection failed: {0}")]
    Connection(String),
    #[error("authentication failed: {0}")]
    Authentication(String),
    #[error("TLS error: {0}")]
    Tls(String),
    #[error("network error: {0}")]
    Network(String),
    #[error("protocol error: {0}")]
    Protocol(String),
    #[error("parse error: {0}")]
    Parse(String),
    #[error("folder not found: {0}")]
    FolderNotFound(String),
    #[error("message not found: {0}")]
    MessageNotFound(String),
    #[error("operation timed out")]
    Timeout,
    #[error("not connected to server")]
    NotConnected,
    #[error("{0}")]
    Other(String),
}
