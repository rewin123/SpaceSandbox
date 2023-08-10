use bevy::{prelude::*, ecs::world::{EntityMut, EntityRef}, utils::HashSet};
use std::fmt::Debug;

pub type AtomCopy = Box<dyn Fn(&mut EntityMut, &EntityRef) + Send + Sync>;
pub type AtomDebug = Box<dyn Fn(&EntityRef) -> Option<String> + Send + Sync>;
pub type AtomEq = Box<dyn Fn(&EntityRef, &EntityRef) -> bool + Send + Sync>;
pub trait Atom : Debug { 
    fn name(&self) -> String;
    fn copy_fn() -> AtomCopy;
    fn debug_fn() -> AtomDebug;
    fn eq_fn() -> AtomEq;
}

fn auto_copy_fn<T: Atom + Component + Clone>() -> AtomCopy {
    Box::new(move |dst, src| {
        // for e in src.iter_entities() {
            if let Some(val) = src.get::<T>() {
                dst.insert(val.clone());
            }
        // }
    })
}

fn auto_debug_fn<T: Atom + Component>() -> AtomDebug {
    Box::new(move |src| {
        src.get::<T>().map(|data| format!("{:?}", data))
    })
}

fn auto_eq_fn<T: Atom + Component + Eq>() -> AtomEq {
    Box::new(move |dst, src| {
        if let Some(data) = src.get::<T>() {
            if let Some(data2) = dst.get::<T>() {
                *data == *data2
            } else {
                false
            }
        } else if let Some(_data2) = dst.get::<T>() {
            false
        } else {
            true
        }
    })
}

#[derive(Component, Debug, Clone, Eq, PartialEq)]
pub struct AtLocation {
    pub id : Entity,
}

impl Atom for AtLocation {
    fn name(&self) -> String {
        format!("AtLocation({:?})", self.id)
    }

    fn copy_fn() -> AtomCopy {
        auto_copy_fn::<AtLocation>()
    }

    fn debug_fn() -> AtomDebug {
       auto_debug_fn::<AtLocation>()
    }

    fn eq_fn() -> AtomEq {
        auto_eq_fn::<AtLocation>()
    }
}

#[derive(Component, Debug, Default, Clone, Eq, PartialEq)]
pub struct Location {
    pub paths : Vec<Entity>
}

impl Atom for Location {
    fn name(&self) -> String {
        "Location".to_string()
    }

    fn copy_fn() -> AtomCopy {
        auto_copy_fn::<Location>()
    }

    fn debug_fn() -> AtomDebug {
       auto_debug_fn::<Location>()
    }

    fn eq_fn() -> AtomEq {
        auto_eq_fn::<Location>()
    }
}

#[derive(Debug, Clone, Component, Default, Eq, PartialEq)]
pub struct Ship;

impl Atom for Ship {
    fn name(&self) -> String {
        "Ship".to_string()
    }

    fn copy_fn() -> AtomCopy {
        auto_copy_fn::<Self>()
    }

    fn debug_fn() -> AtomDebug {
        auto_debug_fn::<Self>()
    }

    fn eq_fn() -> AtomEq {
        auto_eq_fn::<Self>()
    }
}

#[derive(Debug, Clone, Component, Default, Eq, PartialEq)]
pub struct HasItem {
    pub items : HashSet<Entity>,
}

impl Atom for HasItem {
    fn name(&self) -> String {
        "HasItem".to_string()
    }

    fn copy_fn() -> AtomCopy {
        auto_copy_fn::<HasItem>()
    }

    fn debug_fn() -> AtomDebug {
        auto_debug_fn::<HasItem>()
    }

    fn eq_fn() -> AtomEq {
        auto_eq_fn::<HasItem>()
    }
}

#[derive(Debug, Clone, Component, Default, Eq, PartialEq)]
pub struct Item;

impl Atom for Item {
    fn name(&self) -> String {
        "Item".to_string()
    }

    fn copy_fn() -> AtomCopy {
        auto_copy_fn::<Item>()
    }

    fn debug_fn() -> AtomDebug {
        auto_debug_fn::<Item>()
    }

    fn eq_fn() -> AtomEq {
        auto_eq_fn::<Item>()
    }
}

#[derive(Debug, Clone, Component, Default, Eq, PartialEq)]
pub struct Player;

impl Atom for Player {
    fn name(&self) -> String {
        "Player".to_string()
    }

    fn copy_fn() -> AtomCopy {
        auto_copy_fn::<Player>()
    }

    fn debug_fn() -> AtomDebug {
        auto_debug_fn::<Player>()
    }

    fn eq_fn() -> AtomEq {
        auto_eq_fn::<Player>()
    }
}

#[derive(Debug, Clone, Component, Default, Eq, PartialEq)]
pub struct Enemy;
impl Atom for Enemy {
    fn name(&self) -> String {
        "Enemy".to_string()
    }

    fn copy_fn() -> AtomCopy {
        auto_copy_fn::<Enemy>()
    }

    fn debug_fn() -> AtomDebug {
        auto_debug_fn::<Enemy>()
    }

    fn eq_fn() -> AtomEq {
        auto_eq_fn::<Enemy>()
    }
}