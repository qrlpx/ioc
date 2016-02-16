#![feature(associated_type_defaults)] 
#![feature(reflect_marker)]
#![feature(core_intrinsics)]
#![feature(box_syntax)]

#[macro_use] 
extern crate downcast;

mod reflect;
mod errors;
mod guards;
//mod factory;
mod methods;
mod container;

pub use reflect::*;
pub use errors::*;
pub use guards::*;
pub use methods::*;
//pub use factory::*;
pub use container::*;

/// Alias for `Read`.
pub use Read as R;

/// Alias for `Write`.
pub use Write as W;

// NOTE old code
// TODO move this to tests/examples
/*#[macro_use] 
extern crate lazy_static;

#[test]
fn read_trait_object(){    
    macro_rules! service {
        ($ty:ty, $name:expr) => {
            impl ServiceReflect for $ty {
                fn key() -> &'static String {
                    lazy_static!{
                        static ref RET: String = $name.into();
                    }
                    &*RET
                }
            }

            impl Into<Box<DefaultBase>> for Box<$ty> {
                fn into(self) -> Box<DefaultBase> { self }
            }
        };
    }

    trait Foo: Sync {
        fn foo(&self) -> &str;
    }

    service!(Box<Foo>, "foo");

    struct FooBar;

    impl Foo for FooBar{
        fn foo(&self) -> &str { "bar" }  
    }

    let mut builder = ContainerBuilder::<String>::new();
    builder.register(Box::new(FooBar) as Box<Foo>);

    let ioc = builder.build();

    assert_eq!("bar", ioc.read::<Box<Foo>>().unwrap().foo());
}*/


