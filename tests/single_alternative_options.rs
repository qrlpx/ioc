// TODO move this into `tests/`

#[macro_use]
extern crate qregister;

#[derive(Default)]
pub struct NameAndInit;

#[derive(Default)]
pub struct InitAndName;

#[derive(Default)]
pub struct Init;

#[derive(Default)]
pub struct Name;

#[derive(Default)]
pub struct Semicolon;

qregister_load_fns!{
    fn load_fn(&mut qregister::Register) => {
        single NameAndInit { name: "NameAndInit", init: NameAndInit }
        single InitAndName { name: "InitAndName", init: InitAndName }
        single Init { init: Init }
        single Name { name: "Name" }
        single Semicolon;
    }
}

