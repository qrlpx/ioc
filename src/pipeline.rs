use guards::*;
use errors::*;
use factory::*;
use methods::*;
use container::Container;
use reflect;

use downcast::Downcast;

use std::any::Any;
use std::sync::{RwLock, Mutex};
use std::{iter, slice};

pub struct Pipeline<Key, Base: ?Sized> {
    stages: Vec<(Key, Container<Key, Base>)>,
    abba_prevention: Mutex<()>, 
}

pub type Stages<'a, Key, Base: ?Sized> = iter::Map<
    slice::Iter<'a, (Key, Container<Key, Base>)>, 
    fn(&'a (Key, Container<Key, Base>)) -> (&'a Key, &'a Container<Key, Base>)
>;

impl<Key, Base: ?Sized> Pipeline<Key, Base> 
    where Key: reflect::Key, Base: Any
{
    #[doc(hidden)]
    pub fn new() -> Self {
        Pipeline{ 
            stages: Vec::new(),
            abba_prevention: Mutex::new(()),
        }
    }

    #[doc(hidden)]
    pub fn register_stage(&mut self, pos: usize, key: Key){
        self.unregister_stage(&key);

        self.stages.insert(pos, (key, Container::new()));
    }

    #[doc(hidden)]
    pub fn unregister_stage(&mut self, key: &Key) -> bool {
        let len = self.stages.len();
        self.stages.retain(|&(ref k, _)| k != key);
        self.stages.len() != len
    }

    #[doc(hidden)]
    pub fn register_service(&mut self, stage_key: &Key, svc_key: Key, svc: Box<Base>){
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

    pub fn stages(&self) -> Stages<Key, Base> {
        fn map_fn<'a, A, B>(pair: &'a (A, B)) -> (&'a A, &'a B) {
            (&pair.0, &pair.1)
        } 

        self.stages.iter().map(map_fn)
    }

    pub fn get_stage(&self, key: &Key) -> Option<&Container<Key, Base>> {
        for &(ref k, ref stage) in &self.stages {
            if k == key { return Some(stage); }
        }
        None
    }

    pub fn get_service(&self, key: &Key) -> Option<&RwLock<Box<Base>>> {
        for &(_, ref stage) in &self.stages {
            match stage.get_service(key) {
                Some(r) => return Some(r),
                None => {}
            }
        }
        None
    }

    pub fn invoke<'a, M>(&'a self) -> Result<M::Ret, M::Error>
        where M: Method<'a, Self>, 
    {
        // prevent AB-BA deadlocks
        let _ = self.abba_prevention.lock();

        M::invoke(self)
    }
}

impl<'a, Key, Base: ?Sized> Method<'a, Pipeline<Key, Base>> for ()
    where Key: reflect::Key, Base: Any
{
    type Ret = ();
    type Error = DummyError;

    fn invoke(_: &'a Pipeline<Key, Base>) -> Result<Self::Ret, Self::Error> {
        Ok(())
    }
}

impl<'a, Key, Base: ?Sized, Svc> Method<'a, Pipeline<Key, Base>> for Read<Svc>
    where Key: reflect::Key, Base: Any + Downcast<Svc>, Svc: reflect::Object<Key = Key>
{
    type Ret = ReadGuard<'a, Svc, Base>;
    type Error = LockError<'a, Key>;

    fn invoke(ioc: &'a Pipeline<Key, Base>) -> Result<Self::Ret, Self::Error> {
        for (_, stage) in ioc.stages() {
            match stage.invoke::<Self>() {
                Ok(r) => return Ok(r),
                Err(LockError::NotFound{ .. }) => continue,
                Err(err) => return Err(err)
            }
        }
        Err(LockError::NotFound{ key: Svc::key() })
    }
}

impl<'a, Key, Base: ?Sized, Svc> Method<'a, Pipeline<Key, Base>> for Write<Svc>
    where Key: reflect::Key, Base: Any + Downcast<Svc>, Svc: reflect::Object<Key = Key>
{
    type Ret = WriteGuard<'a, Svc, Base>;
    type Error = LockError<'a, Key>;

    fn invoke(ioc: &'a Pipeline<Key, Base>) -> Result<Self::Ret, Self::Error> {
        for (_, stage) in ioc.stages() {
            match stage.invoke::<Self>() {
                Ok(r) => return Ok(r),
                Err(LockError::NotFound{ .. }) => continue,
                Err(err) => return Err(err)
            }
        }
        Err(LockError::NotFound{ key: Svc::key() })
    }
}

impl<'a, Key, Base: ?Sized> Method<'a, Pipeline<Key, Base>> for ReadAll
    where Key: reflect::Key, Base: Any
{
    type Ret = ReadGuardMap<'a, Key, Base>;
    type Error = LockError<'a, Key>;

    fn invoke(ioc: &'a Pipeline<Key, Base>) -> Result<Self::Ret, Self::Error> {
        let mut map = ReadGuardMap::new();
        for (_, stage) in ioc.stages() {
            for (svc_key, service) in stage.services() {
                let guard = match service.read() {
                    Ok(r) => r, Err(_) => return Err(LockError::Poisoned{ key: svc_key })
                };
                map.insert(svc_key, guard);
            }
        }
        Ok(map)
    }
}

impl<'a, Key, Base: ?Sized> Method<'a, Pipeline<Key, Base>> for WriteAll
    where Key: reflect::Key, Base: Any
{
    type Ret = WriteGuardMap<'a, Key, Base>;
    type Error = LockError<'a, Key>;

    fn invoke(ioc: &'a Pipeline<Key, Base>) -> Result<Self::Ret, Self::Error> {
        let mut map = WriteGuardMap::new();
        for (_, stage) in ioc.stages() {
            for (svc_key, service) in stage.services() {
                let guard = match service.write() {
                    Ok(r) => r, Err(_) => return Err(LockError::Poisoned{ key: svc_key })
                };
                map.insert(svc_key, guard);
            }
        }
        Ok(map)
    }
}

impl<'a, Key, Base: ?Sized, Obj> Method<'a, Pipeline<Key, Base>> for Create<Obj> 
where 
    Key: reflect::Key, 
    Base: Any + Downcast<Obj::Factory>,
    Obj: FactoryObject<Key = Key> + 'a, 
    Obj::Factory: Factory<'a, Pipeline<Key, Base>, Obj>
{
    type Ret = Obj;
    type Error = CreationError<'a, 
        <Obj::Factory as reflect::Object>::Key, 
        <Obj::Factory as Factory<'a, Pipeline<Key, Base>, Obj>>::Error
    >;

    fn invoke(ioc: &'a Pipeline<Key, Base>) -> Result<Self::Ret, Self::Error> {
        // NOTE: this is a 1:1 copy of Containers-impl. 

        let factory = try!{Read::<Obj::Factory>::invoke(ioc)};
        
        let key = <Obj::Factory as reflect::Object>::key();
        let args = match <<Obj::Factory as Factory<'a, _, _>>::Args as Method<'a, _>>::invoke(ioc) {
            Ok(args) => args,
            Err(err) => return Err(CreationError::DependencyError{ key: key, error: box err })
        };
        match factory.create(args){
            Ok(r) => Ok(r), 
            Err(err) => Err(CreationError::CreationError{ key: key, error: err })
        }
    }
}

impl<Key, Base: ?Sized> Pipeline<Key, Base> 
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
    pub fn create<'a, Obj>(&'a self) -> Result<Obj, CreationError<<Obj::Factory as reflect::Object>::Key, <Obj::Factory as Factory<'a, Self, Obj>>::Error>>  
    where 
        Obj: FactoryObject<Key = Key> + 'a,
        Obj::Factory: Factory<'a, Self, Obj>,
        Base: Downcast<Obj::Factory>,
    {
        self.invoke::<Create<Obj>>()
    }
}

// ++++++++++++++++++++ PipelineBuilder ++++++++++++++++++++

pub struct PipelineBuilder<Key, Base: ?Sized> {
    ioc: Pipeline<Key, Base>,
}

pub struct StageBuilder<'a, Key: 'a, Base: 'a + ?Sized> {
    ioc: &'a mut Pipeline<Key, Base>,
    stage: Key,
}

impl<'a, Key, Base: ?Sized> StageBuilder<'a, Key, Base>
    where Key: reflect::Key, Base: Any
{
    pub fn register_service(&mut self, key: Key, svc: Box<Base>) -> &mut Self {
        self.ioc.register_service(&self.stage, key, svc);
        self
    }

    /// NOTE: The `Box<Svc>: Into<Box<Base>>`-clause is needed due to rusts lack of 
    /// HKT or a `Coercible`-trait (to name two solutions).
    pub fn register<Svc>(&mut self, svc: Svc) -> &mut Self
        where Svc: reflect::Object<Key = Key>, Box<Svc>: Into<Box<Base>>,
    {
        self.register_service(Svc::key().clone(), Box::new(svc).into())
    }
}

impl<Key, Base: ?Sized> PipelineBuilder<Key, Base>
    where Key: reflect::Key, Base: Any
{
    pub fn new() -> Self {
        PipelineBuilder{ ioc: Pipeline::new() }
    }

    pub fn stage(&mut self, stage: &Key) -> StageBuilder<Key, Base> {
        assert!(self.ioc.get_stage(stage).is_some());
        StageBuilder{ ioc: &mut self.ioc, stage: stage.clone() }
    }

    pub fn register_stage_at_begin(&mut self, stage: Key){
        self.ioc.register_stage(0, stage);
    }

    pub fn register_stage_at_end(&mut self, stage: Key){
        let len = self.ioc.stages().len();
        self.ioc.register_stage(len, stage);
    }

    pub fn register_stage_before(&mut self, pos_stage: &Key, stage: Key){
        let pos = self.ioc.stages().position(|(key, _)| key == pos_stage).unwrap();
        self.ioc.register_stage(pos, stage);
    }

    pub fn register_stage_after(&mut self, pos_stage: &Key, stage: Key){
        let pos = self.ioc.stages().position(|(key, _)| key == pos_stage).unwrap();
        self.ioc.register_stage(pos + 1, stage);
    }

    pub fn register_at_begin<St>(&mut self)
        where St: reflect::Object<Key = Key>
    {
        self.register_stage_at_begin(St::key().clone())
    }

    pub fn register_at_end<St>(&mut self)
        where St: reflect::Object<Key = Key>
    {
        self.register_stage_at_end(St::key().clone())
    }

    pub fn register_before<PosSt, St>(&mut self)
        where PosSt: reflect::Object<Key = Key>, St: reflect::Object<Key = Key>
    {
        self.register_stage_before(PosSt::key(), St::key().clone())
    }

    pub fn register_after<PosSt, St>(&mut self)
        where PosSt: reflect::Object<Key = Key>, St: reflect::Object<Key = Key>
    {
        self.register_stage_after(PosSt::key(), St::key().clone())
    }

    pub fn build(self) -> Pipeline<Key, Base> {
        self.ioc
    }
}

impl<Key, Base: ?Sized> Default for PipelineBuilder<Key, Base>
    where Key: reflect::Key, Base: Any
{
    fn default() -> Self { Self::new() }
}


