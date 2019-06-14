use failure::*;
use std::error::Error;
use futures::sync::mpsc::{TrySendError, SendError};

#[derive(Fail, Debug)]
pub enum LBError {
    /// Error in FromExt node that is fatal.
    #[fail(display = "Load Balancer error: {}", _0)]
    FromExt(String),
    /// Error in server node that is fatal.
    #[fail(display = "Load Balancer error: {}", _0)]
    InternalThrow(String),
    /// Error from user's input that is fatal.
    #[fail(display = "Bad input, user error: {}", _0)]
    UserError(String),
}

impl From<std::option::NoneError> for LBError {
    fn from(_: std::option::NoneError) -> Self {
        LBError::FromExt("Expected Some found None".into())
    }
}

impl From<std::io::Error> for LBError {
    fn from(e: std::io::Error) -> Self {
        LBError::FromExt(e.description().into())
    }
}

impl From<()> for LBError {
    fn from(_: ()) -> Self {
        LBError::FromExt("Empty Error".into())
    }
}

impl<T: 'static> From<TrySendError<T>> for LBError {
    fn from(e: TrySendError<T>) -> Self {
        LBError::FromExt(e.description().into())

    }
}

impl<T: 'static> From<SendError<T>> for LBError {
    fn from(e: SendError<T>) -> Self {
        LBError::FromExt(e.description().into())

    }
}

impl From<LBError> for () {
    fn from(_: LBError) -> Self {
        ()
    }
}
