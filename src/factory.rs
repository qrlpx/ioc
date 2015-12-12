use methods::Method;
use reflect;

use std::error::Error;
use std::marker::Reflect;
use std::ops::Deref;

pub trait FactoryObject: Reflect + Sized {
    type Key: reflect::Key = <Self::Factory as reflect::Object>::Key;
    type Factory: reflect::Object<Key = Self::Key> /* + Factory<'a, _>*/;
}

/*/// TODO is this needed? 
pub trait FactoryBase<Key, Base: ?Sized, Obj> 
    where Key: reflect::Key, Base: Any
{
    type Error: Error;
    
    fn create(&self, ioc: &Container<Key, Base>) -> Result<Obj, CreationError<Self::Error>>;
}

impl<Key, Base: ?Sized, Obj, T> FactoryBase<Key, Base, Obj> for T
    where Key: reflect::Key, Base: Any, T: Deref, T::Target: FactoryBase<Key, Base, Obj>
{
    type Error = <T::Target as FactoryBase<Key, Base, Obj>>::Error;

    fn create(&self, ioc: &Container<Key, Base>) -> Result<Obj, CreationError<Self::Error>> {
        (**self).create(ioc)
    }
}*/

pub trait Factory<'a, Cont, Obj> {
    type Args: Method<'a, Cont>;
    type ArgsRet = <Self::Args as Method<'a, Cont>>::Ret;

    type Error: Error;

    fn create(&self, args: <Self::Args as Method<'a, Cont>>::Ret) -> Result<Obj, Self::Error>;

}

impl<'a, Cont, Obj, T> Factory<'a, Cont, Obj> for T
    where Obj: FactoryObject, T: Deref, T::Target: Factory<'a, Cont, Obj>
{
    type Args = <T::Target as Factory<'a, Cont, Obj>>::Args;

    type Error = <T::Target as Factory<'a, Cont, Obj>>::Error;

    fn create(&self, args: <Self::Args as Method<'a, Cont>>::Ret) -> Result<Obj, Self::Error> {
        (**self).create(args)
    }
}

