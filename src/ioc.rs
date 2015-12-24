use errors::Error;
use container::Container;
use reflect;

use std::any::Any;
use std::collections::{btree_map, BTreeMap};
use std::cell::{self, RefCell};
use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

// ++++++++++++++++++++ Ioc ++++++++++++++++++++

pub struct Ioc<Key, SvcLock> {
    services: BTreeMap<Key, SvcLock>,
}

impl<Key, SvcLock> Ioc<Key, SvcLock> 
    where Key: reflect::Key
{
    #[doc(hidden)]
    pub fn new() -> Self {
        Ioc{ services: BTreeMap::new() }
    }

    #[doc(hidden)]
    pub fn register_service(&mut self, key: Key, svc: SvcLock){
        self.services.insert(key, svc);
    }

    #[doc(hidden)]
    pub fn unregister_service(&mut self, key: &Key) -> bool {
        self.services.remove(key).is_some()
    }

    pub fn services(&self) -> btree_map::Iter<Key, SvcLock> { 
        self.services.iter()
    }

    pub fn get_service(&self, key: &Key) -> Option<&SvcLock> {
        self.services.get(key)
    }
}

impl<'a, Key, SvcBase: ?Sized> Container<'a> for Ioc<Key, RwLock<Box<SvcBase>>>
    where Key: reflect::Key, SvcBase: Any
{
    type Key = Key;
    type ServiceBase = SvcBase;

    type ReadGuardBase = RwLockReadGuard<'a, Box<SvcBase>>;
    type WriteGuardBase = RwLockWriteGuard<'a, Box<SvcBase>>;

    fn read_service_base(
        &'a self, 
        key: &'a Self::Key
    ) -> Result<Self::ReadGuardBase, Error<'a, Self::Key>> {
        match self.get_service(key) {
            Some(service) => match service.read() {
                Ok(r) => Ok(r),
                Err(_) => Err(Error::Poisoned{ key: key })
            },
            None => Err(Error::NotFound{ key: key })
        }
    }

    fn write_service_base(
        &'a self, 
        key: &'a Self::Key
    ) -> Result<Self::WriteGuardBase, Error<'a, Self::Key>> {
        match self.get_service(key) {
            Some(service) => match service.write() {
                Ok(r) => Ok(r),
                Err(_) => Err(Error::Poisoned{ key: key })
            },
            None => Err(Error::NotFound{ key: key })
        }
    }

    fn read_all(
        &'a self, 
    ) -> Result<BTreeMap<&'a Self::Key, Self::ReadGuardBase>, Error<'a, Self::Key>> {
        let mut ret = BTreeMap::new();
        for (key, _) in self.services() {
            ret.insert(key, try!{self.read_service_base(key)});
        }
        Ok(ret)
    }

    fn write_all(
        &'a self, 
    ) -> Result<BTreeMap<&'a Self::Key, Self::WriteGuardBase>, Error<'a, Self::Key>> {
        let mut ret = BTreeMap::new();
        for (key, _) in self.services() {
            ret.insert(key, try!{self.write_service_base(key)});
        }
        Ok(ret)
    }
}

impl<'a, Key, SvcBase: ?Sized> Container<'a> for Ioc<Key, RefCell<Box<SvcBase>>>
    where Key: reflect::Key, SvcBase: Any
{
    type Key = Key;
    type ServiceBase = SvcBase;

    type ReadGuardBase = cell::Ref<'a, Box<SvcBase>>;
    type WriteGuardBase = cell::RefMut<'a, Box<SvcBase>>;

    fn read_service_base(
        &'a self, 
        key: &'a Self::Key
    ) -> Result<Self::ReadGuardBase, Error<'a, Self::Key>> {
        match self.get_service(key) {
            Some(service) => Ok(service.borrow()),
            None => Err(Error::NotFound{ key: key })
        }
    }

    fn write_service_base(
        &'a self, 
        key: &'a Self::Key
    ) -> Result<Self::WriteGuardBase, Error<'a, Self::Key>> {
        match self.get_service(key) {
            Some(service) => Ok(service.borrow_mut()),
            None => Err(Error::NotFound{ key: key })
        }
    }

    fn read_all(
        &'a self, 
    ) -> Result<BTreeMap<&'a Self::Key, Self::ReadGuardBase>, Error<'a, Self::Key>> {
        let mut ret = BTreeMap::new();
        for (key, _) in self.services() {
            ret.insert(key, try!{self.read_service_base(key)});
        }
        Ok(ret)
    }

    fn write_all(
        &'a self, 
    ) -> Result<BTreeMap<&'a Self::Key, Self::WriteGuardBase>, Error<'a, Self::Key>> {
        let mut ret = BTreeMap::new();
        for (key, _) in self.services() {
            ret.insert(key, try!{self.write_service_base(key)});
        }
        Ok(ret)
    }
}

// ++++++++++++++++++++ IocBuilder ++++++++++++++++++++

pub struct IocBuilder<Key, SvcLock> {
    ioc: Ioc<Key, SvcLock>
}

impl<Key, SvcLock> IocBuilder<Key, SvcLock>
    where Key: reflect::Key
{
    pub fn new() -> Self {
        IocBuilder{ ioc: Ioc::new() }
    }

    pub fn build(self) -> Ioc<Key, SvcLock> {
        self.ioc
    }
}

/// NOTE: if `RwLock<T>` were to implement `From<T>`, we could write a more generic impl... 
/// keep an eye open for any changes...
impl<Key, SvcBase: ?Sized> IocBuilder<Key, RwLock<Box<SvcBase>>> 
    where Key: reflect::Key, SvcBase: Any
{
    pub fn register_service(&mut self, key: Key, svc: Box<SvcBase>) -> &mut Self {
        self.ioc.register_service(key, RwLock::new(svc));
        self
    }

    /// NOTE: The `Box<Svc>: Into<Box<Base>>`-clause is needed due to rusts lack of 
    /// HKT or a `Coercible`-trait (to name two solutions).
    pub fn register<Svc>(&mut self, svc: Svc) -> &mut Self
    where
        Svc: reflect::Service<Key = Key>,
        Box<Svc>: Into<Box<SvcBase>>,
    {
        self.register_service(Svc::key().clone(), Box::new(svc).into())
    }
}

/// NOTE: if `RefCell<T>` were to implement `From<T>`, we could write a more generic impl... 
/// keep an eye open for any changes...
impl<Key, SvcBase: ?Sized> IocBuilder<Key, RefCell<Box<SvcBase>>> 
    where Key: reflect::Key, SvcBase: Any
{
    pub fn register_service(&mut self, key: Key, svc: Box<SvcBase>) -> &mut Self {
        self.ioc.register_service(key, RefCell::new(svc));
        self
    }

    /// NOTE: The `Box<Svc>: Into<Box<Base>>`-clause is needed due to rusts lack of 
    /// HKT or a `Coercible`-trait (to name two solutions).
    pub fn register<Svc>(&mut self, svc: Svc) -> &mut Self
    where
        Svc: reflect::Service<Key = Key>,
        Box<Svc>: Into<Box<SvcBase>>,
    {
        self.register_service(Svc::key().clone(), Box::new(svc).into())
    }
}

impl<Key, SvcLock> Default for IocBuilder<Key, SvcLock>
    where Key: reflect::Key
{
    fn default() -> Self { Self::new() }
}

