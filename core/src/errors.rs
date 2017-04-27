use std::fmt;

pub type ResultReturn<T> = Result<T, Error>;

pub enum ErrorCode {
    InvalidParameters,
}

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
        }, (*self).info)
    }
}
