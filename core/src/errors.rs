use std::fmt;

pub type ResultReturn<T> = Result<T, Error>;

#[derive(Debug)]
pub enum ErrorCode {
    InvalidParameters,
    ConnectionErr,
    NetworkErr,
}

#[derive(Debug)]
pub struct Error {
    code: ErrorCode,
    info: String,
}

impl Error {
    pub fn new(code: ErrorCode, info: &str) -> Error {
        Error {code: code, info: String::from(info)}
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[Error] {} - {}", match (*self).code {
            ErrorCode::InvalidParameters => "Invalid parameters",
            ErrorCode::ConnectionErr => "Connection error",
            ErrorCode::NetworkErr => "Network error",
        }, (*self).info)
    }
}
