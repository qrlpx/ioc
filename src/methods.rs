use errors::Error;
//use factory::FactoryBase;
use container::{ReadGuard, WriteGuard, Container};
use reflect;

use downcast::Downcast;

use std::marker::PhantomData;
use std::any::Any;

// ++++++++++++++++++++ Method ++++++++++++++++++++

pub trait Method<'a, Key, SvcBase: ?Sized>: Any
    where Key: reflect::Key, SvcBase: Any
{
    type Ret: 'a;
    fn resolve_unprotected(ioc: &'a Container<Key, SvcBase>) -> Result<Self::Ret, Error<'a, Key>>;
    fn try_resolve_unprotected(ioc: &'a Container<Key, SvcBase>) -> Result<Self::Ret, Error<'a, Key>>;
}

// ++++++++++++++++++++ dummy ++++++++++++++++++++

impl<'a, Key, SvcBase: ?Sized> Method<'a, Key, SvcBase> for ()
    where Key: reflect::Key, SvcBase: Any
{
    type Ret = ();

    fn resolve_unprotected(_: &'a Container<Key, SvcBase>) -> Result<Self::Ret, Error<'a, Key>> {
        Ok(())
    }
    fn try_resolve_unprotected(_: &'a Container<Key, SvcBase>) -> Result<Self::Ret, Error<'a, Key>> {
        Ok(())
    }
}

// ++++++++++++++++++++ Read ++++++++++++++++++++

pub struct Read<Svc>(PhantomData<fn(Svc)>);

impl<'a, Key, SvcBase: ?Sized, Svc> Method<'a, Key, SvcBase> for Read<Svc>
where 
    Key: reflect::Key,
    Svc: reflect::Service<Key = Key>,
    SvcBase: Downcast<Svc>,
{
    type Ret = ReadGuard<'a, Svc, SvcBase>;
    fn resolve_unprotected(ioc: &'a Container<Key, SvcBase>) -> Result<Self::Ret, Error<'a, Key>> {
        ioc.read::<Svc>()
    }
    fn try_resolve_unprotected(ioc: &'a Container<Key, SvcBase>) -> Result<Self::Ret, Error<'a, Key>> {
        ioc.try_read::<Svc>()
    }
}

macro_rules! multi_read {
    ($({$($params:ident)+})+) => {$(
        impl<'a, Key, SvcBase: ?Sized, $($params),+> Method<'a, Key, SvcBase> for Read<($($params,)+)>
        where
            Key: reflect::Key,
            $($params: reflect::Service<Key = Key>),+,
            $(SvcBase: Downcast<$params>),+
        {
            type Ret = ($(<Read<$params> as Method<'a, Key, SvcBase>>::Ret,)+);
            fn resolve_unprotected(ioc: &'a Container<Key, SvcBase>) -> Result<Self::Ret, Error<'a, Key>> {
                Ok((
                    $(try!{ioc.read::<$params>()},)+
                ))
            }
            fn try_resolve_unprotected(ioc: &'a Container<Key, SvcBase>) -> Result<Self::Ret, Error<'a, Key>> {
                Ok((
                    $(try!{ioc.try_read::<$params>()},)+
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
    //{A B C D E F G H J K L M N O P}
    //{A B C D E F G H J K L M N O P Q}
}

// ++++++++++++++++++++ Write ++++++++++++++++++++

pub struct Write<Svc>(PhantomData<fn(Svc)>);

impl<'a, Key, SvcBase: ?Sized, Svc> Method<'a, Key, SvcBase> for Write<Svc>
where 
    Key: reflect::Key,
    Svc: reflect::Service<Key = Key>,
    SvcBase: Downcast<Svc>,
{
    type Ret = WriteGuard<'a, Svc, SvcBase>;
    fn resolve_unprotected(ioc: &'a Container<Key, SvcBase>) -> Result<Self::Ret, Error<'a, Key>> {
        ioc.write::<Svc>()
    }
    fn try_resolve_unprotected(ioc: &'a Container<Key, SvcBase>) -> Result<Self::Ret, Error<'a, Key>> {
        ioc.try_write::<Svc>()
    }
}

macro_rules! multi_write {
    ($({$($params:ident)+})+) => {$(
        impl<'a, Key, SvcBase: ?Sized, $($params),+> Method<'a, Key, SvcBase> for Write<($($params,)+)>
        where
            Key: reflect::Key,
            $($params: reflect::Service<Key = Key>),+,
            $(SvcBase: Downcast<$params>),+
        {
            type Ret = ($(<Write<$params> as Method<'a, Key, SvcBase>>::Ret,)+);
            fn resolve_unprotected(ioc: &'a Container<Key, SvcBase>) -> Result<Self::Ret, Error<'a, Key>> {
                Ok((
                    $(try!{ioc.write::<$params>()},)+
                ))
            }
            fn try_resolve_unprotected(ioc: &'a Container<Key, SvcBase>) -> Result<Self::Ret, Error<'a, Key>> {
                Ok((
                    $(try!{ioc.try_write::<$params>()},)+
                ))
            }
        }
    )+}
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

/*
// ++++++++++++++++++++ Create ++++++++++++++++++++

pub struct Create<Obj>(fn(Obj));

impl<'a, Cont, Obj> Method<'a, Cont> for Create<Obj>
where 
    Obj: reflect::FactoryObject<Key = Cont::Key>,
    Obj::Factory: FactoryBase<'a, Cont, Obj>,
    Cont: Container<'a>,
    Cont::ServiceBase: Downcast<Obj::Factory>,
{
    type Ret = Obj;
    fn resolve_unprotected(ioc: &'a Cont) -> Result<Self::Ret, Error<'a, Cont::Key>> {
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
            fn resolve_unprotected(ioc: &'a Cont) -> Result<Self::Ret, Error<'a, Cont::Key>> {
                Ok((
                    $(try!{ioc.create::<$params>()},)+
                ))
            }
        }
    )+}
}
*/
// ++++++++++++++++++++ multi-method ++++++++++++++++++++

macro_rules! e {
    ($e:expr) => { $e }
}

macro_rules! multi_methods {
    ($({$($idx:tt,$params:ident)+})+) => {$(
        
        impl<'a, Key, SvcBase: ?Sized, $($params),+> Method<'a, Key, SvcBase> for ($($params,)+) 
        where 
            Key: reflect::Key,
            $($params: Method<'a, Key, SvcBase> + 'a),+, 
            SvcBase: Any,
        {
            type Ret = ($($params::Ret,)+);
            fn resolve_unprotected(ioc: &'a Container<Key, SvcBase>) -> Result<Self::Ret, Error<'a, Key>> {
                Ok((
                    $(try!{e![$params::resolve_unprotected(ioc)]},)+
                ))
            }
            fn try_resolve_unprotected(ioc: &'a Container<Key, SvcBase>) -> Result<Self::Ret, Error<'a, Key>> {
                Ok((
                    $(try!{e![$params::try_resolve_unprotected(ioc)]},)+
                ))
            }
        }

    )+}
}

multi_methods!{
    {0,A} 
    {0,A 1,B} 
    {0,A 1,B 2,C}
    {0,A 1,B 2,C 3,D}
    {0,A 1,B 2,C 3,D 4,E}
    {0,A 1,B 2,C 3,D 4,E 5,F}
    {0,A 1,B 2,C 3,D 4,E 5,F 6,G}
    {0,A 1,B 2,C 3,D 4,E 5,F 6,G 7,H}
    {0,A 1,B 2,C 3,D 4,E 5,F 6,G 7,H 8,J}
    {0,A 1,B 2,C 3,D 4,E 5,F 6,G 7,H 8,J 9,K}
    {0,A 1,B 2,C 3,D 4,E 5,F 6,G 7,H 8,J 9,K 10,L}
    {0,A 1,B 2,C 3,D 4,E 5,F 6,G 7,H 8,J 9,K 10,L 11,M}
    {0,A 1,B 2,C 3,D 4,E 5,F 6,G 7,H 8,J 9,K 10,L 11,M 12,N}
    {0,A 1,B 2,C 3,D 4,E 5,F 6,G 7,H 8,J 9,K 10,L 11,M 12,N 13,O}
    {0,A 1,B 2,C 3,D 4,E 5,F 6,G 7,H 8,J 9,K 10,L 11,M 12,N 13,O 14,P}
    {0,A 1,B 2,C 3,D 4,E 5,F 6,G 7,H 8,J 9,K 10,L 11,M 12,N 13,O 14,P 15,Q}
}




