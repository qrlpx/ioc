use std::any::Any;
use std::fmt::Debug;

pub trait Key: Debug + Clone + Ord + Any {}

pub trait Object: Any + Sized {
    type Key: Key = String;
    fn key() -> &'static Self::Key;
}

