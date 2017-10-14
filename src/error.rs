
use std::io;
use std::fmt;
use std::error::Error;

/// An error which can occur parsing, writing, or manipulating an EBML document.
#[derive(Debug)]
pub enum EbmlError {
    /// An error from the standard I/O library.
    StdIo(io::Error),
    /// The EBML document, despite being valid EBML, has malformed content.
    MalformedDocument,
    /// An EBML ID was out of range.
    IdOutOfRange,
    /// The wrong ID was read.
    WrongId,
}
impl fmt::Display for EbmlError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "EBML error: {}", self.description())
    }
}
impl Error for EbmlError {
    fn description(&self) -> &str {
        match *self {
            EbmlError::StdIo(ref e) => e.description(),
            EbmlError::MalformedDocument => "malformed EBML document",
            EbmlError::IdOutOfRange => "an id was out of range",
            EbmlError::WrongId => "the wrong id was read",
        }
    }

    fn cause(&self) -> Option<&Error> {
        match *self {
            EbmlError::StdIo(ref e) => Some(e),

            _ => None,
        }
    }
}
impl From<io::Error> for EbmlError {
    fn from(e: io::Error) -> Self {
        EbmlError::StdIo(e)
    }
}

/// A `Result` with error type `EbmlError`.
pub type EbmlResult<T> = Result<T, EbmlError>;
