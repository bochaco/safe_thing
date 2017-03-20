use std::fmt;

pub type ResultReturn<T> = Result<T, Error>;

pub enum ErrorCode {
    ID_USED_BY_OTHER_ACCOUNT,
    INVALID_PARAMETERS,
}

pub struct Error {
    code: ErrorCode,
    msg: String,
}

impl Error {
    pub fn new(code: ErrorCode, msg: &str) -> Error {
        Error {code: code, msg: String::from(msg)}
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error: - {}", match (*self).code {
            ErrorCode::ID_USED_BY_OTHER_ACCOUNT => "ID belongs to another account",
            ErrorCode::INVALID_PARAMETERS => "Invalid parameters",
        })
    }
}
