use std::{
    error::Error as StdError,
    fmt::{self, Display, Formatter},
};

use rsraw_sys as sys;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone)]
pub enum Error {
    Success,
    Unspecified,
    FileUnsupported,
    RequestForNonexistentImage,
    OutOfOrderCall,
    NoThumbnail,
    UnsupportedThumbnail,
    InputClosed,
    NotImplemented,
    RequestForNonexistentThumbnail,
    UnsufficientMemory,
    Data,
    Io,
    CancelledByCallback,
    BadCrop,
    TooBig,
    MempoolOverflow,
    /// The caller was built against a different LibRaw ABI than the linked library.
    VersionMismatch,
    Unknown(i32),
}

impl From<sys::LibRaw_errors> for Error {
    fn from(code: sys::LibRaw_errors) -> Self {
        match code {
            sys::LibRaw_errors_LIBRAW_SUCCESS => Error::Success,
            sys::LibRaw_errors_LIBRAW_UNSPECIFIED_ERROR => Error::Unspecified,
            sys::LibRaw_errors_LIBRAW_FILE_UNSUPPORTED => Error::FileUnsupported,
            sys::LibRaw_errors_LIBRAW_REQUEST_FOR_NONEXISTENT_IMAGE => {
                Error::RequestForNonexistentImage
            }
            sys::LibRaw_errors_LIBRAW_OUT_OF_ORDER_CALL => Error::OutOfOrderCall,
            sys::LibRaw_errors_LIBRAW_NO_THUMBNAIL => Error::NoThumbnail,
            sys::LibRaw_errors_LIBRAW_UNSUPPORTED_THUMBNAIL => Error::UnsupportedThumbnail,
            sys::LibRaw_errors_LIBRAW_INPUT_CLOSED => Error::InputClosed,
            sys::LibRaw_errors_LIBRAW_NOT_IMPLEMENTED => Error::NotImplemented,
            sys::LibRaw_errors_LIBRAW_REQUEST_FOR_NONEXISTENT_THUMBNAIL => {
                Error::RequestForNonexistentThumbnail
            }
            sys::LibRaw_errors_LIBRAW_UNSUFFICIENT_MEMORY => Error::UnsufficientMemory,
            sys::LibRaw_errors_LIBRAW_DATA_ERROR => Error::Data,
            sys::LibRaw_errors_LIBRAW_IO_ERROR => Error::Io,
            sys::LibRaw_errors_LIBRAW_CANCELLED_BY_CALLBACK => Error::CancelledByCallback,
            sys::LibRaw_errors_LIBRAW_BAD_CROP => Error::BadCrop,
            sys::LibRaw_errors_LIBRAW_TOO_BIG => Error::TooBig,
            sys::LibRaw_errors_LIBRAW_MEMPOOL_OVERFLOW => Error::MempoolOverflow,
            sys::LibRaw_errors_LIBRAW_VERSION_MISMATCH => Error::VersionMismatch,
            _ => Error::Unknown(code),
        }
    }
}

impl Error {
    pub fn check(code: i32) -> Result<()> {
        let err = Error::from(code);
        match err {
            Error::Success => Ok(()),
            _ => Err(err),
        }
    }

    pub fn repr(&self) -> &'static str {
        match self {
            Error::Success => "Success",
            Error::Unspecified => "UnspecifiedError",
            Error::FileUnsupported => "FileUnsupported",
            Error::RequestForNonexistentImage => "RequestForNonexistentImage",
            Error::OutOfOrderCall => "OutOfOrderCall",
            Error::NoThumbnail => "NoThumbnail",
            Error::UnsupportedThumbnail => "UnsupportedThumbnail",
            Error::InputClosed => "InputClosed",
            Error::NotImplemented => "NotImplemented",
            Error::RequestForNonexistentThumbnail => "RequestForNonexistentThumbnail",
            Error::UnsufficientMemory => "UnsufficientMemory",
            Error::Data => "DataError",
            Error::Io => "IoError",
            Error::CancelledByCallback => "CancelledByCallback",
            Error::BadCrop => "BadCrop",
            Error::TooBig => "TooBig",
            Error::MempoolOverflow => "MempoolOverflow",
            Error::VersionMismatch => "VersionMismatch",
            Error::Unknown(_) => "Unknown",
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "libraw error: {}", self.repr())
    }
}

impl StdError for Error {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_mismatch_is_mapped() {
        let err = Error::check(sys::LibRaw_errors_LIBRAW_VERSION_MISMATCH).unwrap_err();
        assert!(matches!(err, Error::VersionMismatch));
        assert_eq!(err.repr(), "VersionMismatch");
    }
}
