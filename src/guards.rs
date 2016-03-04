use downcast::Downcast;
//use shared_mutex::{SharedMutexReadGuard, SharedMutexWriteGuard};

use std::any::Any;
use std::sync::{RwLockReadGuard, RwLockWriteGuard};
use std::ops::{Deref, DerefMut};
use std::marker::PhantomData;

// ++++++++++++++++++++ ReadGuard ++++++++++++++++++++

pub struct ReadGuard<'a, Svc, SvcBase: ?Sized + 'a> {
    inner: RwLockReadGuard<'a, Box<SvcBase>>,
    _phantom: PhantomData<fn(Svc)>,
}

impl<'a, Svc, SvcBase: ?Sized> ReadGuard<'a, Svc, SvcBase> 
    where Svc: Any, SvcBase: Downcast<Svc>
{
    #[doc(hidden)]
    pub fn new(inner: RwLockReadGuard<'a, Box<SvcBase>>) -> Self {
        assert!(inner.is_type());
        ReadGuard{ inner: inner, _phantom: PhantomData }
    }
}

impl<'a, Svc, SvcBase: ?Sized> Deref for ReadGuard<'a, Svc, SvcBase> 
    where Svc: Any, SvcBase: Downcast<Svc>
{
    type Target = Svc;
    fn deref(&self) -> &Self::Target { 
        unsafe { self.inner.unchecked_downcast_ref() } 
    }
}

// ++++++++++++++++++++ WriteGuard ++++++++++++++++++++

pub struct WriteGuard<'a, Svc, SvcBase: ?Sized + 'a> {
    inner: RwLockWriteGuard<'a, Box<SvcBase>>,
    _phantom: PhantomData<fn(Svc)>,
}

impl<'a, Svc, SvcBase: ?Sized> WriteGuard<'a, Svc, SvcBase> 
    where Svc: Any, SvcBase: Downcast<Svc>
{
    #[doc(hidden)]
    pub fn new(inner: RwLockWriteGuard<'a, Box<SvcBase>>) -> Self {
        assert!(inner.is_type());
        WriteGuard{ inner: inner, _phantom: PhantomData }
    }
}

impl<'a, Svc, SvcBase: ?Sized> Deref for WriteGuard<'a, Svc, SvcBase> 
    where Svc: Any, SvcBase: Downcast<Svc>
{
    type Target = Svc;
    fn deref(&self) -> &Self::Target { 
        unsafe { self.inner.unchecked_downcast_ref() } 
    }
}

impl<'a, Svc, SvcBase: ?Sized> DerefMut for WriteGuard<'a, Svc, SvcBase> 
    where Svc: Any, SvcBase: Downcast<Svc>
{
    fn deref_mut(&mut self) -> &mut Self::Target { 
        unsafe { self.inner.unchecked_downcast_mut() } 
    }
}
