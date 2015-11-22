use downcast::Downcast;

use std::any::Any;
use std::fmt::Debug;

// ++++++++++++++++++++ DefaultBase ++++++++++++++++++++

pub trait DefaultBase: Any + Sync {}

impl_downcast!(DefaultBase);
downcast_methods!(DefaultBase);

impl<T: Any + Sync> DefaultBase for T {}

// ++++++++++++++++++++ ServiceReflect ++++++++++++++++++++

pub trait ServiceKey: Debug + Ord + Clone + 'static {}

impl<T: Debug + Ord + Clone + 'static> ServiceKey for T {}

pub trait ServiceReflect: Any + Sized {
    type Key: ServiceKey = String;
    fn key() -> &'static Self::Key;
}


