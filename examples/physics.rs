#[macro_use] extern crate qregister;
pub use qregister::Register;
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
    fn load_module(&mut Register) => {
    
        option Physics;

        alternative(Physics) NPhysicsPhysics;
        
        alternative(Physics) LegacyPhysics;
    }
}


fn main(){
    let mut reg = Register::new();

    self::load_module(&mut reg);

    reg.wire_alternative("Physics", "NPhysicsPhysics");

    reg.objects.get_mut::<Box<Physics>>().unwrap().progress(1.0);

    reg.wire_alternative("Physics", "LegacyPhysics");

    reg.objects.get_mut::<Box<Physics>>().unwrap().progress(1.0);
}
