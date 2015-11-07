use downcast::Downcast;

use std::any::Any;
use std::fmt::Debug;

// ++++++++++++++++++++ DefaultBase ++++++++++++++++++++
// TODO should this be located here?

pub trait DefaultBase: Any {}

impl_downcast!(DefaultBase);
downcast_methods!(DefaultBase);

impl<T: Any> DefaultBase for T {}

// ++++++++++++++++++++ ServiceReflect ++++++++++++++++++++

pub trait ServiceReflect: Any + Sized {
    type Key: ?Sized + Debug + Ord = str;
    fn key() -> &'static Self::Key;
}


