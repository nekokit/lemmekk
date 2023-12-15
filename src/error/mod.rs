use std::{error::Error, fmt::Display, io};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum AppError {
    Config(String),
    Io(String),
    Serde(String),
    Extract(String),
}

impl Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::Config(v) => write!(f, "[AppError:Config] {}", v),
            AppError::Io(v) => write!(f, "[AppError:IO] {}", v),
            AppError::Serde(v) => write!(f, "[AppError:Serde] {}", v),
            AppError::Extract(v) => write!(f, "[AppError:Extract] {}", v),
        }
    }
}

impl Error for AppError {}

impl From<io::Error> for AppError {
    fn from(value: io::Error) -> Self {
        AppError::Io(value.to_string())
    }
}
impl From<toml::ser::Error> for AppError {
    fn from(value: toml::ser::Error) -> Self {
        AppError::Serde(value.to_string())
    }
}
impl From<toml::de::Error> for AppError {
    fn from(value: toml::de::Error) -> Self {
        AppError::Serde(value.to_string())
    }
}
impl From<regex::Error> for AppError {
    fn from(value: regex::Error) -> Self {
        AppError::Serde(value.to_string())
    }
}
