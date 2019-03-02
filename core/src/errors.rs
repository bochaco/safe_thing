// Copyright 2017-2019 Gabriel Viganotti <@bochaco>.
//
// This file is part of the SAFEthing Framework.
//
// The SAFEthing Framework is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// The SAFEthing Framework is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with the SAFEthing Framework. If not, see <https://www.gnu.org/licenses/>.

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
        Error {
            code: code,
            info: String::from(info),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "[Error] {} - {}",
            match (*self).code {
                ErrorCode::InvalidParameters => "Invalid parameters",
                ErrorCode::ConnectionErr => "Connection error",
                ErrorCode::NetworkErr => "Network error",
            },
            (*self).info
        )
    }
}
