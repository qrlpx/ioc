use std::any::Any;
use std::fmt::Debug;

pub trait FactoryObject: Any + Sized {
    type Factory: Factory<Self>;
}

pub trait Factory<Obj: FactoryObject> {
    type Args;
    type Error: Debug;

    fn create(&mut self, args: Self::Args) -> Result<Obj, Self::Error>;
}

impl<Obj, T: ?Sized> Factory<Obj> for Box<T>
    where Obj: FactoryObject, T: Factory<Obj>
{
    type Args = T::Args;
    type Error = T::Error;

    fn create(&mut self, args: Self::Args) -> Result<Obj, Self::Error> {
        (**self).create(args)
    }
}
