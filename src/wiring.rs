use register::Register;

use std::any::Any;
use std::collections::BTreeMap;

/// Utility struct.
///
/// TODO remove this?
///
/// TODO provide some method to export/save a `Wiring` from a `Register`?
#[derive(RustcEncodable, RustcDecodable)]
#[derive(Default, Debug, Clone)]
pub struct Wiring(pub BTreeMap<String, String>);

impl Wiring {
    pub fn apply<Base: Any + ?Sized>(&self, reg: &mut Register<Base>){
        for (opt, alt) in &self.0 {
            reg.wire_alternative(opt, alt);
        }
    }
}

