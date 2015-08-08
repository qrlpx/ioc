/** Utility macro for implementing `OptionReflect` and generating loading functions.

## `option`-statment

Statement for regular options.

Possible forms:

```rust
option $ty { name: $name }
option $ty;
```

Fields:

* `$ty`: the type `OptionReflect` will be implemented for
* `$name`: the `&'static str` that will be returned by `OptionReflect`, defaults to `stringify!($ty)`

## `alternative`-statment

Statement for regular alternatives.

Possible forms:

```rust
alternative($opt) $ty { name: $name, init: $init }
alternative($opt) $ty { init: $init, name: $name }
alternative($opt) $ty { init: $init }
alternative($opt) $ty { name: $name }
alternative($opt) $ty;
```

Fields:

* `$ty`: 
* `$opt`: the option the alternative will be added to
* `$name`: the name of the alternative
* `$init`: the `obj`-value that will be passed to `RegisterModifier::add_alternative`, defaults to
  `box <$ty as Default>::default() as Box<$opt>`

## `single`-statment

Statement for single alternative options.

Possible forms:

```rust
single $ty { name: $name, init: $init }
single $ty { init: $init, name: $name }
single $ty { init: $init }
single $ty { name: $name }
single $ty;
```

Fields:

* `$ty`: the type `OptionReflect` will be implemented for
* `$name`: the `&'static str` that will be returned by `OptionReflect`, defaults to `stringify!($ty)`
* `$init`: the `obj`-value that will be passed to `RegisterModifier::add_single`, defaults to
* `<$ty as Default>::default()`

# Example 

```rust
qregister_load_fns!{

    fn load_stores(&mut RegisterModifier<Store>) => {
        
        single PositionStore;
        
        single physics::LinearVelocityStore{ 
            name: "LinearVelocityStore" 
        }
        
        single ScoreStore{ 
            init: ScoreStore::from_file(cfg.get("score_file"))
        }

    }

    fn load_services(&mut RegisterModifier<Service>) => {

        single Timing;

        option Physics;

        alternative(Physics) nphysics::Physics{ 
            name: "NPhysicsPhysics" 
        }

        alternative(Physics) LegacyPhysics{ 
            init: LegacyPhysics::new("0.1.0") 
        }
    
    }
}

pub fn load_module(regs: &mut MyRegisters, cfg: &Cfg){
    
    load_stores(&mut regs.stores, cfg);
    load_services(&mut regs.services);
    
    register.wire_alternative("Physics", Some("LegacyPhysics"));
}
```
**/
#[macro_export]
macro_rules! qregister_load_fns {

    // 1.0 endpoint
    ($col:ident <-) => {};

    // 1.1 expansion
    ($col:ident <- single $ty:ty { name: $name:expr, init: $init:expr } $($ff:tt)*) => {
        {
            impl ::qregister::OptionReflect for $ty {
                fn option_name() -> &'static str { $name }
            }
        
            $col.singles.push(({$name}.to_string(), box {$init}));

            qregister_load_fns!($col <- $($ff)*);
        }
    };

    // 1.1 variation: $init and $name permutated
    ($col:ident <- single $ty:ty { init: $init:expr, name: $name:expr } $($ff:tt)*) => {
        qregister_load_fns!{
            $col <- single $ty { name: $name, init: $init } 
            $($ff)*
        }
    };

    // 1.1 variation: $name omitted, default to type-name
    ($col:ident <- single $ty:ty { init: $init:expr } $($ff:tt)*) => {
        qregister_load_fns!{
            $col <- single $ty { name: stringify!($ty), init: $init }
            $($ff)*
        }
    };

    // 1.1 variation: $init ommited, default to default-ctor
    ($col:ident <- single $ty:ty { name: $name:expr } $($ff:tt)*) => {
        qregister_load_fns!{
            $col <- single $ty { name: $name, init: <$ty as Default>::default() }
            $($ff)*
        }
    };

    // 1.1 variation: $name and $init ommitted, default type-name and default-ctor
    ($col:ident <- single $ty:ty; $($ff:tt)*) => {
        qregister_load_fns!{
            $col <- single $ty { name: stringify!($ty) }
            $($ff)*
        }
    };

    // 1.2 expansion
    ($col:ident <- option $ty:ty { name: $name:expr } $($ff:tt)*) => {
        {
            impl ::qregister::OptionReflect for $ty {
                fn option_name() -> &'static str { $name }
            }

            $col.options.push({$name}.to_string());

            qregister_load_fns!($col <- $($ff)*);
        }
    };

    // 1.2 variation: $name omitted, default to type-name
    ($col:ident <- option $ty:ty; $($ff:tt)*) => {
        qregister_load_fns!($col <- option $ty { name: stringify!($ty) });
    };

    // 1.3 expansion
    ($col:ident <- alternative($opt:ty) $ty:ty { name: $name:expr, init: $init:expr } $($ff:tt)*) => {
        {
            // utility to ensure that `t` is of type `T` (useful for compile-errors)
            fn _echo<T>(t: T) -> T { t }

            $col.alternatives.push((<$opt as ::qregister::OptionReflect>::option_name(), 
                                    {$name}.to_string(), 
                                    box _echo::<Box<$opt>>($init)));

            qregister_load_fns!($col <- $($ff)*);
        }
    };

    // 1.3 variation: $init and $name permutated
    ($col:ident <- alternative($opt:ty) $ty:ty { init: $init:expr, name: $name:expr } $($ff:tt)*) => {
        qregister_load_fns!{
            $col <- alternative($opt) $ty { name: $name, init: $init } 
            $($ff)*
        }
    };

    // 1.3 variation: $name omitted, default to type-name
    ($col:ident <- alternative($opt:ty) $ty:ty { init: $init:expr } $($ff:tt)*) => {
        qregister_load_fns!{
            $col <- alternative($opt) $ty { name: stringify!($ty), init: $init }
            $($ff)*
        }
    };

    // 1.3 variation: $init ommited, default to default-ctor
    ($col:ident <- alternative($opt:ty) $ty:ty { name: $name:expr } $($ff:tt)*) => {
        qregister_load_fns!{
            $col <- alternative($opt) $ty { name: $name, init: box <$ty as Default>::default() as Box<$opt> } 
            $($ff)*
        }
    };

    // 1.3 variation: $name and $init ommitted, default to type-name and default-ctor
    ($col:ident <- alternative($opt:ty) $ty:ty; $($ff:tt)*) => {
        qregister_load_fns!{
            $col <- alternative($opt) $ty { name: stringify!($ty) }
            $($ff)*
        }
    };

    // 2.0 endpoint
    () => {};

    // 2.1 expansion
    ($(#[$mmm:meta])* fn $name:ident($reg_type:ty, $($args:ident: $arg_types:ty),*) => { $($stmts:tt)+ } $($ff:tt)*) => {
        $(#[$mmm])*
        #[allow(unused)]
        fn $name(reg: $reg_type, $($args: $arg_types),*){
            struct Collect<Base: Any + ?Sized> {
                singles: Vec<(String, Box<Base>)>,
                options: Vec<String>,
                alternatives: Vec<(&'static str, String, Box<Base>)>,
            }
            let mut col = Collect{ singles: vec![], options: vec![], alternatives: vec![] };
            
            qregister_load_fns!(col <- $($stmts)+);

            for (name, obj) in col.singles {
                reg.add_single(name, obj);
            }
            for name in col.options {
                reg.add_option(name);
            }
            for (opt_name, alt_name, obj) in col.alternatives {
                reg.add_alternative(opt_name, alt_name, obj);
            }
        }
        qregister_load_fns!($($ff)*);
    };

    // 2.1 variation: no `,` after $reg_type
    ($(#[$mmm:meta])* fn $name:ident($reg_type:ty) => { $($stmts:tt)+ } $($ff:tt)*) => {
        qregister_load_fns!{
            $(#[$mmm])*
            fn $name($reg_type,) => {
                $($stmts)+
            }
            $($ff)*
        }
    };
}

/*
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

/*#[test]
pub fn test(){


    qregister_load_fns!{
        col <- single Foo{ init: Foo }
        col <- option SecondBase{ name: "2ndBase" }
        col <- alternative(SecondBase) Bar;
    }
}*/

#[derive(Default)]
pub struct Arb;
impl Base for Arb {}
impl SecondBase for Arb {}

qregister_load_fns!{
    fn test(&mut ::qregister::RegisterModifier<Base>, lel: i8) => {
        single Foo{ init: {lel + 2; Foo } }

        option SecondBase;

        alternative(SecondBase) Bar;
    }
    fn test_two(&mut ::qregister::RegisterModifier<SecondBase>) => {
        single Arb;
    }
}*/
