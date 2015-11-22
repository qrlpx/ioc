use service::*;
use factory::*;
use invocation_method::*;

use downcast::Downcast;

use std::any::Any;
use std::collections::BTreeMap;
use std::sync::{Mutex, RwLock};

// ++++++++++++++++++++ Container ++++++++++++++++++++

/// TODO naming?
pub struct Container<Key = str, Base: ?Sized = DefaultBase> {
    services: BTreeMap<Key, RwLock<Box<Base>>>,

    deadlock_prevention: Mutex<()>, 
}

impl<Key, Base: ?Sized> Container<Key, Base> 
    where Key: ServiceKey, Base: Any
{
    fn new(services: BTreeMap<Key, RwLock<Box<Base>>>) -> Self {
        Container{ 
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
    pub fn read<Svc>(&self) -> Result<ServiceReadGuard<Svc, Base>, LockError<Svc::Key>>
        where Svc: ServiceReflect<Key = Key>, Base: Downcast<Svc>
    {
        self.invoke::<Read<Svc>>(())
    }

    /// Shortcut for `.invoke::<ioc::Write<{Svc}>>(())`.
    pub fn write<Svc>(&self) -> Result<ServiceWriteGuard<Svc, Base>, LockError<Svc::Key>>
        where Svc: ServiceReflect<Key = Key>, Base: Downcast<Svc>
    {
        self.invoke::<Write<Svc>>(())
    }

    /// Shortcut for `.invoke::<ioc::Create<{Obj}>>(args)`.
    pub fn create<Obj>(
        &self, 
        args: <Obj::Factory as Factory<Obj>>::Args,
    ) -> Result<Obj, CreationError<<Obj::Factory as ServiceReflect>::Key, <Obj::Factory as Factory<Obj>>::Error>>  
    where 
        Obj: FactoryObject, 
        Obj::Factory: ServiceReflect<Key = Key>, 
        Base: Downcast<Obj::Factory>,
    {
        self.invoke::<Create<Obj>>(args)
    }

    /// Shortcut for `.invoke::<ioc::ReadAll>(())`.
    pub fn read_all(&self) -> Result<ServiceReadGuardMap<Key, Base>, LockError<Key>> {
        self.invoke::<ReadAll>(())
    }

    /// Shortcut for `.invoke::<ioc::WriteAll>(())`.
    pub fn write_all(&self) -> Result<ServiceWriteGuardMap<Key, Base>, LockError<Key>> {
        self.invoke::<WriteAll>(())
    }
}

// ++++++++++++++++++++ ContainerBuilder ++++++++++++++++++++

/// TODO This type needs some ironing out.
pub struct ContainerBuilder<Key = str, Base: ?Sized = DefaultBase> {
    services: BTreeMap<Key, RwLock<Box<Base>>>,
}

impl<Key, Base: ?Sized> ContainerBuilder<Key, Base>
    where Key: ServiceKey, Base: Any
{
    pub fn new() -> Self {
        ContainerBuilder{ services: BTreeMap::new() }
    }

    pub fn register_service(&mut self, key: Key, svc: Box<Base>) -> &mut Self {
        self.services.insert(key, RwLock::new(svc));
        self
    }

    /// NOTE: The `Box<Svc>: Into<Box<Base>>`-clause is needed due to rusts lack of 
    /// HKT or a `Coercible`-trait (to name two solutions).
    pub fn register<Svc>(&mut self, svc: Svc) -> &mut Self
        where Svc: ServiceReflect<Key = Key>, Box<Svc>: Into<Box<Base>>,
    {
        let key = Svc::key();
        self.register_service(key.clone(), Box::new(svc).into())
    }

    pub fn build(self) -> Container<Key, Base> {
        Container::new(self.services)
    }
}

impl<Key, Base: ?Sized> Default for ContainerBuilder<Key, Base>
    where Key: ServiceKey, Base: Any
{
    fn default() -> Self { Self::new() }
}

