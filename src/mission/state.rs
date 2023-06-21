use std::borrow::BorrowMut;
use std::hash::Hasher;
use std::marker::PhantomData;
use std::sync::{Mutex, Arc};

use bevy::prelude::*;
use downcast_rs::Downcast;
use super::FindNode;
use super::atom::*;
use super::operator::*;
use std::fmt::Debug;
use std::hash::Hash;

use std::any::{Any, TypeId};
use std::collections::HashMap;

#[derive(Default)]
pub struct StateConext {
    pub writers : Vec<AtomCopy>,
    pub debuggers : Vec<AtomDebug>,
    pub equals : Vec<AtomEq>,
    pub op_rules : Vec<Box<dyn OperatorRule + Send + Sync>>,
    pub hash_indexer : Mutex<u64>
}


impl StateConext {
    pub fn register_atom<T: Atom>(&mut self) {
        self.writers.push(T::copy_fn());
        self.debuggers.push(T::debug_fn());
        self.equals.push(T::eq_fn());
    }

    pub fn regiter_rule<T: OperatorRule + 'static + Send + Sync>(&mut self, rule : T) {
        self.op_rules.push(Box::new(rule));
    }
}

pub enum StateEntity {
    DynObj(Entity),
    StaticObj(Entity)
}

pub struct State {
    pub world : World,
    pub ctx : Arc<StateConext>,
    pub hash : u64
}

impl State {
    pub fn new(ctx : Arc<StateConext>) -> Self {
        State {
            world : World::default(),
            ctx,
            hash : 0
        }
    }

    pub fn setup_hash(&mut self) {
        let mut query = self.world.query::<&AtLocation>();
        let mut hash = 0;
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        for at_loc in query.iter(&self.world) {
            at_loc.id.hash(&mut hasher);
        }
        self.hash = hasher.finish();
    }
}

impl PartialEq for State {
    fn eq(&self, other: &Self) -> bool {
        for entity in self.world.iter_entities() {
            let e = other.world.entity(entity.id());
            for atom in self.ctx.equals.iter() {
                if !atom(&entity, &e) {
                    return false;
                }
            }
        }
        true
    }
}

impl Hash for State {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.hash.hash(state);
    }
}

impl Eq for State {

}

impl Debug for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = f.debug_list();
        for entity in self.world.iter_entities() {
            let mut res = format!("{:?}", entity.id()).to_string();
            for atom in self.ctx.debuggers.iter() {
                if let Some(s) = atom(&entity) {
                    res = format!("{res}, {s}");
                }
            }
            s.entry(&res);
        }
        s.finish()
    }
}


impl Clone for State {
    fn clone(&self) -> Self {
        let mut new_world = World::default();

        // for e in self.world.iter_entities() {
        //     new_world.get_or_spawn(e.id());
        // }

        // for atom in self.ctx.writers.iter() {
        //     atom(&mut new_world, &self.world);
        // }
        
        for src in self.world.iter_entities() {

            

            if let Some(mut dst) = new_world.get_or_spawn(src.id()) {
                for atom in self.ctx.writers.iter() {
                    atom(&mut dst, &src);
                }
            }
        }


        let mut state = State::new(self.ctx.clone());
        state.world = new_world;
        state.setup_hash();
        state
    }
}

impl State {
    pub fn successors(&mut self) -> Vec<(FindNode, i32)> {
        
        let mut res = vec![];
        let ctx = self.ctx.clone();
        for rule in ctx.op_rules.iter() {
            res.extend(rule.batch_effect(self));
        }
        res
    }
}

