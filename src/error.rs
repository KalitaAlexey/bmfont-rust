use super::ConfigParseError;
use std::error::Error as StdError;
use std::fmt::{Display, Error as FmtError, Formatter};
use std::io::Error as IOError;

#[derive(Debug)]
pub enum Error {
    IOError(IOError),
    ConfigParseError(ConfigParseError),
}

impl Display for Error {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), FmtError> {
        match *self {
            Error::IOError(ref error) => Display::fmt(error, formatter),
            Error::ConfigParseError(ref error) => Display::fmt(error, formatter),
        }
    }
}

impl From<IOError> for Error {
    fn from(io_error: IOError) -> Error {
        Error::IOError(io_error)
    }
}

impl From<ConfigParseError> for Error {
    fn from(parse_error: ConfigParseError) -> Error {
        Error::ConfigParseError(parse_error)
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        "BMFont creation error"
    }
}
