use std::{sync::Arc, ops::Deref, fmt::Debug, time::{Instant, Duration}, any::TypeId, hash::Hash, thread};

use bevy::{prelude::*, utils::HashMap, ecs::{entity::EntityMap, world::{EntityRef, EntityMut}}};
use bevy_egui::egui::mutex::Mutex;
use rand::prelude::Distribution;
use rayon::{str, ThreadBuilder};
use SpaceSandbox::mission::*;
use SpaceSandbox::mission::State;

pub struct QuestGenome {
    pub s0 : State,
    pub goal : Goal,
    pub sequence : Vec<Box<dyn Operator>>
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
    let star_count = 10;
    for _ in 0..star_count {
        let id = state.world.spawn(Location::default()).id();
        stars.push(id);
    }
    let mut rnd = rand::thread_rng();
    let star_distr = rand::distributions::Uniform::new(0, stars.len());
    for i in 0..star_count {
        let links = rand::distributions::Uniform::new(1, 5).sample(&mut rnd);
        let mut star_loc = state.world.get_mut::<Location>(stars[i]).unwrap();
        for _ in 0..links {
            star_loc.paths.push(stars[star_distr.sample(&mut rnd)]);
        }
    }

    let ship_id = state.world.spawn(AtLocation {id : stars[0]})
        .insert(Ship)
        .id();

    let goal = Goal {
        pred : vec![Box::new(GoalLocation {target_loc : stars[stars.len() - 1], target_obj : ship_id})],
    };


    println!("{:#?}", &state);
    println!("Test eq: {}", state == state.clone());

    let start_time = Instant::now();
    
    let seq = find_sequence(&state, &goal);
    
    // let res = pathfinding::prelude::dijkstra_all(&state, |s| s.clone().successors());
    
    let elapsed_time = start_time.elapsed();
    println!("Res (time {elapsed_time:?}): {:#?}", seq);

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