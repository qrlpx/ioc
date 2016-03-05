use reflect;

use std::error::Error as StdError;
use std::fmt::{self, Display};
use std::sync::{PoisonError, TryLockError};

// ++++++++++++++++++++ DummyError ++++++++++++++++++++

#[derive(Debug)]
pub struct DummyError(());

impl Display for DummyError {
    fn fmt(&self, _: &mut fmt::Formatter) -> Result<(), fmt::Error> { unreachable!() }
}

impl StdError for DummyError {
    fn description(&self) -> &str { unreachable!() }
}

// ++++++++++++++++++++ Error ++++++++++++++++++++
// TODO: error messages need some work

/// TODO something about BorrowState when using RefCells for Services?
#[derive(Debug)]
pub enum Error<'a, Key: 'a> {
    NotFound{ key: &'a Key },
    Poisoned{ key: &'a Key },
    WouldBlock{ key: &'a Key },
    MismatchedType{ key: &'a Key, expected: &'static str, found: &'static str },
    CreationError{ key: &'a Key, error: Box<StdError> }
}

impl<'a, Key> Display for Error<'a, Key>
    where Key: reflect::Key
{
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let desc = self.description();
        match self {
            &Error::NotFound{ key } 
            | &Error::Poisoned{ key } 
            | &Error::WouldBlock{ key } => {
                fmt.write_fmt(format_args!("[{:?}] {}.", key, desc))
            }
            &Error::MismatchedType{ key, expected, found } => {
                fmt.write_fmt(format_args!("[{:?}] {}: Expected '{}' found '{}'.", key, desc, expected, found))
            }
            &Error::CreationError{ key, ref error } => {
                fmt.write_fmt(format_args!("[{:?}] {}: {}.", key, desc, error))
            }
        }
    }
}

impl<'a, Key> StdError for Error<'a, Key> 
    where Key: reflect::Key
{
    fn description(&self) -> &str {
        match self {
            &Error::NotFound{ .. } => "Service could not be found",
            &Error::Poisoned{ .. } => "Service could not be aquired, mutex was poisoned",
            &Error::WouldBlock{ .. } => "Service could not be aquired, mutex would block",
            &Error::MismatchedType{ .. } => "Service is of wrong type",
            &Error::CreationError{ .. } => "Factory failed to create object",
        }
    }
}

impl<'a, Key, X> From<(&'a Key, PoisonError<X>)> for Error<'a, Key> 
    where Key: reflect::Key
{
    fn from((key, _): (&'a Key, PoisonError<X>)) -> Self {
        Error::Poisoned{ key: key }
    }
}

impl<'a, Key, X> From<(&'a Key, TryLockError<X>)> for Error<'a, Key> 
    where Key: reflect::Key
{
    fn from((key, err): (&'a Key, TryLockError<X>)) -> Self {
        match err {
            TryLockError::Poisoned(_) => Error::Poisoned{ key: key },
            TryLockError::WouldBlock => Error::WouldBlock{ key: key }
        }
    }
}

// ++++++++++++++++++++ utility ++++++++++++++++++++

pub fn or_err<'a, Key, X, E>(key: &'a Key, res: Result<X, E>) -> Result<X, Error<'a, Key>>
    where Key: reflect::Key, Error<'a, Key>: From<(&'a Key, E)>
{
    match res {
        Ok(r) => Ok(r),
        Err(err) => Err(Error::from((key, err)))
    }
}
