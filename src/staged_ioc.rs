use error::Error;
use container::{Container, StagedContainer};
use ioc::Ioc;
use reflect;

use std::any::Any;
use std::collections::BTreeMap;
use std::cell::{self, RefCell};
use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::{slice, iter};

// ++++++++++++++++++++ StagedIoc ++++++++++++++++++++

pub struct StagedIoc<Key, SvcLock> {
    stages: Vec<(Key, Ioc<Key, SvcLock>)>,
}


impl<Key, SvcLock> StagedIoc<Key, SvcLock> 
    where Key: reflect::Key
{
    #[doc(hidden)]
    pub fn new() -> Self {
        StagedIoc{ stages: Vec::new() }
    }

    #[doc(hidden)]
    pub fn register_stage(&mut self, pos: usize, key: Key){
        self.unregister_stage(&key);

        self.stages.insert(pos, (key, Ioc::new()));
    }

    #[doc(hidden)]
    pub fn unregister_stage(&mut self, key: &Key) -> bool {
        let len = self.stages.len();
        self.stages.retain(|&(ref k, _)| k != key);
        self.stages.len() != len
    }

    #[doc(hidden)]
    pub fn register_service(&mut self, stage_key: &Key, svc_key: Key, svc: SvcLock){
        self.unregister_service(&svc_key);

        for &mut (ref k, ref mut stage) in &mut self.stages {
            if k == stage_key { stage.register_service(svc_key, svc); return }
        }
        panic!("missing stage '{:?}'!", stage_key);
    }

    #[doc(hidden)]
    pub fn unregister_service(&mut self, key: &Key) -> bool {
        for &mut (_, ref mut stage) in &mut self.stages {
            if stage.unregister_service(&key) { return true; }
        }
        false
    }

    pub fn get_stage(&self, key: &Key) -> Option<&Ioc<Key, SvcLock>> {
        for &(ref k, ref stage) in &self.stages {
            if k == key { return Some(stage); }
        }
        None
    }

    pub fn get_service(&self, key: &Key) -> Option<&SvcLock> {
        for &(_, ref stage) in &self.stages {
            match stage.get_service(key) {
                Some(r) => return Some(r),
                None => {}
            }
        }
        None
    }
}

impl<'a, Key, SvcBase: ?Sized> Container<'a> for StagedIoc<Key, RwLock<Box<SvcBase>>>
    where Key: reflect::Key, SvcBase: Any
{
    type Key = Key;
    type ServiceBase = SvcBase;

    type ReadGuardBase = RwLockReadGuard<'a, Box<SvcBase>>;
    type WriteGuardBase = RwLockWriteGuard<'a, Box<SvcBase>>;

    //type ReadGuardMap
    //type WriteGuardMap

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
        for &(_, ref stage) in &self.stages {
            for (key, _) in stage.services() {
                ret.insert(key, try!{self.read_service_base(key)});
            }
        }
        Ok(ret)
    }

    fn write_all(
        &'a self, 
    ) -> Result<BTreeMap<&'a Self::Key, Self::WriteGuardBase>, Error<'a, Self::Key>> {
        let mut ret = BTreeMap::new();
        for &(_, ref stage) in &self.stages {
            for (key, _) in stage.services() {
                ret.insert(key, try!{self.write_service_base(key)});
            }
        }
        Ok(ret)
    }
}

impl<'a, Key, SvcBase: ?Sized> Container<'a> for StagedIoc<Key, RefCell<Box<SvcBase>>>
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
        for &(_, ref stage) in &self.stages {
            for (key, _) in stage.services() {
                ret.insert(key, try!{self.read_service_base(key)});
            }
        }
        Ok(ret)
    }

    fn write_all(
        &'a self, 
    ) -> Result<BTreeMap<&'a Self::Key, Self::WriteGuardBase>, Error<'a, Self::Key>> {
        let mut ret = BTreeMap::new();
        for &(_, ref stage) in &self.stages {
            for (key, _) in stage.services() {
                ret.insert(key, try!{self.write_service_base(key)});
            }
        }
        Ok(ret)
    }
}

pub type StageIter<'a, Key, SvcLock> = iter::Map<
    slice::Iter<'a, (Key, Ioc<Key, SvcLock>)>, 
    fn(&'a (Key, Ioc<Key, SvcLock>)) -> (&'a Key, &'a Ioc<Key, SvcLock>)
>;

impl<'a, Key, SvcBase: ?Sized> StagedContainer<'a> for StagedIoc<Key, RwLock<Box<SvcBase>>> 
    where Key: reflect::Key, SvcBase: Any
{
    type Stage = Ioc<Key, RwLock<Box<SvcBase>>>;

    type StageIter = StageIter<'a, Key, RwLock<Box<SvcBase>>>;

    fn stages(&'a self) -> Self::StageIter {
        fn map_fn<'a, A, B>(pair: &'a (A, B)) -> (&'a A, &'a B) {
            (&pair.0, &pair.1)
        } 

        self.stages.iter().map(map_fn)
    }
}

impl<'a, Key, SvcBase: ?Sized> StagedContainer<'a> for StagedIoc<Key, RefCell<Box<SvcBase>>> 
    where Key: reflect::Key, SvcBase: Any
{
    type Stage = Ioc<Key, RefCell<Box<SvcBase>>>;

    type StageIter = StageIter<'a, Key, RefCell<Box<SvcBase>>>;

    fn stages(&'a self) -> Self::StageIter {
        fn map_fn<'a, A, B>(pair: &'a (A, B)) -> (&'a A, &'a B) {
            (&pair.0, &pair.1)
        } 

        self.stages.iter().map(map_fn)
    }
}

// ++++++++++++++++++++ StagedIocBuilder ++++++++++++++++++++

pub struct StagedIocBuilder<Key, SvcLock> {
    ioc: StagedIoc<Key, SvcLock>
}

pub struct StageBuilder<'a, Key: 'a, SvcLock: 'a> {
    ioc: &'a mut StagedIoc<Key, SvcLock>,
    stage: Key,
}

/// NOTE: if `RwLock<T>` were to implement `From<T>`, we could write a more generic impl... 
/// keep an eye open for any changes...
impl<'a, Key, SvcBase: ?Sized> StageBuilder<'a, Key, RwLock<Box<SvcBase>>> 
    where Key: reflect::Key, SvcBase: Any
{
    pub fn register_service(&mut self, key: Key, svc: Box<SvcBase>) -> &mut Self {
        self.ioc.register_service(&self.stage, key, RwLock::new(svc));
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
impl<'a, Key, SvcBase: ?Sized> StageBuilder<'a, Key, RefCell<Box<SvcBase>>> 
    where Key: reflect::Key, SvcBase: Any
{
    pub fn register_service(&mut self, key: Key, svc: Box<SvcBase>) -> &mut Self {
        self.ioc.register_service(&self.stage, key, RefCell::new(svc));
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

impl<Key, SvcLock> StagedIocBuilder<Key, SvcLock>
    where Key: reflect::Key, 
{
    pub fn new() -> Self {
        StagedIocBuilder{ ioc: StagedIoc::new() }
    }

    pub fn at_stage(&mut self, stage: &Key) -> StageBuilder<Key, SvcLock> {
        assert!(self.ioc.get_stage(stage).is_some());
        StageBuilder{ ioc: &mut self.ioc, stage: stage.clone() }
    }

    pub fn at<St>(&mut self) -> StageBuilder<Key, SvcLock>
        where St: reflect::Stage<Key = Key>
    {
        self.at_stage(St::key())
    }

    pub fn register_stage_at_begin(&mut self, stage: Key){
        self.ioc.register_stage(0, stage);
    }

    pub fn register_stage_at_end(&mut self, stage: Key){
        let len = self.ioc.stages.len();
        self.ioc.register_stage(len, stage);
    }

    pub fn register_stage_before(&mut self, pos_stage: &Key, stage: Key){
        let pos = self.ioc.stages.iter().position(|&(ref key, _)| key == pos_stage).unwrap();
        self.ioc.register_stage(pos, stage);
    }

    pub fn register_stage_after(&mut self, pos_stage: &Key, stage: Key){
        let pos = self.ioc.stages.iter().position(|&(ref key, _)| key == pos_stage).unwrap();
        self.ioc.register_stage(pos + 1, stage);
    }

    pub fn register_at_begin<St>(&mut self)
        where St: reflect::Stage<Key = Key>
    {
        self.register_stage_at_begin(St::key().clone())
    }

    pub fn register_at_end<St>(&mut self)
        where St: reflect::Stage<Key = Key>
    {
        self.register_stage_at_end(St::key().clone())
    }

    pub fn register_before<PosSt, St>(&mut self)
        where PosSt: reflect::Stage<Key = Key>, St: reflect::Stage<Key = Key>
    {
        self.register_stage_before(PosSt::key(), St::key().clone())
    }

    pub fn register_after<PosSt, St>(&mut self)
        where PosSt: reflect::Stage<Key = Key>, St: reflect::Stage<Key = Key>
    {
        self.register_stage_after(PosSt::key(), St::key().clone())
    }

    pub fn build(self) -> StagedIoc<Key, SvcLock> {
        self.ioc
    }
}

impl<Key, SvcLock> Default for StagedIocBuilder<Key, SvcLock>
    where Key: reflect::Key 
{
    fn default() -> Self { Self::new() }
}


