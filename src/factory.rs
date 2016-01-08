use errors::Error;
use methods::Method;
use container::Container;

use std::any::Any;
use std::error::Error as StdError;
use std::ops::Deref;

/// TODO get rid of Key-param?
pub trait FactoryBase<'a, Cont, Obj>: Any 
    where Cont: Container<'a>
{
    fn create(
        &self, 
        self_key: &'a Cont::Key, 
        ioc: &'a Cont
    ) -> Result<Obj, Error<'a, Cont::Key>>;
}

pub trait Factory<'a, Cont, Obj>: Any
    where Cont: Container<'a>
{
    type ArgSelection: Method<'a, Cont>;
    type Args = <Self::ArgSelection as Method<'a, Cont>>::Ret;

    type Error: StdError;

    fn create(&self, args: <Self::ArgSelection as Method<'a, Cont>>::Ret) -> Result<Obj, Self::Error>;
}

/// Auto-implement `FactoryBase` for a type which implements `Factory`.
/// 
/// FIXME: Remove (and replace) this as soon as partial-impls are available.
#[macro_export]
macro_rules! qioc_autoimpl_factory_base {
    ($ty:ty, $cont:ty, $obj:ty) => {
        impl<'a> ::qioc::FactoryBase<'a, $cont, $obj> for $ty {
            fn create(
                &self, 
                self_key: &'a <$cont as ::qioc::Container<'a>>::Key, 
                ioc: &'a $cont
            ) -> Result<$obj, ::qioc::Error<'a, <$cont as ::ioc::Container<'a>>::Key>> {
                // TODO use try!
                let args = <<Self as ::qioc::Factory<'a, $cont, $obj>>::ArgSelection as ::qioc::Method<_>>::invoke(ioc).unwrap();

                match ::qioc::Factory::create(self, args) {
                    Ok(r) => Ok(r),
                    Err(err) => Err(::qioc::Error::CreationError{ 
                        key: self_key,
                        error: Box::new(err)
                    })
                }
            }
        }
   
    };
}

// ++++++++++++++++++++ newtype-impls ++++++++++++++++++++

impl<'a, Cont, Obj, T> FactoryBase<'a, Cont, Obj> for T
    where Cont: Container<'a>, T: Any + Deref, T::Target: FactoryBase<'a, Cont, Obj>
{
    fn create(
        &self, 
        self_key: &'a Cont::Key,
        ioc: &'a Cont
    ) -> Result<Obj, Error<'a, Cont::Key>> {
        (**self).create(self_key, ioc)
    }
}

impl<'a, Cont, Obj, T> Factory<'a, Cont, Obj> for T
    where Cont: Container<'a>, T: Any + Deref, T::Target: Factory<'a, Cont, Obj>
{
    type ArgSelection = <T::Target as Factory<'a, Cont, Obj>>::ArgSelection;

    type Error = <T::Target as Factory<'a, Cont, Obj>>::Error;

    fn create(&self, args: <Self::ArgSelection as Method<'a, Cont>>::Ret) -> Result<Obj, Self::Error> {
        (**self).create(args)
    }
}

