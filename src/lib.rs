//! ### TODO
//!
//! * Clarify terminology (to wire, to load, option, alternative, option).
//! * Single alternative options are a _special case_ of options/alternatives. 
//!   To avoid confusion, all code & doc related to single alternative options 
//!   should be listed after regular options/alternatives.
//! * Panics vs error?
//! * remove_option, remove_alternative?

#![feature(get_type_id)]
#![feature(box_syntax)]

#[macro_use] extern crate qdowncast;
extern crate qindex_multi;
extern crate rustc_serialize;

#[macro_use] mod macros;

mod register;
pub use register::*;

