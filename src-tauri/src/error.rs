use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum SetupError {
    Io(std::io::Error),
    Reqwest(reqwest::Error),
    Zip(zip::result::ZipError),
    Join(tokio::task::JoinError),
    Tauri(tauri::Error),
    Other(String),
}

impl Error for SetupError {}

impl fmt::Display for SetupError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SetupError::Io(e) => write!(f, "IO error: {}", e),
            SetupError::Reqwest(e) => write!(f, "HTTP error: {}", e),
            SetupError::Zip(e) => write!(f, "ZIP error: {}", e),
            SetupError::Join(e) => write!(f, "Task join error: {}", e),
            SetupError::Tauri(e) => write!(f, "Tauri error: {}", e),
            SetupError::Other(e) => write!(f, "Error: {}", e),
        }
    }
}

impl From<std::io::Error> for SetupError {
    fn from(err: std::io::Error) -> Self {
        SetupError::Io(err)
    }
}

impl From<reqwest::Error> for SetupError {
    fn from(err: reqwest::Error) -> Self {
        SetupError::Reqwest(err)
    }
}

impl From<zip::result::ZipError> for SetupError {
    fn from(err: zip::result::ZipError) -> Self {
        SetupError::Zip(err)
    }
}

impl From<tokio::task::JoinError> for SetupError {
    fn from(err: tokio::task::JoinError) -> Self {
        SetupError::Join(err)
    }
}

impl From<tauri::Error> for SetupError {
    fn from(err: tauri::Error) -> Self {
        SetupError::Tauri(err)
    }
}