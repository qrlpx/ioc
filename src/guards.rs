use reflect;

use downcast::Downcast;

use std::any::Any;
use std::collections::BTreeMap;
use std::sync::{RwLockReadGuard, RwLockWriteGuard};
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

// ++++++++++++++++++++ guards ++++++++++++++++++++

pub struct ReadGuard<'a, Svc, Base: ?Sized + Any> {
    inner: RwLockReadGuard<'a, Box<Base>>,
    _phantom: PhantomData<Svc>,
}

impl<'a, Svc, Base: ?Sized> ReadGuard<'a, Svc, Base> 
    where Svc: reflect::Object, Base: Downcast<Svc>
{
    #[doc(hidden)]
    pub fn new(inner: RwLockReadGuard<'a, Box<Base>>) -> Self {
        assert!(inner.is_type());
        ReadGuard{ inner: inner, _phantom: PhantomData }
    }
}

impl<'a, Svc, Base: ?Sized> Deref for ReadGuard<'a, Svc, Base> 
    where Svc: reflect::Object, Base: Downcast<Svc>
{
    type Target = Svc;
    fn deref(&self) -> &Self::Target { 
        unsafe { self.inner.unchecked_downcast_ref() } 
    }
}

pub struct WriteGuard<'a, Svc, Base: ?Sized + Any> {
    inner: RwLockWriteGuard<'a, Box<Base>>,
    _phantom: PhantomData<Svc>,
}

impl<'a, Svc, Base: ?Sized> WriteGuard<'a, Svc, Base> 
    where Svc: reflect::Object, Base: Downcast<Svc>
{
    #[doc(hidden)]
    pub fn new(inner: RwLockWriteGuard<'a, Box<Base>>) -> Self {
        assert!(inner.is_type());
        WriteGuard{ inner: inner, _phantom: PhantomData }
    }
}

impl<'a, Svc, Base: ?Sized> Deref for WriteGuard<'a, Svc, Base> 
    where Svc: reflect::Object, Base: Downcast<Svc>
{
    type Target = Svc;
    fn deref(&self) -> &Self::Target { 
        unsafe { self.inner.unchecked_downcast_ref() } 
    }
}

impl<'a, Svc, Base: ?Sized> DerefMut for WriteGuard<'a, Svc, Base> 
    where Svc: reflect::Object, Base: Downcast<Svc>
{
    fn deref_mut(&mut self) -> &mut Self::Target { 
        unsafe { self.inner.unchecked_downcast_mut() } 
    }
}

pub type ReadGuardMap<'a, Key, Base: ?Sized>
    where Key: 'a
= BTreeMap<&'a Key, RwLockReadGuard<'a, Box<Base>>>;

pub type WriteGuardMap<'a, Key: 'a, Base: ?Sized>
    where Key: 'a
= BTreeMap<&'a Key, RwLockWriteGuard<'a, Box<Base>>>;


