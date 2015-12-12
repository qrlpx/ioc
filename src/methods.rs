use std::error::Error;

// ++++++++++++++++++++ Method ++++++++++++++++++++

pub trait Method<'a, Cont> {
    type Ret;
    type Error: Error;
    fn invoke(ioc: &'a Cont) -> Result<Self::Ret, Self::Error>;
}

// ++++++++++++++++++++ method-types ++++++++++++++++++++

pub struct Read<Svc>(Svc);

pub struct Write<Svc>(Svc);

pub struct ReadAll(());

pub struct WriteAll(());

pub struct Create<Obj>(Obj);

pub struct ReadStage<St>(St);

pub struct WriteStage<St>(St);

// ++++++++++++++++++++ multi-methods ++++++++++++++++++++

use errors::MultiError;

macro_rules! multi_methods {
    ($({$($params:ident: $fields:tt)+})+) => {$(
        
        impl<'a, Cont, $($params),+> Method<'a, Cont> for ($($params,)+) 
            where $($params: Method<'a, Cont> + 'a),+, 
        {
            type Ret = ($($params::Ret,)+);
            type Error = MultiError<'a>;

            fn invoke(ioc: &'a Cont) -> Result<Self::Ret, Self::Error> {
                let mut idx = 0;
                Ok(($(
                    match $params::invoke(ioc) {
                        Ok(r) => { idx += 1; r }
                        Err(err) => return Err(MultiError{ 
                            idx: idx, 
                            error: Box::new(err)
                        })
                    }
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




