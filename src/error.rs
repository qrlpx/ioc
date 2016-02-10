use reflect;

use std::error::Error as StdError;
use std::fmt::{self, Display};

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
    MismatchedType{ key: &'a Key, expected: &'static str, found: &'static str },
    CreationError{ key: &'a Key, error: Box<StdError> }
}

impl<'a, Key> Display for Error<'a, Key>
    where Key: reflect::Key
{
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let desc = self.description();
        match self {
            &Error::NotFound{ key } => {
                fmt.write_fmt(format_args!("[{:?}] {}.", key, desc))
            }
            &Error::Poisoned{ key } => {
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
            &Error::Poisoned{ .. } => "Service mutex was poisoned",
            &Error::MismatchedType{ .. } => "Service is of wrong type",
            &Error::CreationError{ .. } => "Factory failed to create object",
        }
    }
}

