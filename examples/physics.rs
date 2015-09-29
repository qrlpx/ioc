#[macro_use] extern crate qregister;
use qregister::ServiceRegister;
use std::any::Any;

pub trait Physics: Any {
    fn progress(&mut self, dt: f32);
}

#[derive(Default)]
pub struct NPhysicsPhysics;

impl Physics for NPhysicsPhysics {
    fn progress(&mut self, _: f32) {
        println!("progressing nphysics-physics subsystem...");
    }
}

#[derive(Default)]
pub struct LegacyPhysics;

impl Physics for LegacyPhysics {
    fn progress(&mut self, _: f32) {
        println!("progressing legacy-physics subsystem...");
    }
}

qregister_load_fns!{
    fn load_module(&mut ServiceRegister) => {
    
        option Physics;

        alternative(Physics) NPhysicsPhysics;
        
        alternative(Physics) LegacyPhysics;
    }
}


fn main(){
    let mut reg = ServiceRegister::new();

    self::load_module(&mut reg);

    reg.wire_alternative("Physics", "NPhysicsPhysics");

    reg.services_mut().get_mut::<Box<Physics>>().unwrap().progress(1.0);

    reg.wire_alternative("Physics", "LegacyPhysics");

    reg.services_mut().get_mut::<Box<Physics>>().unwrap().progress(1.0);
}
