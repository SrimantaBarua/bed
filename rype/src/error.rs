// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::{error, fmt, io, result};

pub type Result<T> = result::Result<T, Error>;

/// Errors within rype
#[derive(Debug)]
pub enum Error {
    /// IO errors (from reading files etc)
    Io(io::Error),
    /// Invalid font file
    Invalid,
    /// Face index out of bounds
    FaceIndexOutOfBounds,
    /// No suitable table found for cmap
    CmapNoTable,
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Error {
        Error::Io(e)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Io(e) => writeln!(f, "IO error: {}", e),
            Error::Invalid => writeln!(f, "invalid font file"),
            Error::FaceIndexOutOfBounds => writeln!(f, "face index out of bounds"),
            Error::CmapNoTable => writeln!(f, "no suitable cmap table found"),
        }
    }
}

impl error::Error for Error {}
