use errors::Error;
use guards::{ReadGuard, WriteGuard};
use methods::Method;
use reflect;

use downcast::Downcast;
//use shared_mutex::{SharedMutex, SharedMutexReadGuard, SharedMutexWriteGuard};

use std::any::Any;
use std::collections::BTreeMap;
use std::sync::{TryLockError, RwLock, RwLockReadGuard, RwLockWriteGuard};

fn type_name<T: Any>() -> &'static str {
    "NOT IMPLEMENTED"
    //unsafe { ::std::intrinsics::type_name::<T>() }
}

// ++++++++++++++++++++ Container ++++++++++++++++++++

pub struct Container<Key, SvcBase: ?Sized> {
    services: BTreeMap<Key, RwLock<Box<SvcBase>>>,
}

impl<Key, SvcBase: ?Sized> Container<Key, SvcBase> 
    where Key: reflect::Key, SvcBase: Any
{
    #[doc(hidden)]
    pub fn new() -> Self {
        Container{ services: BTreeMap::new() }
    }

    #[doc(hidden)]
    pub fn register_service(&mut self, key: Key, svc: Box<SvcBase>) -> &mut Self {
        self.services.insert(key, RwLock::new(svc));
        self
    }

    #[doc(hidden)]
    pub fn register<Svc>(&mut self, svc: Svc) -> &mut Self
    where
        Svc: reflect::Service<Key = Key>,
        Box<Svc>: Into<Box<SvcBase>>,
    {
        self.register_service(Svc::key().clone(), Box::new(svc).into())
    }

    pub fn services(&self) -> &BTreeMap<Key, RwLock<Box<SvcBase>>> {
        &self.services
    }
    
    pub fn get_service(&self, key: &Key) -> Option<&RwLock<Box<SvcBase>>> {
        self.services.get(key)
    }

    pub fn read_service_base<'a>(
        &'a self, 
        key: &'a Key
    ) -> Result<RwLockReadGuard<Box<SvcBase>>, Error<'a, Key>> {
        match self.get_service(key) {
            Some(service) => match service.read() {
                Ok(r) => Ok(r),
                Err(_) => Err(Error::Poisoned{ key: key })
            },
            None => Err(Error::NotFound{ key: key })
        }
    }

    pub fn write_service_base<'a>(
        &'a self, 
        key: &'a Key
    ) -> Result<RwLockWriteGuard<Box<SvcBase>>, Error<'a, Key>> {
        match self.get_service(key) {
            Some(service) => match service.write() {
                Ok(r) => Ok(r),
                Err(_) => Err(Error::Poisoned{ key: key })
            },
            None => Err(Error::NotFound{ key: key })
        }
    }

    pub fn read_service<'a, Svc>(
        &'a self, 
        key: &'a Key
    ) -> Result<ReadGuard<Svc, SvcBase>, Error<'a, Key>>
        where Svc: Any, SvcBase: Downcast<Svc>
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

    pub fn write_service<'a, Svc>(
        &'a self, 
        key: &'a Key
    ) -> Result<WriteGuard<Svc, SvcBase>, Error<'a, Key>>
        where Svc: Any, SvcBase: Downcast<Svc>
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

    pub fn read<'a, Svc>(
        &'a self
    ) -> Result<ReadGuard<Svc, SvcBase>, Error<'a, Key>>
        where Svc: reflect::Service<Key = Key>, SvcBase: Downcast<Svc>
    {
        self.read_service(Svc::key())
    }

    pub fn write<'a, Svc>(
        &'a self
    ) -> Result<WriteGuard<Svc, SvcBase>, Error<'a, Key>>
        where Svc: reflect::Service<Key = Key>, SvcBase: Downcast<Svc>
    {
        self.write_service(Svc::key())
    }

    pub fn try_read_service_base<'a>(
        &'a self, 
        key: &'a Key
    ) -> Result<RwLockReadGuard<Box<SvcBase>>, Error<'a, Key>> {
        match self.get_service(key) {
            Some(service) => match service.try_read() {
                Ok(r) => Ok(r),
                Err(TryLockError::Poisoned(_)) => Err(Error::Poisoned{ key: key }),
                Err(TryLockError::WouldBlock) => Err(Error::WouldBlock{ key: key })
            },
            None => Err(Error::NotFound{ key: key })
        }
    }

    pub fn try_write_service_base<'a>(
        &'a self, 
        key: &'a Key
    ) -> Result<RwLockWriteGuard<Box<SvcBase>>, Error<'a, Key>> {
        match self.get_service(key) {
            Some(service) => match service.try_write() {
                Ok(r) => Ok(r),
                Err(TryLockError::Poisoned(_)) => Err(Error::Poisoned{ key: key }),
                Err(TryLockError::WouldBlock) => Err(Error::WouldBlock{ key: key })
            },
            None => Err(Error::NotFound{ key: key })
        }
    }

    pub fn try_read_service<'a, Svc>(
        &'a self, 
        key: &'a Key
    ) -> Result<ReadGuard<Svc, SvcBase>, Error<'a, Key>>
        where Svc: Any, SvcBase: Downcast<Svc>
    {
        let base = try!{self.try_read_service_base(key)};
        if !base.is_type() {
            return Err(Error::MismatchedType{ 
                key: key, 
                expected: type_name::<Svc>(),
                found: type_name::<Svc>(),
            })
        };
        Ok(ReadGuard::new(base))
    }

    pub fn try_write_service<'a, Svc>(
        &'a self, 
        key: &'a Key
    ) -> Result<WriteGuard<Svc, SvcBase>, Error<'a, Key>>
        where Svc: Any, SvcBase: Downcast<Svc>
    {
        let base = try!{self.try_write_service_base(key)};
        if !base.is_type() {
            return Err(Error::MismatchedType{ 
                key: key, 
                expected: type_name::<Svc>(),
                found: type_name::<Svc>(),
            })
        };
        Ok(WriteGuard::new(base))
    }

    pub fn try_read<'a, Svc>(
        &'a self
    ) -> Result<ReadGuard<Svc, SvcBase>, Error<'a, Key>>
        where Svc: reflect::Service<Key = Key>, SvcBase: Downcast<Svc>
    {
        self.try_read_service(Svc::key())
    }

    pub fn try_write<'a, Svc>(
        &'a self
    ) -> Result<WriteGuard<Svc, SvcBase>, Error<'a, Key>>
        where Svc: reflect::Service<Key = Key>, SvcBase: Downcast<Svc>
    {
        self.try_write_service(Svc::key())
    }

    pub fn resolve<'a, M>(&'a self, method: M) -> Result<M::Ret, Error<Key>>
        where M: Method<'a, Key, SvcBase>
    {
        method.resolve(self)
    }
    pub fn try_resolve<'a, M>(&'a self, method: M) -> Result<M::Ret, Error<Key>>
        where M: Method<'a, Key, SvcBase>
    {
        method.try_resolve(self)
    }
}

// ++++++++++++++++++++ ContainerBuilder ++++++++++++++++++++

pub struct ContainerBuilder<Key, SvcBase: ?Sized> {
    cont: Container<Key, SvcBase>
}

impl<Key, SvcBase: ?Sized> ContainerBuilder<Key, SvcBase>
    where Key: reflect::Key, SvcBase: Any
{
    pub fn new() -> Self {
        ContainerBuilder{ cont: Container::new() }
    }

    pub fn register_service(&mut self, key: Key, svc: Box<SvcBase>) -> &mut Self {
        self.cont.register_service(key, svc);
        self
    }

    /// NOTE: The `Box<Svc>: Into<Box<Base>>`-clause is needed due to rusts lack of 
    /// HKT or a `Coercible`-trait (to name two solutions).
    pub fn register<Svc>(&mut self, svc: Svc) -> &mut Self
    where
        Svc: reflect::Service<Key = Key>,
        Box<Svc>: Into<Box<SvcBase>>,
    {
        self.cont.register::<Svc>(svc);
        self
    }

    pub fn build(self) -> Container<Key, SvcBase> {
        self.cont
    }
}

impl<Key, SvcBase: ?Sized> Default for Container<Key, SvcBase> 
    where Key: reflect::Key, SvcBase: Any
{
    fn default() -> Self { Self::new() }
}
