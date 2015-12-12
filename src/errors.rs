// TODO: error messages need some work

use reflect;

use std::error::Error;
use std::fmt::{self, Debug, Display};

// ++++++++++++++++++++ DummyError ++++++++++++++++++++

#[derive(Debug)]
pub struct DummyError(());

impl Display for DummyError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> { unreachable!() }
}

impl Error for DummyError {
    fn description(&self) -> &str { unreachable!() }
}

// ++++++++++++++++++++ LockError ++++++++++++++++++++

#[derive(Debug)]
pub enum LockError<'a, Key: 'a> {
    NotFound{ key: &'a Key },
    Poisoned{ key: &'a Key },
    MismatchedType{ key: &'a Key, expected: &'static str, found: &'static str },
}

impl<'a, Key> Display for LockError<'a, Key> 
    where Key: reflect::Key
{
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let desc = self.description();
        match self {
            &LockError::NotFound{ key } => {
                fmt.write_fmt(format_args!("[{:?}] {}.", key, desc))
            }
            &LockError::Poisoned{ key } => {
                fmt.write_fmt(format_args!("[{:?}] {}.", key, desc))
            }
            &LockError::MismatchedType{ key, expected, found } => {
                //TODO print .expected & .found?
                fmt.write_fmt(format_args!(
                    "[{:?}] {}: Expected '{}' found '{}'", 
                    key, desc, expected, found
                ))
            }
        }
    }
}

impl<'a, Key> Error for LockError<'a, Key> 
    where Key: reflect::Key
{
    fn description(&self) -> &str {
        match self {
            &LockError::NotFound{ .. } => "Service could not be found",
            &LockError::Poisoned{ .. } => "Service lock was poisoned",
            &LockError::MismatchedType{ .. } => "Service is of wrong type"
        }
    }
}

// ++++++++++++++++++++ CreationError ++++++++++++++++++++

#[derive(Debug)]
pub enum CreationError<'a, Key: 'a, CE> {
    ///FIXME far future: this should be embedded.. CreationError should 'derive' LockError...
    LockError(LockError<'a, Key>),
    
    ///TODO naming?
    DependencyError{ key: &'a Key, error: Box<Error + 'a> },

    ///TODO naming?
    CreationError{ key: &'a Key, error: CE },
}

impl<'a, Key: 'a, CE> From<LockError<'a, Key>> for CreationError<'a, Key, CE> {
    fn from(err: LockError<'a, Key>) -> Self {
        CreationError::LockError(err)
    }
}

impl<'a, Key, CE> Display for CreationError<'a, Key, CE> 
    where Key: reflect::Key, CE: Error
{
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let desc = self.description();
        match self {
            &CreationError::LockError(ref err) => Display::fmt(err, fmt),
            &CreationError::DependencyError{ key, ref error } => {
                fmt.write_fmt(format_args!("[{:?}] {}: {}.", key, desc, error))
            }
            &CreationError::CreationError{ key, ref error } => {
                fmt.write_fmt(format_args!("[{:?}] {}: {}.", key, desc, error))
            }
        }
    }
}

impl<'a, Key, CE> Error for CreationError<'a, Key, CE> 
    where Key: reflect::Key, CE: Error
{
    fn description(&self) -> &str {
        match self {
            &CreationError::LockError(ref err) => err.description(),
            &CreationError::DependencyError{ .. } => "Failed to resolve dependencies",
            &CreationError::CreationError{ .. } => "Factory failed to create object",
        }
    }
    fn cause(&self) -> Option<&Error> {
        match self {
            //FIXME https://github.com/rust-lang/rust/issues/30349
            //&CreationError::DependencyError{ ref error, .. } => Some(&*error),
            &CreationError::CreationError{ ref error, .. } => Some(&*error),
            _ => None
        }
    }
}

// ++++++++++++++++++++ CreationError ++++++++++++++++++++

#[derive(Debug)]
pub struct MultiError<'a> {
    pub idx: usize,
    pub error: Box<Error + 'a>,
}

impl<'a> Display for MultiError<'a> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        fmt.write_fmt(format_args!("[method #{}] {}",  self.idx, self.error))
    }
}

impl<'a> Error for MultiError<'a> {
    fn description(&self) -> &str { self.error.description() }
    fn cause(&self) -> Option<&Error> { Some(&*self.error) }
}

