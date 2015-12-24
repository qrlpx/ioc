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

pub struct ReadAll(());

impl<'a, Cont> Method<'a, Cont> for ReadAll
    where Cont: Container<'a>,
{
    type Ret = BTreeMap<&'a Cont::Key, Cont::ReadGuardBase>;
    fn invoke(ioc: &'a Cont) -> Result<Self::Ret, Error<'a, Cont::Key>> {
        ioc.read_all()
    }
}

pub struct WriteAll(());

impl<'a, Cont> Method<'a, Cont> for WriteAll
    where Cont: Container<'a>,
{
    type Ret = BTreeMap<&'a Cont::Key, Cont::WriteGuardBase>;
    fn invoke(ioc: &'a Cont) -> Result<Self::Ret, Error<'a, Cont::Key>> {
        ioc.write_all()
    }
}

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

macro_rules! multi_methods {
    ($({$($params:ident: $fields:tt)+})+) => {$(
        
        #[allow(unused_assignments)] // FIXME `idx` get's falsely reported.
        impl<'a, Cont, $($params),+> Method<'a, Cont> for ($($params,)+) 
            where Cont: Container<'a>, $($params: Method<'a, Cont> + 'a),+, 
        {
            type Ret = ($($params::Ret,)+);

            fn invoke(ioc: &'a Cont) -> Result<Self::Ret, Error<'a, Cont::Key>> {
                Ok(($(
                    try!{$params::invoke(ioc)}
                ,)+))
            }
        }

    )+}
}

multi_methods!{
    {A:0} 
    {A:0 B:1} 
    {A:0 B:1 C:2}
    {A:0 B:1 C:2 D:3}
    {A:0 B:1 C:2 D:3 E:4}
    {A:0 B:1 C:2 D:3 E:4 F:5}
    {A:0 B:1 C:2 D:3 E:4 F:5 G:6}
    {A:0 B:1 C:2 D:3 E:4 F:5 G:6 H:7}
    {A:0 B:1 C:2 D:3 E:4 F:5 G:6 H:7 J:8}
    {A:0 B:1 C:2 D:3 E:4 F:5 G:6 H:7 J:8 K:9}
    {A:0 B:1 C:2 D:3 E:4 F:5 G:6 H:7 J:8 K:9 L:10}
    {A:0 B:1 C:2 D:3 E:4 F:5 G:6 H:7 J:8 K:9 L:10 M:11}
    {A:0 B:1 C:2 D:3 E:4 F:5 G:6 H:7 J:8 K:9 L:10 M:11 N:12}
    {A:0 B:1 C:2 D:3 E:4 F:5 G:6 H:7 J:8 K:9 L:10 M:11 N:12 O:13}
    {A:0 B:1 C:2 D:3 E:4 F:5 G:6 H:7 J:8 K:9 L:10 M:11 N:12 O:13 P:14}
    {A:0 B:1 C:2 D:3 E:4 F:5 G:6 H:7 J:8 K:9 L:10 M:11 N:12 O:13 P:14 Q:15}
}




