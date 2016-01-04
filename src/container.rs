use errors::Error;
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

    fn invoke<M>(&'a self) -> Result<M::Ret, Error<'a, Self::Key>>
        where M: Method<'a, Self>
    {
        M::invoke(self)
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
        where St: reflect::Service<Key = Self::Key>;*/

    type StageIter: Iterator<Item = (&'a Self::Key, &'a Self::Stage)> + 'a;

    fn stages(&'a self) -> Self::StageIter;
}

// ++++++++++++++++++++ methods ++++++++++++++++++++
// TODO move this somewhere else?
/*
// ++++++++++++++++++++ Container ++++++++++++++++++++

pub struct Container<Key, Base: ?Sized> {
    services: BTreeMap<Key, RwLock<Box<Base>>>,
    abba_prevention: Mutex<()>, 
}

pub type Services<'a, Key, Base: ?Sized> = btree_map::Iter<'a, Key, RwLock<Box<Base>>>;

impl<Key, Base: ?Sized> Container<Key, Base> 
    where Key: reflect::Key, Base: Any
{
    #[doc(hidden)]
    pub fn new() -> Self {
        Container{ 
            services: BTreeMap::new(),
            abba_prevention: Mutex::new(()),
        }
    }

    #[doc(hidden)]
    pub fn register_service(&mut self, key: Key, svc: Box<Base>){
        self.services.insert(key, RwLock::new(svc));
    }

    #[doc(hidden)]
    pub fn unregister_service(&mut self, key: &Key) -> bool {
        self.services.remove(key).is_some()
    }

    pub fn services(&self) -> Services<Key, Base> { 
        self.services.iter()
    }

    pub fn get_service(&self, key: &Key) -> Option<&RwLock<Box<Base>>> {
        self.services.get(key)
    }

    pub fn invoke<'a, M>(&'a self) -> Result<M::Ret, M::Error>
        where M: Method<'a, Self>, 
    {
        // prevent AB-BA deadlocks
        let _ = self.abba_prevention.lock();

        M::invoke(self)
    }
}

impl<'a, Key, Base: ?Sized> Method<'a, Container<Key, Base>> for ()
    where Key: reflect::Key, Base: Any
{
    type Ret = ();
    type Error = DummyError;

    fn invoke(_: &'a Container<Key, Base>) -> Result<Self::Ret, Self::Error> {
        Ok(())
    }
}

impl<'a, Key, Base: ?Sized, Svc> Method<'a, Container<Key, Base>> for Read<Svc>
    where Key: reflect::Key, Base: Any + Downcast<Svc>, Svc: reflect::Object<Key = Key>
{
    type Ret = ReadGuard<'a, Svc, Base>;
    type Error = LockError<'a, Key>;

    fn invoke(ioc: &'a Container<Key, Base>) -> Result<Self::Ret, Self::Error> {
        let key = Svc::key();
        let service = match ioc.get_service(key) {
            Some(r) => r, 
            None => return Err(LockError::NotFound{ key: key })
        };
        let guard = match service.read() {
            Ok(r) => r, 
            Err(_) => return Err(LockError::Poisoned{ key: key })
        };
        if !guard.is_type() {
            return Err(LockError::MismatchedType{ 
                key: key, 
                expected: type_name::<Svc>(),
                found: type_name::<Svc>(),
            })
        };
        Ok(ReadGuard::new(guard))
    }
}

impl<'a, Key, Base: ?Sized, Svc> Method<'a, Container<Key, Base>> for Write<Svc>
    where Key: reflect::Key, Base: Any + Downcast<Svc>, Svc: reflect::Object<Key = Key>
{
    type Ret = WriteGuard<'a, Svc, Base>;
    type Error = LockError<'a, Key>;

    fn invoke(ioc: &'a Container<Key, Base>) -> Result<Self::Ret, Self::Error> {
        let key = Svc::key();
        let service = match ioc.get_service(key) {
            Some(r) => r, 
            None => return Err(LockError::NotFound{ key: key })
        };
        let guard = match service.write() {
            Ok(r) => r, 
            Err(_) => return Err(LockError::Poisoned{ key: key })
        };
        if !guard.is_type() {
            return Err(LockError::MismatchedType{ 
                key: key, 
                expected: type_name::<Svc>(),
                found: type_name::<Svc>(),
            })
        };
        Ok(WriteGuard::new(guard))
    }
}

impl<'a, Key, Base: ?Sized> Method<'a, Container<Key, Base>> for ReadAll
    where Key: reflect::Key, Base: Any
{
    type Ret = ReadGuardMap<'a, Key, Base>;
    type Error = LockError<'a, Key>;

    fn invoke(ioc: &'a Container<Key, Base>) -> Result<Self::Ret, Self::Error> {
        let mut map = ReadGuardMap::new();
        for (key, service) in ioc.services() {
            let guard = match service.read() {
                Ok(r) => r, Err(_) => return Err(LockError::Poisoned{ key: key })
            };
            map.insert(key, guard);
        }
        Ok(map)
    }
}

impl<'a, Key, Base: ?Sized> Method<'a, Container<Key, Base>> for WriteAll
    where Key: reflect::Key, Base: Any
{
    type Ret = WriteGuardMap<'a, Key, Base>;
    type Error = LockError<'a, Key>;

    fn invoke(ioc: &'a Container<Key, Base>) -> Result<Self::Ret, Self::Error> {
        let mut map = WriteGuardMap::new();
        for (key, service) in ioc.services() {
            let guard = match service.write() {
                Ok(r) => r, Err(_) => return Err(LockError::Poisoned{ key: key })
            };
            map.insert(key, guard);
        }
        Ok(map)
    }
}

impl<'a, Key, Base: ?Sized, Obj> Method<'a, Container<Key, Base>> for Create<Obj> 
where 
    Key: reflect::Key, 
    Base: Any + Downcast<Obj::Factory>,
    Obj: FactoryObject<Key = Key> + 'a, 
    Obj::Factory: FactoryBase<'a, Container<Key, Base>, Obj>
{
    type Ret = Obj;
    type Error = CreationError<'a, 
        <Obj::Factory as reflect::Object>::Key, 
        <Obj::Factory as FactoryBase<'a, Container<Key, Base>, Obj>>::Error
    >;

    fn invoke(ioc: &'a Container<Key, Base>) -> Result<Self::Ret, Self::Error> {
        let factory = try!{Read::<Obj::Factory>::invoke(ioc)};
        
        factory.create(ioc)
    }
}

impl<Key, Base: ?Sized> Container<Key, Base> 
    where Key: reflect::Key, Base: Any
{
    /// Shortcut for `.invoke::<ioc::Read<{Svc}>>(())`.
    pub fn read<Svc>(&self) -> Result<ReadGuard<Svc, Base>, LockError<Svc::Key>>
        where Svc: reflect::Object<Key = Key>, Base: Downcast<Svc>
    {
        self.invoke::<Read<Svc>>()
    }

    /// Shortcut for `.invoke::<ioc::Write<{Svc}>>(())`.
    pub fn write<Svc>(&self) -> Result<WriteGuard<Svc, Base>, LockError<Svc::Key>>
        where Svc: reflect::Object<Key = Key>, Base: Downcast<Svc>
    {
        self.invoke::<Write<Svc>>()
    }

    /// Shortcut for `.invoke::<ioc::ReadAll>(())`.
    pub fn read_all(&self) -> Result<ReadGuardMap<Key, Base>, LockError<Key>> {
        self.invoke::<ReadAll>()
    }

    /// Shortcut for `.invoke::<ioc::WriteAll>(())`.
    pub fn write_all(&self) -> Result<WriteGuardMap<Key, Base>, LockError<Key>> {
        self.invoke::<WriteAll>()
    }

    /// Shortcut for `.invoke::<ioc::Create<{Obj}>>(args)`.
    pub fn create<'a, Obj>(&'a self) -> Result<Obj, CreationError<<Obj::Factory as reflect::Object>::Key, <Obj::Factory as FactoryBase<'a, Self, Obj>>::Error>>  
    where 
        Obj: FactoryObject<Key = Key> + 'a,
        Obj::Factory: FactoryBase<'a, Self, Obj>,
        Base: Downcast<Obj::Factory>,
    {
        self.invoke::<Create<Obj>>()
    }
}

// ++++++++++++++++++++ ContainerBuilder ++++++++++++++++++++

pub struct ContainerBuilder<Key, Base: ?Sized> {
    ioc: Container<Key, Base>,
}

impl<Key, Base: ?Sized> ContainerBuilder<Key, Base>
    where Key: reflect::Key, Base: Any
{
    pub fn new() -> Self {
        ContainerBuilder{ ioc: Container::new() }
    }

    pub fn register_service(&mut self, key: Key, svc: Box<Base>) -> &mut Self {
        self.ioc.register_service(key, svc);
        self
    }

    /// NOTE: The `Box<Svc>: Into<Box<Base>>`-clause is needed due to rusts lack of 
    /// HKT or a `Coercible`-trait (to name two solutions).
    pub fn register<Svc>(&mut self, svc: Svc) -> &mut Self
        where Svc: reflect::Object<Key = Key>, Box<Svc>: Into<Box<Base>>,
    {
        self.register_service(Svc::key().clone(), Box::new(svc).into())
    }

    pub fn build(self) -> Container<Key, Base> {
        self.ioc
    }
}

impl<Key, Base: ?Sized> Default for ContainerBuilder<Key, Base>
    where Key: reflect::Key, Base: Any
{
    fn default() -> Self { Self::new() }
}
*/
