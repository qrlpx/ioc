//! ### TODO
//!
//! * Clarify terminology (to wire, to load, option, alternative, option).
//! * Single alternative options are a _special case_ of options/alternatives. 
//!   To avoid confusion, all code & doc related to single alternative options 
//!   should be listed after regular options/alternatives.

#![feature(get_type_id)]
#![feature(box_syntax)]

#[macro_use] extern crate qdowncast;
extern crate qindex_multi;

#[macro_use] mod macros;

use qdowncast::QDowncastable;
use qindex_multi::MultiIndexable;

use std::any::{Any, TypeId};
use std::borrow::Borrow;
use std::collections::btree_map::{self, BTreeMap};
use std::ops::{Deref, DerefMut, Index, IndexMut};

// ++++++++++++++++++++ OptionReflect ++++++++++++++++++++ 

/// Our `Reflect`-derivative. 
/// This supplies necessary information for retrieving an object at compile-time.
pub trait OptionReflect: Any {
    fn option_name() -> &'static str;
}

// ++++++++++++++++++++ GetObjectError ++++++++++++++++++++ 

#[derive(Debug, Clone)]
pub enum GetObjectError<'a> {
    TypeMismatch{ option_name: &'a str, expected: TypeId, found: TypeId },
    MissingOption(&'a str),
}

impl<'a> GetObjectError<'a> {
    pub fn type_mismatch<Expected>(found: TypeId) -> GetObjectError<'static> 
        where Expected: OptionReflect
    {
        GetObjectError::TypeMismatch{
            option_name: Expected::option_name(),
            expected: TypeId::of::<Expected>(),
            found: found,
        }
    }
}

pub type GetObjectResult<'a, T> = Result<T, GetObjectError<'a>>;

// ++++++++++++++++++++ Entry ++++++++++++++++++++ 

/// TODO expose this to the user?
enum Entry<Obj: Any + ?Sized> {
    /// A option with zero or more alternatives. May be wired to one of its objects.
    Multi{
        wired: Option<usize>,
        alternatives: Vec<(String, Box<Obj>)>,
    }
    /// A single alternative option. Is always wired to exactly one object.
    Single(Box<Obj>),
}

impl<Obj: Any + ?Sized> Entry<Obj> {
    fn has_alternative(&self, alt_name: &str) -> bool {
        match self {
            &Entry::Multi{ ref alternatives, .. } => {
                alternatives.iter().any(|e| &*e.0 == alt_name)
            }
            _ => false,
        }
    }

    fn add_alternative(&mut self, alt_name: String, obj: Box<Obj>){
        assert!(!self.has_alternative(&alt_name), 
                "Alternative '{}' already exists,", alt_name);

        match self {
            &mut Entry::Multi{ ref mut alternatives, .. } => {
                alternatives.push((alt_name, obj));
            }
            _ => { 
                panic!("Can't add alternative '{}' to single alternative option", alt_name);
            }
        }
    }

    fn wire_alternative(&mut self, alt_name: &str) {
        assert!(self.has_alternative(&alt_name), 
                "Can't wire missing alternative '{}'", alt_name);

        match self {
            &mut Entry::Multi{ ref mut wired, ref alternatives } => {
                *wired = Some(alternatives.iter().position(|e| &*e.0 == alt_name).unwrap());
            }
            _ => { unreachable!() }
        }
    }

    fn object_ref(&self) -> Option<&Obj> {
        match self {
            &Entry::Single(ref obj) => Some(&**obj),
            &Entry::Multi{ wired, ref alternatives } => match wired {
                Some(idx) => Some(&*alternatives[idx].1), 
                None => None,
            }
        }
    }

    fn object_mut(&mut self) -> Option<&mut Obj> {
        match self {
            &mut Entry::Single(ref mut obj) => Some(&mut**obj),
            &mut Entry::Multi{ wired, ref mut alternatives } => match wired {
                Some(idx) => Some(&mut*alternatives[idx].1), 
                None => None,
            }
        }
    }
}

// ++++++++++++++++++++ Register ++++++++++++++++++++ 

pub trait DefaultBase: Any {}
impl<T: Any + ?Sized> DefaultBase for T {}
qdowncastable!(DefaultBase);
qdowncast_methods!(DefaultBase);

pub struct Register<Obj: Any + ?Sized = DefaultBase> {
    entrys: BTreeMap<String, Entry<Obj>>,
}

impl<Obj: Any + ?Sized> Register<Obj> {
    /// Gets the object wired to option `opt_name` immutably.
    pub fn get_object(&self, opt_name: &str) -> Option<&Obj> {
        self.entrys.get(opt_name).and_then(|entry| entry.object_ref())
    }

    /// Gets the object wired to option `opt_name` mutably.
    pub fn get_object_mut(&mut self, opt_name: &str) -> Option<&mut Obj> {
        self.entrys.get_mut(opt_name).and_then(|entry| entry.object_mut())
    }

    /// Gets the object wired to option `opt_name` immutably, then tries to downcast it.
    pub fn get<T>(&self) -> GetObjectResult<&T> 
        where T: OptionReflect, Obj: QDowncastable<T>
    { 
        match self.get_object(T::option_name()) {
            Some(base) => {
                let ty = (&*base).get_type_id();
                match QDowncastable::downcast_ref(base) {
                    Some(ret) => Ok(ret),
                    None => Err(GetObjectError::type_mismatch::<T>(ty))
                }
            }
            None => Err(GetObjectError::MissingOption(T::option_name()))
        }
    }

    /// Gets the object wired to option `opt_name` mutably, then tries to downcast it.
    pub fn get_mut<T>(&mut self) -> GetObjectResult<&mut T> 
        where T: OptionReflect, Obj: QDowncastable<T>
    { 
        match self.get_object_mut(T::option_name()) {
            Some(base) => {
                let ty = (&*base).get_type_id();
                match QDowncastable::downcast_mut(base) {
                    Some(ret) => Ok(ret),
                    None => Err(GetObjectError::type_mismatch::<T>(ty))
                }
            }
            None => Err(GetObjectError::MissingOption(T::option_name()))
        }
    }

    /// Iterate over all wired objects immutably.
    pub fn iter(&self) -> Iter<Obj> {
        Iter{ entrys: self.entrys.iter() }
    }

    /// Iterate over all wired objects mutably.
    pub fn iter_mut(&mut self) -> IterMut<Obj> {
        IterMut{ entrys: self.entrys.iter_mut() }
    }
}

#[derive(Clone)]
pub struct Iter<'a, Obj: Any + ?Sized = DefaultBase> {
    entrys: btree_map::Iter<'a, String, Entry<Obj>>,
}

impl<'a, Obj: Any + ?Sized> Iterator for Iter<'a, Obj> {
    type Item = (&'a str, &'a Obj);
    fn next(&mut self) -> Option<Self::Item> {
        match self.entrys.next() {
            Some((opt_name, entry)) => match entry.object_ref() {
                Some(obj) => Some((&opt_name, obj)),
                None => self.next(),
            },
            None => None,
        }
    }
}

pub struct IterMut<'a, Obj: Any + ?Sized = DefaultBase> {
    entrys: btree_map::IterMut<'a, String, Entry<Obj>>,
}

impl<'a, Obj: Any + ?Sized> Iterator for IterMut<'a, Obj> {
    type Item = (&'a str, &'a mut Obj);
    fn next(&mut self) -> Option<Self::Item> {
        match self.entrys.next() {
            Some((opt_name, entry)) => match entry.object_mut() {
                Some(obj) => Some((&opt_name, obj)),
                None => self.next(),
            },
            None => None,
        }
    }
}

impl<'a, Obj: Any + ?Sized> IntoIterator for &'a Register<Obj> {
    type Item = <Self::IntoIter as Iterator>::Item;
    type IntoIter = Iter<'a, Obj>;
    fn into_iter(self) -> Self::IntoIter { self.iter() }
}

impl<'a, Obj: Any + ?Sized> IntoIterator for &'a mut Register<Obj> {
    type Item = <Self::IntoIter as Iterator>::Item;
    type IntoIter = IterMut<'a, Obj>;
    fn into_iter(self) -> Self::IntoIter { self.iter_mut() }
}

impl<'a, Str, Obj: ?Sized> Index<&'a Str> for Register<Obj> 
    where Str: Ord + Borrow<str>, Obj: Any
{
    type Output = Obj;
    fn index(&self, name: &'a Str) -> &Self::Output { 
        self.get_object(name.borrow()).unwrap()
    }
}

impl<'a, Str, Obj: ?Sized> IndexMut<&'a Str> for Register<Obj> 
    where Str: Ord + Borrow<str>, Obj: Any
{
    fn index_mut(&mut self, name: &'a Str) -> &mut Self::Output { 
        self.get_object_mut(name.borrow()).unwrap()
    }
}

unsafe impl<'a, Str, Obj: ?Sized> MultiIndexable<&'a Str> for Register<Obj> 
    where Str: Ord + Borrow<str>, Obj: Any
{}

// ++++++++++++++++++++ RegisterModifier ++++++++++++++++++++ 

pub struct RegisterModifier<Obj: Any + ?Sized = DefaultBase>(Register<Obj>);

impl<Obj: Any + ?Sized> Deref for RegisterModifier<Obj> {
    type Target = Register<Obj>;
    fn deref(&self) -> &Self::Target { &self.0 }
}
impl<Obj: Any + ?Sized> DerefMut for RegisterModifier<Obj> {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}

// TODO: remove duplicated code
impl<Obj: Any + ?Sized> RegisterModifier<Obj> {
    pub fn new() -> RegisterModifier<Obj> { 
        RegisterModifier(Register{ entrys: BTreeMap::new() })
    }

    pub fn unwrap(self) -> Register<Obj> { self.0 }

    /// Adds a option to the register.
    pub fn add_option(&mut self, name: String){
        assert!(self.entrys.contains_key(&name), "Option '{}' already exists!", &name);

        self.entrys.insert(name, Entry::Multi{
            wired: None, alternatives: Vec::new()
        });
    }

    /// Adds an alternative to an option of the register.
    pub fn add_alternative(&mut self, opt_name: &str, alt_name: String, obj: Box<Obj>){
        let entry = self.entrys.get_mut(opt_name);
        let entry = entry.expect(&format!("Option '{}' doesn't exist", &opt_name));

        entry.add_alternative(alt_name, obj);
    }

    /// Wires an alternative of an option of this register.
    pub fn wire_alternative(&mut self, opt_name: &str, alt_name: &str){
        let entry = self.entrys.get_mut(opt_name);
        let entry = entry.expect(&format!("Option '{}' doesn't exist", &opt_name));
        
        entry.wire_alternative(alt_name);
    }

    /// Adds a single alternative option to the register.
    pub fn add_single(&mut self, name: String, obj: Box<Obj>){
        assert!(self.entrys.contains_key(&name), "Option '{}' already exists!", &name);

        self.entrys.insert(name, Entry::Single(obj));
    }
}

