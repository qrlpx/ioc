use errors::Error;
use guards::{ReadGuard, WriteGuard};
use factory::FactoryBase;
use container::Container;
use reflect;

use downcast::Downcast;

use std::collections::BTreeMap;

// ++++++++++++++++++++ Method ++++++++++++++++++++

pub trait Method<'a, Cont>
    where Cont: Container<'a>
{
    type Ret;
    fn invoke(ioc: &'a Cont) -> Result<Self::Ret, Error<'a, Cont::Key>>;
}

// ++++++++++++++++++++ Method ++++++++++++++++++++

pub struct Read<Svc>(Svc);

impl<'a, Cont, Svc> Method<'a, Cont> for Read<Svc>
where 
    Svc: reflect::Service<Key = Cont::Key>,
    Cont: Container<'a>,
    Cont::ServiceBase: Downcast<Svc>,
{
    type Ret = ReadGuard<Svc, Cont::ServiceBase, Cont::ReadGuardBase>;
    fn invoke(ioc: &'a Cont) -> Result<Self::Ret, Error<'a, Cont::Key>> {
        ioc.read::<Svc>()
    }
}

macro_rules! multi_read {
    ($({$($params:ident)+})+) => {$(
        impl<'a, Cont, $($params),+> Method<'a, Cont> for Read<($($params,)+)>
        where
            $($params: reflect::Service<Key = Cont::Key>),+,
            Cont: Container<'a>,
            $(Cont::ServiceBase: Downcast<$params>),+
        {
            type Ret = ($(<Read<$params> as Method<'a, Cont>>::Ret,)+);
            fn invoke(ioc: &'a Cont) -> Result<Self::Ret, Error<'a, Cont::Key>> {
                Ok((
                    $(try!{ioc.read::<$params>()},)+
                ))
            }
        }
    )+}
}


// ++++++++++++++++++++ Method ++++++++++++++++++++

pub struct Write<Svc>(fn(Svc));

impl<'a, Cont, Svc> Method<'a, Cont> for Write<Svc>
where 
    Svc: reflect::Service<Key = Cont::Key>,
    Cont: Container<'a>,
    Cont::ServiceBase: Downcast<Svc>,
{
    type Ret = WriteGuard<Svc, Cont::ServiceBase, Cont::WriteGuardBase>;
    fn invoke(ioc: &'a Cont) -> Result<Self::Ret, Error<'a, Cont::Key>> {
        ioc.write::<Svc>()
    }
}

macro_rules! multi_write {
    ($({$($params:ident)+})+) => {$(
        impl<'a, Cont, $($params),+> Method<'a, Cont> for Write<($($params,)+)>
        where
            $($params: reflect::Service<Key = Cont::Key>),+,
            Cont: Container<'a>,
            $(Cont::ServiceBase: Downcast<$params>),+
        {
            type Ret = ($(<Write<$params> as Method<'a, Cont>>::Ret,)+);
            fn invoke(ioc: &'a Cont) -> Result<Self::Ret, Error<'a, Cont::Key>> {
                Ok((
                    $(try!{ioc.write::<$params>()},)+
                ))
            }
        }
    )+}
}

// ++++++++++++++++++++ Method ++++++++++++++++++++

pub struct Create<Obj>(fn(Obj));

impl<'a, Cont, Obj> Method<'a, Cont> for Create<Obj>
where 
    Obj: reflect::FactoryObject<Key = Cont::Key>,
    Obj::Factory: FactoryBase<'a, Cont, Obj>,
    Cont: Container<'a>,
    Cont::ServiceBase: Downcast<Obj::Factory>,
{
    type Ret = Obj;
    fn invoke(ioc: &'a Cont) -> Result<Self::Ret, Error<'a, Cont::Key>> {
        ioc.create::<Obj>()
    }
}

macro_rules! multi_create {
    ($({$($params:ident)+})+) => {$(
        impl<'a, Cont, $($params),+> Method<'a, Cont> for Create<($($params,)+)>
        where
            $($params: reflect::FactoryObject<Key = Cont::Key>),+,
            $($params::Factory: FactoryBase<'a, Cont, $params>),+,
            Cont: Container<'a>,
            $(Cont::ServiceBase: Downcast<$params::Factory>),+
        {
            type Ret = ($(<Create<$params> as Method<'a, Cont>>::Ret,)+);
            fn invoke(ioc: &'a Cont) -> Result<Self::Ret, Error<'a, Cont::Key>> {
                Ok((
                    $(try!{ioc.create::<$params>()},)+
                ))
            }
        }
    )+}
}

// ++++++++++++++++++++ Method ++++++++++++++++++++

pub struct ReadAll(());

impl<'a, Cont> Method<'a, Cont> for ReadAll
    where Cont: Container<'a>,
{
    type Ret = BTreeMap<&'a Cont::Key, Cont::ReadGuardBase>;
    fn invoke(ioc: &'a Cont) -> Result<Self::Ret, Error<'a, Cont::Key>> {
        ioc.read_all()
    }
}

// ++++++++++++++++++++ Method ++++++++++++++++++++

pub struct WriteAll(());

impl<'a, Cont> Method<'a, Cont> for WriteAll
    where Cont: Container<'a>,
{
    type Ret = BTreeMap<&'a Cont::Key, Cont::WriteGuardBase>;
    fn invoke(ioc: &'a Cont) -> Result<Self::Ret, Error<'a, Cont::Key>> {
        ioc.write_all()
    }
}

// ++++++++++++++++++++ multi-method ++++++++++++++++++++

macro_rules! multi_methods {
    ($({$($params:ident)+})+) => {$(
        
        impl<'a, Cont, $($params),+> Method<'a, Cont> for ($($params,)+) 
            where Cont: Container<'a>, $($params: Method<'a, Cont> + 'a),+, 
        {
            type Ret = ($($params::Ret,)+);

            fn invoke(ioc: &'a Cont) -> Result<Self::Ret, Error<'a, Cont::Key>> {
                Ok((
                    $(try!{$params::invoke(ioc)},)+
                ))
            }
        }

    )+}
}

multi_read!{
    {A} 
    {A B} 
    {A B C}
    {A B C D}
    {A B C D E}
    {A B C D E F}
    {A B C D E F G}
    {A B C D E F G H}
    {A B C D E F G H J}
    {A B C D E F G H J K}
    {A B C D E F G H J K L}
    {A B C D E F G H J K L M}
    {A B C D E F G H J K L M N}
    {A B C D E F G H J K L M N O}
    {A B C D E F G H J K L M N O P}
    {A B C D E F G H J K L M N O P Q}
}

multi_write!{
    {A} 
    {A B} 
    {A B C}
    {A B C D}
    {A B C D E}
    {A B C D E F}
    {A B C D E F G}
    {A B C D E F G H}
    {A B C D E F G H J}
    {A B C D E F G H J K}
    {A B C D E F G H J K L}
    {A B C D E F G H J K L M}
    {A B C D E F G H J K L M N}
    {A B C D E F G H J K L M N O}
    {A B C D E F G H J K L M N O P}
    {A B C D E F G H J K L M N O P Q}
}

multi_create!{
    {A} 
    {A B} 
    {A B C}
    {A B C D}
    {A B C D E}
    {A B C D E F}
    {A B C D E F G}
    {A B C D E F G H}
    {A B C D E F G H J}
    {A B C D E F G H J K}
    {A B C D E F G H J K L}
    {A B C D E F G H J K L M}
    {A B C D E F G H J K L M N}
    {A B C D E F G H J K L M N O}
    {A B C D E F G H J K L M N O P}
    {A B C D E F G H J K L M N O P Q}
}

multi_methods!{
    {A} 
    {A B} 
    {A B C}
    {A B C D}
    {A B C D E}
    {A B C D E F}
    {A B C D E F G}
    {A B C D E F G H}
    {A B C D E F G H J}
    {A B C D E F G H J K}
    {A B C D E F G H J K L}
    {A B C D E F G H J K L M}
    {A B C D E F G H J K L M N}
    {A B C D E F G H J K L M N O}
    {A B C D E F G H J K L M N O P}
    {A B C D E F G H J K L M N O P Q}
}




