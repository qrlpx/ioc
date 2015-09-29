use qdowncast::QDowncastable;
use qindex_multi::MultiIndexable;

use std::any::{Any, TypeId};
use std::borrow::Borrow;
use std::collections::btree_map::{self, BTreeMap};
use std::ops::{Index, IndexMut};

// ++++++++++++++++++++ ServiceReflect ++++++++++++++++++++ 

/// Our `Reflect`-derivative. 
/// This supplies necessary information for retrieving an service at compile-time.
pub trait ServiceReflect: Any {
    fn option_name() -> &'static str;
}

// ++++++++++++++++++++ GetSerectError ++++++++++++++++++++ 

#[derive(Debug, Clone)]
pub enum GetSerectError<'a> {
    TypeMismatch{ option_name: &'a str, expected: TypeId, found: TypeId },
    MissingOption(&'a str),
}

impl<'a> GetSerectError<'a> {
    pub fn type_mismatch<Expected>(found: TypeId) -> GetSerectError<'static> 
        where Expected: ServiceReflect
    {
        GetSerectError::TypeMismatch{
            option_name: Expected::option_name(),
            expected: TypeId::of::<Expected>(),
            found: found,
        }
    }
}

pub type GetSerectResult<'a, T> = Result<T, GetSerectError<'a>>;

// ++++++++++++++++++++ WiringOption ++++++++++++++++++++ 

/// TODO expose this to the user?
enum WiringOption<Ser: Any + ?Sized> {
    /// An option with zero or more alternatives. May be wired to one of its services.
    Multi{
        wired: Option<usize>,
        alternatives: Vec<(String, Box<Ser>)>,
    },
    /// A single alternative option. Is always wired to exactly one service.
    Single(Box<Ser>),
}

impl<Ser: Any + ?Sized> WiringOption<Ser> {
    fn has_alternative(&self, alt_name: &str) -> bool {
        match self {
            &WiringOption::Multi{ ref alternatives, .. } => {
                alternatives.iter().any(|e| &*e.0 == alt_name)
            }
            _ => false,
        }
    }

    fn add_alternative(&mut self, alt_name: String, service: Box<Ser>){
        assert!(!self.has_alternative(&alt_name), 
                "Alternative '{}' already exists,", alt_name);

        match self {
            &mut WiringOption::Multi{ ref mut alternatives, .. } => {
                alternatives.push((alt_name, service));
            }
            _ => { 
                panic!("can't add alternative '{}' to single alternative option", alt_name);
            }
        }
    }

    fn wire_alternative(&mut self, alt_name: &str) {
        assert!(self.has_alternative(&alt_name), 
                "Can't wire missing alternative '{}'", alt_name);

        match self {
            &mut WiringOption::Multi{ ref mut wired, ref alternatives } => {
                *wired = Some(alternatives.iter().position(|e| &*e.0 == alt_name).unwrap());
            }
            _ => { unreachable!() }
        }
    }

    fn service(&self) -> Option<&Ser> {
        match self {
            &WiringOption::Single(ref service) => Some(&**service),
            &WiringOption::Multi{ wired, ref alternatives } => match wired {
                Some(idx) => Some(&*alternatives[idx].1), 
                None => None,
            }
        }
    }

    fn service_mut(&mut self) -> Option<&mut Ser> {
        match self {
            &mut WiringOption::Single(ref mut service) => Some(&mut**service),
            &mut WiringOption::Multi{ wired, ref mut alternatives } => match wired {
                Some(idx) => Some(&mut*alternatives[idx].1), 
                None => None,
            }
        }
    }
}

// ++++++++++++++++++++ ServiceMap ++++++++++++++++++++ 

pub trait DefaultBase: Any {}
impl<T: Any + ?Sized> DefaultBase for T {}
qdowncastable!(DefaultBase);
qdowncast_methods!(DefaultBase);

pub struct ServiceMap<Ser: Any + ?Sized = DefaultBase> {
    options: BTreeMap<String, WiringOption<Ser>>,
}

impl<Ser: Any + ?Sized> ServiceMap<Ser> {
    /// Gets the service wired to option `opt_name` immutably.
    pub fn get_service(&self, opt_name: &str) -> Option<&Ser> {
        self.options.get(opt_name).and_then(|option| option.service())
    }

    /// Gets the service wired to option `opt_name` mutably.
    pub fn get_service_mut(&mut self, opt_name: &str) -> Option<&mut Ser> {
        self.options.get_mut(opt_name).and_then(|option| option.service_mut())
    }

    /// Gets the service wired to option `opt_name` immutably, then tries to downcast it.
    pub fn get<T>(&self) -> GetSerectResult<&T> 
        where T: ServiceReflect, Ser: QDowncastable<T>
    { 
        match self.get_service(T::option_name()) {
            Some(base) => {
                let ty = (&*base).get_type_id();
                match QDowncastable::downcast_ref(base) {
                    Some(ret) => Ok(ret),
                    None => Err(GetSerectError::type_mismatch::<T>(ty))
                }
            }
            None => Err(GetSerectError::MissingOption(T::option_name()))
        }
    }

    /// Gets the service wired to option `opt_name` mutably, then tries to downcast it.
    pub fn get_mut<T>(&mut self) -> GetSerectResult<&mut T> 
        where T: ServiceReflect, Ser: QDowncastable<T>
    { 
        match self.get_service_mut(T::option_name()) {
            Some(base) => {
                let ty = (&*base).get_type_id();
                match QDowncastable::downcast_mut(base) {
                    Some(ret) => Ok(ret),
                    None => Err(GetSerectError::type_mismatch::<T>(ty))
                }
            }
            None => Err(GetSerectError::MissingOption(T::option_name()))
        }
    }

    /// Iterate over all wired services immutably.
    pub fn iter(&self) -> Iter<Ser> {
        Iter{ options: self.options.iter() }
    }

    /// Iterate over all wired services mutably.
    pub fn iter_mut(&mut self) -> IterMut<Ser> {
        IterMut{ options: self.options.iter_mut() }
    }
}

/// TODO impl more Iterator-traits?
#[derive(Clone)]
pub struct Iter<'a, Ser: Any + ?Sized = DefaultBase> {
    options: btree_map::Iter<'a, String, WiringOption<Ser>>,
}

impl<'a, Ser: Any + ?Sized> Iterator for Iter<'a, Ser> {
    type Item = (&'a str, &'a Ser);
    fn next(&mut self) -> Option<Self::Item> {
        match self.options.next() {
            Some((opt_name, option)) => match option.service() {
                Some(service) => Some((&opt_name, service)),
                None => self.next(),
            },
            None => None,
        }
    }
}

/// TODO impl more Iterator-traits?
pub struct IterMut<'a, Ser: Any + ?Sized = DefaultBase> {
    options: btree_map::IterMut<'a, String, WiringOption<Ser>>,
}

impl<'a, Ser: Any + ?Sized> Iterator for IterMut<'a, Ser> {
    type Item = (&'a str, &'a mut Ser);
    fn next(&mut self) -> Option<Self::Item> {
        match self.options.next() {
            Some((opt_name, option)) => match option.service_mut() {
                Some(service) => Some((&opt_name, service)),
                None => self.next(),
            },
            None => None,
        }
    }
}

impl<'a, Ser: Any + ?Sized> IntoIterator for &'a ServiceMap<Ser> {
    type Item = <Self::IntoIter as Iterator>::Item;
    type IntoIter = Iter<'a, Ser>;
    fn into_iter(self) -> Self::IntoIter { self.iter() }
}

impl<'a, Ser: Any + ?Sized> IntoIterator for &'a mut ServiceMap<Ser> {
    type Item = <Self::IntoIter as Iterator>::Item;
    type IntoIter = IterMut<'a, Ser>;
    fn into_iter(self) -> Self::IntoIter { self.iter_mut() }
}

impl<'a, Str, Ser: ?Sized> Index<&'a Str> for ServiceMap<Ser> 
    where Str: Ord + Borrow<str>, Ser: Any
{
    type Output = Ser;
    fn index(&self, name: &'a Str) -> &Self::Output { 
        self.get_service(name.borrow()).unwrap()
    }
}

impl<'a, Str, Ser: ?Sized> IndexMut<&'a Str> for ServiceMap<Ser> 
    where Str: Ord + Borrow<str>, Ser: Any
{
    fn index_mut(&mut self, name: &'a Str) -> &mut Self::Output { 
        self.get_service_mut(name.borrow()).unwrap()
    }
}

unsafe impl<'a, Str, Ser: ?Sized> MultiIndexable<&'a Str> for ServiceMap<Ser> 
    where Str: Ord + Borrow<str>, Ser: Any
{}

// ++++++++++++++++++++ ServiceRegister ++++++++++++++++++++ 

pub struct ServiceRegister<Ser: Any + ?Sized = DefaultBase>{
    services: ServiceMap<Ser>
}

// TODO: remove duplicated code
impl<Ser: Any + ?Sized> ServiceRegister<Ser> {
    pub fn new() -> ServiceRegister<Ser> { 
        ServiceRegister{
            services: ServiceMap{ options: BTreeMap::new() }
        }
    }

    pub fn services(&self) -> &ServiceMap<Ser> { &self.services }
    pub fn services_mut(&mut self) -> &mut ServiceMap<Ser> { &mut self.services }
    pub fn into_services(self) -> ServiceMap<Ser> { self.services }

    /// Adds a option to the register.
    pub fn add_option(&mut self, name: String){
        assert!(!self.services.options.contains_key(&name), "option '{}' already exists!", &name);

        self.services.options.insert(name, WiringOption::Multi{
            wired: None, alternatives: Vec::new()
        });
    }

    /// Adds an alternative to an option of the register.
    pub fn add_alternative(&mut self, opt_name: &str, alt_name: String, service: Box<Ser>){
        let option = self.services.options.get_mut(opt_name);
        let option = option.expect(&format!("option '{}' doesn't exist", &opt_name));

        option.add_alternative(alt_name, service);
    }

    /// Wires an alternative of an option of this register.
    pub fn wire_alternative(&mut self, opt_name: &str, alt_name: &str){
        let option = self.services.options.get_mut(opt_name);
        let option = option.expect(&format!("option '{}' doesn't exist", &opt_name));
        
        option.wire_alternative(alt_name);
    }

    /// Adds a single alternative option to the register.
    pub fn add_single(&mut self, name: String, service: Box<Ser>){
        assert!(self.services.options.contains_key(&name), "option '{}' already exists!", &name);

        self.services.options.insert(name, WiringOption::Single(service));
    }
}

