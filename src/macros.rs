
/* vvv SHOULD LOOK LIKE THIS vvv

qregister_wire_fns!{

    fn load_stores |cfg: &Cfg| => {
        
        single PositionStore;
        
        single physics::LinearVelocityStore{ name: "LinearVelocityStore" }
        
        single ScoreStore{ init: ScoreStore::from_file(cfg.get_string("score_file.txt")) }
        
    }

    fn load_services => {

        single Timing;

        option Physics;

        alternative(Physics) nphysics::Physics{ name: "NPhysicsPhysics" }

        alternative(Physics) LegacyPhysics{ init: LegacyPhysics::new("0.1.0") }
    
    }
}

pub fn load_module(register: &mut Register<MyBase>, cfg: &Cfg){
    
    load_stores(register, cfg);
    load_services(register);
    
    register.wire_alternative("Physics", "LegacyPhysics");
}

*/

#[macro_export]
macro_rules! qregister_wire_fns {
    () => {};
    ($col:ident <- single $ty:ty { name: $name:expr, init: $init:expr } $($ff:tt)*) => {
        {
            impl super::OptionReflect for $ty {
                fn option_name() -> &'static str { $name }
            }
        
            $col.singles.push(({$name}.to_string(), box {$init}));

            qregister_wire_fns!($($ff)*);
        }
    };
    ($col:ident <- single $ty:ty { init: $init:expr, name: $name:expr } $($ff:tt)*) => {
        qregister_wire_fns!{
            $col <- single $ty { name: $name, init: $init } 
            $($ff)*
        }
    };
    ($col:ident <- single $ty:ty { init: $init:expr } $($ff:tt)*) => {
        qregister_wire_fns!{
            $col <- single $ty { name: stringify!($ty), init: $init }
            $($ff)*
        }
    };
    ($col:ident <- single $ty:ty { name: $name:expr } $($ff:tt)*) => {
        qregister_wire_fns!{
            $col <- single $ty { name: $name, init: <$ty as Default>::default() }
            $($ff)*
        }
    };
    ($col:ident <- single $ty:ty; $($ff:tt)*) => {
        qregister_wire_fns!{
            $col <- single $ty { name: stringify!($ty) }
            $($ff)*
        }
    };
    ($col:ident <- option $ty:ty { name: $name:expr } $($ff:tt)*) => {
        {
            impl super::OptionReflect for $ty {
                fn option_name() -> &'static str { $name }
            }

            $col.options.push({$name}.to_string());

            qregister_wire_fns!($($ff)*);
        }
    };
    ($col:ident <- option $ty:ty; $($ff:tt)*) => {
        qregister_wire_fns!($col <- option $ty { name: stringify!($ty) });
    };
    ($col:ident <- alternative($opt:ty) $ty:ty { name: $name:expr, init: $init:expr } $($ff:tt)*) => {
        {
            $col.alternatives.push((<$opt as super::OptionReflect>::option_name().to_string(), 
                                    {$name}.to_string(), 
                                    box {$init}));

            qregister_wire_fns!($($ff)*);
        }
    };
    ($col:ident <- alternative($opt:ty) $ty:ty { init: $init:expr, name: $name:expr } $($ff:tt)*) => {
        qregister_wire_fns!{
            $col <- alternative($opt) $ty { name: $name, init: $init } 
            $($ff)*
        }
    };
    ($col:ident <- alternative($opt:ty) $ty:ty { init: $init:expr } $($ff:tt)*) => {
        qregister_wire_fns!{
            $col <- alternative($opt) $ty { name: stringify!($ty), init: $init }
            $($ff)*
        }
    };
    ($col:ident <- alternative($opt:ty) $ty:ty { name: $name:expr } $($ff:tt)*) => {
        qregister_wire_fns!{
            $col <- alternative($opt) $ty { name: $name, init: box <$ty as Default>::default() as Box<$opt> }
            $($ff)*
        }
    };
    ($col:ident <- alternative($opt:ty) $ty:ty; $($ff:tt)*) => {
        qregister_wire_fns!{
            $col <- alternative($opt) $ty { name: stringify!($ty) }
            $($ff)*
        }
    };
}

use std::any::Any;
pub trait Base: Any + 'static {}
impl<B: ?Sized + Base> Base for Box<B> {}

pub trait SecondBase: Base {}

#[derive(Default)]
pub struct Foo;
impl Base for Foo {}

#[derive(Default)]
pub struct Bar;
impl Base for Bar {}
impl SecondBase for Bar {}

#[test]
pub fn test(){
    struct Collect {
        singles: Vec<(String, Box<Base>)>,
        options: Vec<String>,
        alternatives: Vec<(String, String, Box<Base>)>,
    }

    let mut col = Collect{ singles: vec![], options: vec![], alternatives: vec![] };

    qregister_wire_fns!{
        col <- single Foo{ init: Foo }
        col <- option SecondBase{ name: "2ndBase" }
        col <- alternative(SecondBase) Bar;
    }
}

