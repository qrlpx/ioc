use error::Error;
use guards::{ReadGuard, WriteGuard};
use factory::FactoryBase;
use methods::Method;
use reflect;

use downcast::Downcast;

use std::any::Any;
use std::collections::BTreeMap;
use std::ops::{Deref, DerefMut};

fn type_name<T: ::std::marker::Reflect>() -> &'static str {
    unsafe { ::std::intrinsics::type_name::<T>() }
}

pub trait Container<'a>: Any + Sized {
    type Key: reflect::Key;
    type ServiceBase: ?Sized + Any;

    type ReadGuardBase: Deref<Target = Box<Self::ServiceBase>> + 'a;
    type WriteGuardBase: DerefMut<Target = Box<Self::ServiceBase>> + 'a;

    fn read_service_base(
        &'a self, 
        key: &'a Self::Key
    ) -> Result<Self::ReadGuardBase, Error<'a, Self::Key>>;

    fn write_service_base(
        &'a self, 
        key: &'a Self::Key
    ) -> Result<Self::WriteGuardBase, Error<'a, Self::Key>>;

    fn read_service<Svc>(
        &'a self, 
        key: &'a Self::Key
    ) -> Result<ReadGuard<Svc, Self::ServiceBase, Self::ReadGuardBase>, Error<'a, Self::Key>>
        where Svc: Any, Self::ServiceBase: Downcast<Svc>
    {
        let base = try!{self.read_service_base(key)};
        if !base.is_type() {
            return Err(Error::MismatchedType{ 
                key: key, 
                expected: type_name::<Svc>(),
                found: type_name::<Svc>(),
            })
        };
        Ok(ReadGuard::new(base))
    }

    fn write_service<Svc>(
        &'a self, 
        key: &'a Self::Key
    ) -> Result<WriteGuard<Svc, Self::ServiceBase, Self::WriteGuardBase>, Error<'a, Self::Key>>
        where Svc: Any, Self::ServiceBase: Downcast<Svc>
    {
        let base = try!{self.write_service_base(key)};
        if !base.is_type() {
            return Err(Error::MismatchedType{ 
                key: key, 
                expected: type_name::<Svc>(),
                found: type_name::<Svc>(),
            })
        };
        Ok(WriteGuard::new(base))
    }

    fn read<Svc>(
        &'a self
    ) -> Result<ReadGuard<Svc, Self::ServiceBase, Self::ReadGuardBase>, Error<'a, Self::Key>>
        where Svc: reflect::Service<Key = Self::Key>, Self::ServiceBase: Downcast<Svc>
    {
        self.read_service(Svc::key())
    }

    fn write<Svc>(
        &'a self
    ) -> Result<WriteGuard<Svc, Self::ServiceBase, Self::WriteGuardBase>, Error<'a, Self::Key>>
        where Svc: reflect::Service<Key = Self::Key>, Self::ServiceBase: Downcast<Svc>
    {
        self.write_service(Svc::key())
    }

    fn read_all(
        &'a self, 
    ) -> Result<BTreeMap<&'a Self::Key, Self::ReadGuardBase>, Error<'a, Self::Key>>;

    fn write_all(
        &'a self, 
    ) -> Result<BTreeMap<&'a Self::Key, Self::WriteGuardBase>, Error<'a, Self::Key>>;

    fn create_factory_object<Obj, Svc>(
        &'a self,
        svc: &'a Self::Key
    ) -> Result<Obj, Error<'a, Self::Key>>
        where Svc: FactoryBase<'a, Self, Obj>, Self::ServiceBase: Downcast<Svc>
    {
        let factory = try!{self.read_service::<Svc>(svc)};
        factory.create(svc, self)
    }

    fn create<Obj>(&'a self) -> Result<Obj, Error<'a, Self::Key>>
    where 
        Obj: reflect::FactoryObject<Key = Self::Key>,
        Obj::Factory: FactoryBase<'a, Self, Obj>, 
        Self::ServiceBase: Downcast<Obj::Factory>
    {
        let key = <Obj::Factory as reflect::Service>::key();
        self.create_factory_object::<Obj, Obj::Factory>(key)
    }

    fn resolve<M>(&'a self) -> Result<M::Ret, Error<'a, Self::Key>>
        where M: Method<'a, Self>
    {
        M::resolve(self)
    }
}

pub trait StagedContainer<'a>: Container<'a> {
    type Stage: Container<'a, Key = Self::Key, ServiceBase = Self::ServiceBase>;

    /* TODO
    fn get_stage<St>(&self) -> Option<&Self::Stage>
        where St: reflect::Service<Key = Self::Key>;
    
    fn read_stage<St>(
        &'a self,
    ) -> Result<BTreeMap<&'a Self::Key, Self::ReadGuardBase>, Error<'a, Self::Key>> 
        where St: reflect::Service<Key = Self::Key>;

    fn write_stage<St>(
        &'a self,
    ) -> Result<BTreeMap<&'a Self::Key, Self::WriteGuardBase>, Error<'a, Self::Key>> 
        where St: reflect::Service<Key = Self::Key>;
    */

    type StageIter: Iterator<Item = (&'a Self::Key, &'a Self::Stage)> + 'a;

    fn stages(&'a self) -> Self::StageIter;
}

