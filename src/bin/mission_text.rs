use std::{sync::Arc, ops::Deref, fmt::Debug, time::{Instant, Duration}, any::TypeId, hash::Hash, thread};

use bevy::{prelude::*, utils::HashMap, ecs::{entity::EntityMap, world::{EntityRef, EntityMut}}};
use bevy_egui::egui::mutex::Mutex;
use rand::prelude::Distribution;
use rayon::{str, ThreadBuilder};

pub type Id = u32;

pub struct QuestGenome {
    pub s0 : State,
    pub goal : Goal,
    pub sequence : Vec<Box<dyn Operator>>
}

pub type AtomCopy = Box<dyn Fn(&mut EntityMut, &EntityRef) + Send + Sync>;
pub type AtomDebug = Box<dyn Fn(&EntityRef) -> Option<String> + Send + Sync>;
pub type AtomEq = Box<dyn Fn(&EntityRef, &EntityRef) -> bool + Send + Sync>;
pub trait Atom : Debug { 
    fn name(&self) -> String;
    fn copy_fn() -> AtomCopy;
    fn debug_fn() -> AtomDebug;
    fn eq_fn() -> AtomEq;
}
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


pub struct State {
    pub world : World,
    pub ctx : Arc<StateConext>,
    pub id : u64
}

impl State {
    pub fn new(ctx : Arc<StateConext>) -> Self {
        let id;
        {
            let mut ctx_id = ctx.hash_indexer.lock();
            id = *ctx_id;
            *ctx_id += 1;
        }
        State {
            world : World::default(),
            ctx,
            id
        }
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
        self.id.hash(state);
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
        for src in self.world.iter_entities() {
            if let Some(mut dst) = new_world.get_or_spawn(src.id()) {
                for atom in self.ctx.writers.iter() {
                    atom(&mut dst, &src);
                }
            }
        }
        let mut state = State::new(self.ctx.clone());
        state.world = new_world;
        state
    }
}

impl State {
    pub fn successors(&mut self) -> Vec<(State, i32)> {
        
        let mut res = vec![];
        let ctx = self.ctx.clone();
        for rule in ctx.op_rules.iter() {
            res.extend(rule.batch_effect(self));
        }
        res
    }
}



pub struct Goal {
    pub pred : Vec<Box<dyn GoalPred>>
}

pub trait GoalPred : Debug {
    fn name(&self) -> String;
    fn precondition(&self, state : &State) -> bool;
}

pub trait OperatorRule {
    fn name(&self) -> String;
    fn can_effect(&self, state : &mut State) -> Vec<Box<dyn Operator>>;
    fn batch_effect(&self, state : &mut State) -> Vec<(State, i32)> {
        let mut ops = self.can_effect(state);
        let mut res = vec![];
        for op in ops {
            res.push((op.effect(state), 1));
        }
        res
    }
}

pub trait Operator {
    fn name(&self) -> String;
    fn effect(&self, state : &mut State) -> State;
}

pub struct GoRule {}

pub struct Go {
    pub id : Entity,
    pub move_to : Entity
}

impl OperatorRule for GoRule {
    fn name(&self) -> String {
        "Go".to_string()
    }

    fn can_effect(&self, state : &mut State) -> Vec<Box<dyn Operator>> {
        let mut query = state.world.query_filtered::<(Entity, &AtLocation), With<Ship>>();
        let mut res : Vec<Box<dyn Operator>> = vec![];
        for (e, at_loc) in query.iter(&state.world) {
            if let Some(loc) = state.world.get::<Location>(at_loc.id) {
                for p in loc.paths.iter() {
                    res.push(Box::new(Go {
                        id : e,
                        move_to : *p
                    }));
                    
                }
            }
        }

        res
    }
}

impl Operator for Go {
    fn name(&self) -> String {
        "Go".to_string()
    }

    fn effect(&self, state : &mut State) -> State {
        let mut new_state = state.clone();
        if let Some(mut new_at_loc) = new_state.world.get_mut::<AtLocation>(self.id) {
            new_at_loc.id = self.move_to;
        }
        new_state
    }
}

#[derive(Component, Debug)]
pub struct AtLocation {
    pub id : Entity,
}

impl Atom for AtLocation {
    fn name(&self) -> String {
        format!("AtLocation({:?})", self.id)
    }

    fn copy_fn() -> AtomCopy {
        Box::new(move |dst, src| {
            if let Some(data) = src.get::<AtLocation>() {
                dst.insert(AtLocation {id : data.id});
            }
        })
    }

    fn debug_fn() -> AtomDebug {
        Box::new(move |src| {
            if let Some(data) = src.get::<AtLocation>() {
                Some(format!("AtLocation({:?})", data.id))
            } else {
                None
            }
        })
    }

    fn eq_fn() -> AtomEq {
        Box::new(move |dst, src| {
            if let Some(data) = src.get::<AtLocation>() {
                if let Some(data2) = dst.get::<AtLocation>() {
                    data.id == data2.id
                } else {
                    false
                }
            } else if let Some(data2) = dst.get::<AtLocation>() {
                false
            } else {
                true
            }
            
        })
    }
}

#[derive(Component, Debug, Default, Clone)]
pub struct Location {
    pub paths : Vec<Entity>
}

impl Atom for Location {
    fn name(&self) -> String {
        "Location".to_string()
    }

    fn copy_fn() -> AtomCopy {
        Box::new(move |dst, src| {
            if let Some(data) = src.get::<Location>() {
                dst.insert(data.clone());
            }
        })
    }

    fn debug_fn() -> AtomDebug {
        Box::new(move |src| {
            if let Some(data) = src.get::<Location>() {
                Some(format!("Location with paths to {:?}", &data.paths))
            } else {
                None
            }
        })
    }

    fn eq_fn() -> AtomEq {
        Box::new(move |dst, src| {
            if let Some(data) = src.get::<Location>() {
                if let Some(data2) = dst.get::<Location>() {
                    data.paths == data2.paths
                } else {
                    false
                }
            } else if let Some(data2) = dst.get::<Location>() {
                false
            } else {
                true
            }
        })
    }
}

#[derive(Debug, Clone, Component, Default)]
pub struct Ship;

impl Atom for Ship {
    fn name(&self) -> String {
        "Ship".to_string()
    }

    fn copy_fn() -> AtomCopy {
        Box::new(move |dst, src| {
            if let Some(data) = src.get::<Ship>() {
                dst.insert(Ship);
            }
        })
    }

    fn debug_fn() -> AtomDebug {
        Box::new(move |src| {
            if let Some(data) = src.get::<Ship>() {
                Some(format!("Ship"))
            } else {
                None
            }
        })
    }

    fn eq_fn() -> AtomEq {
        Box::new(move |dst, src| {
            if let Some(data) = src.get::<Ship>() {
                if let Some(data2) = dst.get::<Ship>() {
                    true
                } else {
                    false
                }
            } else if let Some(data2) = dst.get::<Ship>() {
                false
            } else {
                true
            }
        })
    }
}

#[derive(Debug)]
pub struct GoalLocation {
    pub target_loc : Entity,
    pub target_obj : Entity
}

impl GoalPred for GoalLocation{
    fn name(&self) -> String {
        "GoalLocation".to_string()
    }

    fn precondition(&self, state : &State) -> bool {
        if let Some(obj) = state.world.get::<AtLocation>(self.target_obj) {
            obj.id == self.target_loc
        } else {
            false
        }
    }
}

fn main() {
    let mut ctx = StateConext::default();
    ctx.register_atom::<AtLocation>();
    ctx.register_atom::<Location>();
    ctx.register_atom::<Ship>();
    
    ctx.regiter_rule(GoRule {});


    let mut state = State::new(Arc::new(ctx));

    //generate star map
    let mut stars = vec![];
    for _ in 0..100 {
        let id = state.world.spawn(Location::default()).id();
        stars.push(id);
    }
    let mut rnd = rand::thread_rng();
    let star_distr = rand::distributions::Uniform::new(0, stars.len());
    for i in 0..100 {
        let links = rand::distributions::Uniform::new(1, 5).sample(&mut rnd);
        let mut star_loc = state.world.get_mut::<Location>(stars[i]).unwrap();
        for _ in 0..links {
            star_loc.paths.push(stars[star_distr.sample(&mut rnd)]);
        }
    }

    let ship_id = state.world.spawn(AtLocation {id : stars[0]})
        .insert(Ship)
        .id();

    let go = GoRule {};



    println!("{:#?}", state);
    println!("Test eq: {}", state == state.clone());
    let mut goal = GoalLocation {
        target_loc : stars[stars.len() - 1],
        target_obj : ship_id
    };

    let start_time = Instant::now();
    
    let mut find_thr = thread::spawn(move || {
        let state = state;
        let res = pathfinding::prelude::dijkstra(&state, |s| s.clone().successors(), |s| goal.precondition(s));
        res
    });
    while !find_thr.is_finished() && start_time.elapsed() < Duration::from_secs(1) {
        thread::sleep(Duration::from_millis(1));
    }
    let res = if find_thr.is_finished() {
        find_thr.join().unwrap()
    } else {
        None
    };
    // let res = pathfinding::prelude::dijkstra_all(&state, |s| s.clone().successors());
    
    let elapsed_time = start_time.elapsed();
    println!("Res (time {elapsed_time:?}): {:#?}", res);

    // for i in 0..1000 {
    //     state.world.spawn(AtLocation {id : ship_id}).id();
    //     state.world.spawn(Location::default());
    // }

    // let start = Instant::now();
    // for _ in 0..1000 {
    //     let state_2 = state.clone();
    // }
    // println!("{:?}", 1000.0 / start.elapsed().as_secs_f32());


    // pathfinding::prelude::astar(start, successors, heuristic, success)
}