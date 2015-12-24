use downcast::Downcast;

use std::any::Any;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

// ++++++++++++++++++++ ReadGuard ++++++++++++++++++++

/// * `Svc`: The service-type `SvcBase` will be downcasted to.
/// * `SvcBase`: The service-base-type.
/// * `Inner`: The guard-type which will be wrapped (derefs to `Box<SvcBase>`).
pub struct ReadGuard<Svc, SvcBase: ?Sized, Inner> {
    inner: Inner,
    _phantom: PhantomData<fn(Svc, SvcBase)>,
}

impl<Svc, SvcBase: ?Sized, Inner> ReadGuard<Svc, SvcBase, Inner> 
    where Svc: Any, SvcBase: Downcast<Svc>, Inner: Deref<Target = Box<SvcBase>>
{
    #[doc(hidden)]
    pub fn new(inner: Inner) -> Self {
        assert!(inner.is_type());
        ReadGuard{ inner: inner, _phantom: PhantomData }
    }
}

impl<Svc, SvcBase: ?Sized, Inner> Deref for ReadGuard<Svc, SvcBase, Inner> 
    where Svc: Any, SvcBase: Downcast<Svc>, Inner: Deref<Target = Box<SvcBase>>
{
    type Target = Svc;
    fn deref(&self) -> &Self::Target { 
        unsafe { self.inner.unchecked_downcast_ref() } 
    }
}

// ++++++++++++++++++++ WriteGuard ++++++++++++++++++++

/// * `Svc`: The service-type `SvcBase` will be downcasted to.
/// * `SvcBase`: The service-base-type.
/// * `Inner`: The guard-type which will be wrapped (derefs to `Box<SvcBase>`).
pub struct WriteGuard<Svc, SvcBase: ?Sized, Inner> {
    inner: Inner,
    _phantom: PhantomData<fn(Svc, SvcBase)>,
}

impl<Svc, SvcBase: ?Sized, Inner> WriteGuard<Svc, SvcBase, Inner> 
    where Svc: Any, SvcBase: Downcast<Svc>, Inner: Deref<Target = Box<SvcBase>>
{
    #[doc(hidden)]
    pub fn new(inner: Inner) -> Self {
        assert!(inner.is_type());
        WriteGuard{ inner: inner, _phantom: PhantomData }
    }
}

impl<Svc, SvcBase: ?Sized, Inner> Deref for WriteGuard<Svc, SvcBase, Inner> 
    where Svc: Any, SvcBase: Downcast<Svc>, Inner: Deref<Target = Box<SvcBase>>
{
    type Target = Svc;
    fn deref(&self) -> &Self::Target { 
        unsafe { self.inner.unchecked_downcast_ref() } 
    }
}

impl<Svc, SvcBase: ?Sized, Inner> DerefMut for WriteGuard<Svc, SvcBase, Inner> 
    where Svc: Any, SvcBase: Downcast<Svc>, Inner: DerefMut<Target = Box<SvcBase>>
{
    fn deref_mut(&mut self) -> &mut Self::Target { 
        unsafe { self.inner.unchecked_downcast_mut() } 
    }
}

