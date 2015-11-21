use service::*;
use factory::*;
use invocation_method::*;

use downcast::Downcast;

use std::any::Any;
use std::borrow::Borrow;
use std::collections::BTreeMap;
use std::fmt::Debug;
use std::sync::{Mutex, RwLock};

// ++++++++++++++++++++ Ioc ++++++++++++++++++++

/// TODO naming?
pub struct Ioc<Key = str, Base: ?Sized = DefaultBase> {
    services: BTreeMap<Key, RwLock<Box<Base>>>,

    deadlock_prevention: Mutex<()>, 
}

impl<Key, Base: ?Sized> Ioc<Key, Base> 
    where Key: Debug + Ord, Base: Any
{
    fn new(services: BTreeMap<Key, RwLock<Box<Base>>>) -> Self {
        Ioc{ 
            services: services,
            deadlock_prevention: Mutex::new(()),
        }
    }

    pub fn invoke<'a, M>(&'a self, args: M::Args) -> Result<M::Ret, M::Error>
        where M: InvocationMethod<'a, Key, Base>, 
    {
        // in order to prevent AB-BA deadlocks, we aquire a mutex before letting 
        // the InvocationMethod do it's thing.
        let _ = self.deadlock_prevention.lock();

        M::invoke(&self.services, args)
    }

    /// Shortcut for `.invoke::<ioc::Read<{Svc}>>(())`.
    pub fn read<'a, Svc>(
        &'a self
    ) -> Result<ServiceReadGuard<Svc, Base>, LockError<&'static Svc::Key>>
        where Svc: ServiceReflect, Key: Borrow<Svc::Key>, Base: Downcast<Svc>
    {
        self.invoke::<Read<Svc>>(())
    }

    /// Shortcut for `.invoke::<ioc::Write<{Svc}>>(())`.
    pub fn write<'a, Svc>(
        &'a self
    ) -> Result<ServiceWriteGuard<Svc, Base>, LockError<&'static Svc::Key>>
        where Svc: ServiceReflect, Key: Borrow<Svc::Key>, Base: Downcast<Svc>
    {
        self.invoke::<Write<Svc>>(())
    }

    /// Shortcut for `.invoke::<ioc::Create<{Obj}>>(args)`.
    pub fn create<'a, Obj>(
        &'a self, 
        args: <Obj::Factory as Factory<Obj>>::Args,
    ) -> Result<Obj, CreationError<&'static <Obj::Factory as ServiceReflect>::Key, <Obj::Factory as Factory<Obj>>::Error>>  
    where 
        Obj: FactoryObject, 
        Obj::Factory: ServiceReflect, 
        Key: Borrow<<Obj::Factory as ServiceReflect>::Key>,
        Base: Downcast<Obj::Factory>,
    {
        self.invoke::<Create<Obj>>(args)
    }

    /// Shortcut for `.invoke::<ioc::ReadAll>(())`.
    pub fn read_all(&self) -> Result<ServiceReadGuardMap<Key, Base>, LockError<&Key>> {
        self.invoke::<ReadAll>(())
    }

    /// Shortcut for `.invoke::<ioc::WriteAll>(())`.
    pub fn write_all(&self) -> Result<ServiceWriteGuardMap<Key, Base>, LockError<&Key>> {
        self.invoke::<WriteAll>(())
    }
}

// ++++++++++++++++++++ IocBuilder ++++++++++++++++++++

/// TODO This type needs some ironing out.
pub struct IocBuilder<Key = str, Base: ?Sized = DefaultBase> {
    services: BTreeMap<Key, RwLock<Box<Base>>>,
}

impl<Key, Base: ?Sized> IocBuilder<Key, Base>
    where Key: Debug + Ord, Base: Any
{
    pub fn new() -> Self {
        IocBuilder{ services: BTreeMap::new() }
    }

    pub fn register_service(&mut self, key: Key, svc: Box<Base>) -> &mut Self {
        self.services.insert(key, RwLock::new(svc));
        self
    }

    /// NOTE: The `Box<Svc>: Into<Box<Base>>`-clause is needed due to rusts lack of 
    /// HKT or a `Coercible`-trait (to name two solutions).
    pub fn register<Svc>(&mut self, svc: Svc) -> &mut Self
        where Svc: ServiceReflect, 
              &'static Svc::Key: Into<Key>, 
              Key: Borrow<Svc::Key>, //TODO why is this line required?
              Box<Svc>: Into<Box<Base>>,
    {
        let key = Svc::key();
        self.register_service(key.into(), Box::new(svc).into())
    }

    pub fn build(self) -> Ioc<Key, Base> {
        Ioc::new(self.services)
    }
}

impl<Key, Base: ?Sized> Default for IocBuilder<Key, Base>
    where Key: Debug + Ord, Base: Any
{
    fn default() -> Self { Self::new() }
}

