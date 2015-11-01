//! ### TODO
//!
//! * impl {Error, Display} for errors?

#![feature(get_type_id)] 
#![feature(associated_type_defaults)] 

#[macro_use] extern crate downcast;

mod service;
mod factory;
mod invocation_method;
mod ioc;

pub use service::*;
pub use factory::*;
pub use invocation_method::*;
pub use ioc::*;

#[test]
fn read_trait_object(){
    macro_rules! service {
        ($ty:ty, $name:expr) => {
            impl ServiceReflect for $ty {
                fn key() -> &'static str { $name }
            }

            impl Into<Box<DefaultBase>> for Box<$ty> {
                fn into(self) -> Box<DefaultBase> { self }
            }
        };
    }

    trait Foo {
        fn foo(&self) -> &str;
    }

    service!(Box<Foo>, "foo");

    struct FooBar;

    impl Foo for FooBar{
        fn foo(&self) -> &str { "bar" }  
    }

    let mut builder = IocBuilder::<String>::new();
    builder.register(Box::new(FooBar) as Box<Foo>);

    let ioc = builder.build();

    assert_eq!("bar", ioc.read::<Box<Foo>>().unwrap().foo());
}


