// TODO? lots of type-noise here, very ugly... maybe we should just use `InvocationMethod` 
// for dispatching, and move the actual functionality to the `Ioc`.

use service::{DefaultBase, ServiceKey, ServiceReflect};
use factory::{FactoryObject, Factory};

use downcast::Downcast;

use std::any::{Any, TypeId};
use std::collections::BTreeMap;
use std::fmt::Debug;
use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::ops::{Deref, DerefMut};
use std::marker::PhantomData;

// ++++++++++++++++++++ InvocationMethod ++++++++++++++++++++

/// TODO naming? `Invocation`?
pub trait InvocationMethod<'a, Key = String, Base: ?Sized = DefaultBase> 
    where Key: ServiceKey, Base: Any
{
    type Args;
    type Ret;
    type Error: Debug;

    fn invoke(
        services: &'a BTreeMap<Key, RwLock<Box<Base>>>, 
        args: Self::Args
    ) -> Result<Self::Ret, Self::Error>;
}

// ++++++++++++++++++++ NOP ++++++++++++++++++++

impl<'a, Key, Base: ?Sized> InvocationMethod<'a, Key, Base> for () 
    where Key: ServiceKey, Base: Any
{
    type Args = ();
    type Ret = ();
    type Error = ();

    fn invoke(
        _: &'a BTreeMap<Key, RwLock<Box<Base>>>,
        _: Self::Args
    ) -> Result<Self::Ret, Self::Error> {
        Ok(())
    }
}

// ++++++++++++++++++++ errors ++++++++++++++++++++

#[derive(Debug)]
pub enum LockError<'a, Key: 'a> {
    NotFound{ key: &'a Key },
    Poisoned{ key: &'a Key },
    MismatchedType{ key: &'a Key, expected: TypeId, found: TypeId },
}

#[derive(Debug)]
pub enum CreationError<'a, Key: 'a, CE> {
    LockError(LockError<'a, Key>),
    CreationError{ key: &'a Key, error: CE },
}

impl<'a, Key: 'a, CE> From<LockError<'a, Key>> for CreationError<'a, Key, CE> {
    fn from(err: LockError<'a, Key>) -> Self {
        CreationError::LockError(err)
    }
}

// ++++++++++++++++++++ Read ++++++++++++++++++++

pub struct ReadGuard<'a, Svc, Base: ?Sized + Any = DefaultBase> {
    inner: RwLockReadGuard<'a, Box<Base>>,
    _phantom: PhantomData<Svc>,
}

impl<'a, Svc, Base: ?Sized> Deref for ReadGuard<'a, Svc, Base> 
    where Svc: ServiceReflect, Base: Downcast<Svc>
{
    type Target = Svc;
    fn deref(&self) -> &Self::Target { 
        unsafe { self.inner.unchecked_downcast_ref() } 
    }
}

pub struct Read<Svc>(PhantomData<Svc>);

impl<'a, Svc, Key, Base: ?Sized> InvocationMethod<'a, Key, Base> for Read<Svc> 
    where Svc: ServiceReflect<Key = Key>, Key: ServiceKey, Base: Downcast<Svc>
{
    type Args = ();
    type Ret = ReadGuard<'a, Svc, Base>;
    type Error = LockError<'a, Svc::Key>;

    fn invoke(
        services: &'a BTreeMap<Key, RwLock<Box<Base>>>,
        _: Self::Args
    ) -> Result<Self::Ret, Self::Error> {
        let key = Svc::key();
        let service = match services.get(key) {
            Some(r) => r, 
            None => return Err(LockError::NotFound{ key: key })
        };
        let guard = match service.read() {
            Ok(r) => r, 
            Err(_) => return Err(LockError::Poisoned{ key: key })
        };
        if !guard.is_type(){
            return Err(LockError::MismatchedType{ 
                key: key, 
                expected: TypeId::of::<Svc>(),
                found: (*guard).get_type_id(),
            })
        };
        Ok(ReadGuard{ inner: guard, _phantom: PhantomData })
    }
}

// ++++++++++++++++++++ Write ++++++++++++++++++++

pub struct WriteGuard<'a, Svc, Base: ?Sized + Any = DefaultBase> {
    inner: RwLockWriteGuard<'a, Box<Base>>,
    _phantom: PhantomData<Svc>,
}

impl<'a, Svc, Base: ?Sized> Deref for WriteGuard<'a, Svc, Base> 
    where Svc: ServiceReflect, Base: Downcast<Svc>
{
    type Target = Svc;
    fn deref(&self) -> &Self::Target { 
        unsafe { self.inner.unchecked_downcast_ref() } 
    }
}

impl<'a, Svc, Base: ?Sized> DerefMut for WriteGuard<'a, Svc, Base> 
    where Svc: ServiceReflect, Base: Downcast<Svc>
{
    fn deref_mut(&mut self) -> &mut Self::Target { 
        unsafe { self.inner.unchecked_downcast_mut() } 
    }
}

pub struct Write<Svc>(PhantomData<Svc>);

impl<'a, Svc, Key, Base: ?Sized> InvocationMethod<'a, Key, Base> for Write<Svc> 
    where Svc: ServiceReflect<Key = Key>, Key: ServiceKey, Base: Downcast<Svc>
{
    type Args = ();
    type Ret = WriteGuard<'a, Svc, Base>;
    type Error = LockError<'a, Svc::Key>;

    fn invoke(
        services: &'a BTreeMap<Key, RwLock<Box<Base>>>,
        _: Self::Args
    ) -> Result<Self::Ret, Self::Error> {
        let key = Svc::key();
        let service = match services.get(key) {
            Some(r) => r, 
            None => return Err(LockError::NotFound{ key: key })
        };
        let guard = match service.write() {
            Ok(r) => r, 
            Err(_) => return Err(LockError::Poisoned{ key: key })
        };
        if !guard.is_type(){
            return Err(LockError::MismatchedType{ 
                key: key, 
                expected: TypeId::of::<Svc>(),
                found: (*guard).get_type_id(),
            })
        };
        Ok(WriteGuard{ inner: guard, _phantom: PhantomData })
    }
}

// ++++++++++++++++++++ Create ++++++++++++++++++++

pub struct Create<Obj>(PhantomData<Obj>);

impl<'a, Obj, Key, Base: ?Sized> InvocationMethod<'a, Key, Base> for Create<Obj> 
where 
    Obj: FactoryObject, 
    Obj::Factory: ServiceReflect<Key = Key>, 
    Key: ServiceKey, 
    Base: Downcast<Obj::Factory>,
{
    type Args = <Obj::Factory as Factory<Obj>>::Args;
    type Ret = Obj;
    type Error = CreationError<'a,
        <Obj::Factory as ServiceReflect>::Key, 
        <Obj::Factory as Factory<Obj>>::Error
    >;

    fn invoke(
        services: &'a BTreeMap<Key, RwLock<Box<Base>>>, 
        args: Self::Args
    ) -> Result<Self::Ret, Self::Error> {
        let mut factory = try!{Write::<Obj::Factory>::invoke(services, ())};
        match factory.create(args){
            Ok(r) => Ok(r), 
            Err(err) => Err(CreationError::CreationError{ 
                key: <Obj::Factory as ServiceReflect>::key(),
                error: err,
            })
        }
    }
}

// ++++++++++++++++++++ ReadAll ++++++++++++++++++++

pub type ReadGuardMap<'a, Key, Base: ?Sized>
    where Key: ServiceKey
= BTreeMap<&'a Key, RwLockReadGuard<'a, Box<Base>>>;

pub struct ReadAll(());

impl<'a, Key, Base: ?Sized> InvocationMethod<'a, Key, Base> for ReadAll 
    where Key: ServiceKey + 'a, Base: Any
{
    type Args = ();
    type Ret = ReadGuardMap<'a, Key, Base>;
    type Error = LockError<'a, Key>;

    fn invoke(
        services: &'a BTreeMap<Key, RwLock<Box<Base>>>,
        _: Self::Args
    ) -> Result<Self::Ret, Self::Error> {
        let mut map = ReadGuardMap::new();
        for (key, service) in services.iter() {
            let guard = match service.read() {
                Ok(r) => r, Err(_) => return Err(LockError::Poisoned{ key: key })
            };
            map.insert(key, guard);
        }
        Ok(map)
    }
}

// ++++++++++++++++++++ WriteAll ++++++++++++++++++++

pub type WriteGuardMap<'a, Key, Base: ?Sized>
    where Key: ServiceKey
= BTreeMap<&'a Key, RwLockWriteGuard<'a, Box<Base>>>;

pub struct WriteAll(());

impl<'a, Key, Base: ?Sized> InvocationMethod<'a, Key, Base> for WriteAll 
    where Key: ServiceKey + 'a, Base: Any
{
    type Args = ();
    type Ret = WriteGuardMap<'a, Key, Base>;
    type Error = LockError<'a, Key>;

    fn invoke(
        services: &'a BTreeMap<Key, RwLock<Box<Base>>>,
        _: Self::Args
    ) -> Result<Self::Ret, Self::Error> {
        let mut map = WriteGuardMap::new();
        for (key, service) in services.iter() {
            let guard = match service.write() {
                Ok(r) => r, Err(_) => return Err(LockError::Poisoned{ key: key })
            };
            map.insert(key, guard);
        }
        Ok(map)
    }
}

// ++++++++++++++++++++ multi ++++++++++++++++++++

macro_rules! e {
    ($e:expr) => { $e };
}

macro_rules! multi_methods {
    ($($err:ident {$($params:ident: $fields:tt)+})+) => {$(
        
        #[derive(Debug)]
        pub enum $err<$($params),+>{
            $($params($params)),+
        }

        impl<'a, $($params),+, Key, Base: ?Sized> InvocationMethod<'a, Key, Base> for ($($params,)+) 
            where $($params: InvocationMethod<'a, Key, Base>),+, Key: ServiceKey, Base: Any
        {
            type Args = ($($params::Args,)+);
            type Ret = ($($params::Ret,)+);
            type Error = $err<$($params::Error),+>;

            fn invoke(
                services: &'a BTreeMap<Key, RwLock<Box<Base>>>, 
                args: Self::Args
            ) -> Result<Self::Ret, Self::Error> {
                Ok(($(
                    match $params::invoke(services, e![args.$fields]){
                        Ok(r) => r, Err(r) => return Err($err::$params(r))
                    }
                ,)+))
            }
        }

    )+}
}

multi_methods!{
    MultiError1 {A:0} 
    MultiError2 {A:0 B:1} 
    MultiError3 {A:0 B:1 C:2}
    MultiError4 {A:0 B:1 C:2 D:3}
    MultiError5 {A:0 B:1 C:2 D:3 E:4}
    MultiError6 {A:0 B:1 C:2 D:3 E:4 F:5}
    MultiError7 {A:0 B:1 C:2 D:3 E:4 F:5 G:6}
    MultiError8 {A:0 B:1 C:2 D:3 E:4 F:5 G:6 H:7}
    MultiError9 {A:0 B:1 C:2 D:3 E:4 F:5 G:6 H:7 J:8}
    MultiError10 {A:0 B:1 C:2 D:3 E:4 F:5 G:6 H:7 J:8 K:9}
    MultiError11 {A:0 B:1 C:2 D:3 E:4 F:5 G:6 H:7 J:8 K:9 L:10}
    MultiError12 {A:0 B:1 C:2 D:3 E:4 F:5 G:6 H:7 J:8 K:9 L:10 M:11}
    MultiError13 {A:0 B:1 C:2 D:3 E:4 F:5 G:6 H:7 J:8 K:9 L:10 M:11 N:12}
    MultiError14 {A:0 B:1 C:2 D:3 E:4 F:5 G:6 H:7 J:8 K:9 L:10 M:11 N:12 O:13}
    MultiError15 {A:0 B:1 C:2 D:3 E:4 F:5 G:6 H:7 J:8 K:9 L:10 M:11 N:12 O:13 P:14}
    MultiError16 {A:0 B:1 C:2 D:3 E:4 F:5 G:6 H:7 J:8 K:9 L:10 M:11 N:12 O:13 P:14 Q:15}
}

