use std::error::Error;
use std::{fmt, io};
use std::fmt::{Formatter};
use reqwest::StatusCode;

#[derive(Debug)]
pub enum APIError {
    IOError(io::Error),
    ReqwestError(reqwest::Error),
    JSONError(serde_json::Error),
    Forbidden(String),
    Unauthorized(String),
    NotFound(String),
    BadRequest(String),
    ServerError,
    UnhandledStatusCode(StatusCode)
}

// Implement Display for APIError.
impl fmt::Display for APIError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match *self {
            APIError::IOError(ref error) => write!(f, "IO error: {}", error),
            APIError::ReqwestError(ref error) => write!(f, "Reqwest error: {}", error),
            APIError::JSONError(ref error) => write!(f, "JSON error: {}", error),
            APIError::Forbidden(ref message) => write!(f, "Forbidden! {}", message),
            APIError::Unauthorized(ref message) => write!(f, "Unauthorized! {}", message),
            APIError::NotFound(ref message) => write!(f, "Not found! {}", message),
            APIError::BadRequest(ref message) =>
                write!(f, "Bad request! {}", message),
            APIError::ServerError => write!(f, "Server error!{}", ""),
            APIError::UnhandledStatusCode(ref status_code) =>
                write!(f, "Unhandled status code: {}", status_code.as_str())
        }
    }
}

// Implement Error for APIError.
impl Error for APIError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match *self {
            APIError::IOError(ref err) => Some(err),
            _ => None,
        }
    }
}

// Allow automatic conversion from io::Error to APIError.
impl From<io::Error> for APIError {
    fn from(value: io::Error) -> Self {
        APIError::IOError(value)
    }
}

// Allow automatic conversion from reqwest::Error to APIError.
impl From<reqwest::Error> for APIError {
    fn from(value: reqwest::Error) -> Self {
        APIError::ReqwestError(value)
    }
}

// Allow automatic conversion from serde_json::Error to APIError.
impl From<serde_json::Error> for APIError {
    fn from(value: serde_json::Error) -> Self {
        APIError::JSONError(value)
    }
}