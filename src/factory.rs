use std::any::Any;
use std::fmt::Debug;

pub trait FactoryObject: Any + Sized {
    type Factory: Factory<Self>;
}

pub trait Factory<Obj: FactoryObject> {
    type Error: Debug;
    fn create(&mut self) -> Result<Obj, Self::Error>;
}

impl<Obj, T: ?Sized> Factory<Obj> for Box<T>
    where Obj: FactoryObject, T: Factory<Obj>
{
    type Error = T::Error;
    fn create(&mut self) -> Result<Obj, Self::Error> {
        (**self).create()
    }
}
