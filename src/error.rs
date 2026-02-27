//! Error types for the widget plugin.
//!
//! Provides a unified [`Error`] enum used across desktop and mobile implementations.
//! All variants carry a human-readable message that is serialized to JSON when
//! returned to the webview frontend.

use serde::{Serialize, Serializer};

/// Alias for `std::result::Result<T, Error>`.
pub type Result<T> = std::result::Result<T, Error>;

/// All possible errors produced by this plugin.
#[derive(Debug, Clone)]
pub enum Error {
    /// An I/O error (file system, network, etc.).
    Io(String),
    /// A generic string error.
    String(String),
    /// An error originating from the native mobile plugin bridge.
    #[cfg(mobile)]
    PluginInvoke(String),
    /// A JSON serialization / deserialization error.
    SerdeJson(String),
    /// The requested operation is not supported on the current platform.
    Unsupported(String),
}

impl Error {
    /// Create a generic [`Error::String`] from anything convertible to `String`.
    pub fn new(msg: impl Into<String>) -> Self {
        Error::String(msg.into())
    }

    /// Create an [`Error::Unsupported`] indicating that the operation is
    /// unavailable on the current platform.
    pub fn unsupported(operation: &str) -> Self {
        Error::Unsupported(format!(
            "`{}` is not supported on this platform",
            operation
        ))
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Io(err) => write!(f, "IO error: {}", err),
            Error::String(s) => write!(f, "{}", s),
            #[cfg(mobile)]
            Error::PluginInvoke(err) => write!(f, "Plugin invoke error: {}", err),
            Error::SerdeJson(err) => write!(f, "Serde JSON error: {}", err),
            Error::Unsupported(msg) => write!(f, "Unsupported: {}", msg),
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Io(err.to_string())
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::SerdeJson(err.to_string())
    }
}

#[cfg(mobile)]
impl From<tauri::plugin::mobile::PluginInvokeError> for Error {
    fn from(err: tauri::plugin::mobile::PluginInvokeError) -> Self {
        Error::PluginInvoke(err.to_string())
    }
}

impl From<&str> for Error {
    fn from(err: &str) -> Self {
        Error::String(err.to_string())
    }
}

impl From<String> for Error {
    fn from(err: String) -> Self {
        Error::String(err)
    }
}

impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

unsafe impl Send for Error {}
unsafe impl Sync for Error {}
