use std::any::Any;
use std::fmt::Debug;

pub trait Key: Debug + Clone + Ord + Any {}

impl<T> Key for T
    where T: Debug + Clone + Ord + Any
{}

pub trait Service: Any + Sized {
    type Key: Key = String;
    fn key() -> &'static Self::Key;
}

pub trait FactoryObject: Any + Sized {
    type Key: Key = <Self::Factory as Service>::Key;
    type Factory: Service<Key = Self::Key> /* + FactoryBase<'a, _>*/;
}


