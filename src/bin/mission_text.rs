use std::{sync::{Arc, RwLock}, ops::Deref, fmt::Debug, time::{Instant, Duration}, any::TypeId, hash::Hash, thread};

use bevy::{prelude::*, utils::{HashMap, HashSet}, ecs::{entity::EntityMap, world::{EntityRef, EntityMut}}};
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
    ctx.register_atom::<Item>();
    ctx.register_atom::<HasItem>();
    ctx.register_atom::<Player>();
    ctx.register_atom::<Enemy>();
    
    ctx.regiter_rule(GoRule {});
    ctx.regiter_rule(TakeRule {});


    let mut state = State::new(Arc::new(ctx));
    let goal = generate_state(&mut state);

    println!("{:#?}", &state);
    println!("Test eq: {}", state == state.clone());

    println!("Banching state copy...");
    {
        let time_start = Instant::now();
        for _ in 0..1000 {
            let state_2 = state.clone();
        }
        println!("Copy per second: {:?}", 1000.0 / time_start.elapsed().as_secs_f32());
    }

    let start_time = Instant::now();
    
    let seq = find_sequence(&state, &goal);
    
    let elapsed_time = start_time.elapsed();
    if let Some(seq) = seq {
        println!("Res (time {elapsed_time:?}): {}", print_planning_plan(&state.world, &seq));
    } else {
        println!("No sequence");
    }
}

fn generate_state(state: &mut State) -> Goal {
    //generate star map
    let mut stars = vec![];
    let star_count = 5;
    for i in 0..star_count {
        let id = state.world.spawn(Location::default())
            .insert(Name::new(format!("Star {i}")))
            .id();
        stars.push(id);


        let enemy_ship = state.world.spawn(Ship)
            .insert(Enemy)
            .insert(Name::new(format!("Enemy Ship {i}")))
            .insert(AtLocation {id : id})
            .id();

    }
    let mut rnd = rand::thread_rng();
    let star_distr = rand::distributions::Uniform::new(0, stars.len());
    for i in 0..star_count {
        let links = rand::distributions::Uniform::new(1, 4).sample(&mut rnd);
        for _ in 0..links {
            let star_idx = star_distr.sample(&mut rnd);
            {
                let mut star_loc = state.world.get_mut::<Location>(stars[i]).unwrap();
                star_loc.paths.push(stars[star_idx]);
            }
            {
                let mut star_loc = state.world.get_mut::<Location>(stars[star_idx]).unwrap();
                star_loc.paths.push(stars[i]);
            }
        }
    }

    let mut planets = vec![];
    let mut items = vec![];
    //generate planets
    for star_idx in 0..star_count {
        let planet_count = rand::distributions::Uniform::new(1, 10).sample(&mut rnd);
        let star_id = stars[star_idx];
        let star_name = state.world.get::<Name>(star_id).unwrap().to_string();
        for planet_idx in 0..planet_count {

            let item_id = state.world.spawn(Item)
                .insert(Name::new(format!("Item {planet_idx} of {star_name}")))
                .id();
            items.push(item_id);


            let planet_id = state.world
                .spawn(Location {paths : vec![star_id]})
                .insert(Name::new(format!("Planet {planet_idx} of {star_name}")))
                .insert(HasItem {items : HashSet::from([item_id])})
                .id();
            planets.push(planet_id);
            state.world.get_mut::<Location>(star_id).unwrap().paths.push(planet_id);


        }
    }

    let ship_id = state.world.spawn(AtLocation {id : planets[0]})
        .insert(Ship)
        .insert(Player)
        .insert(Name::new("Ship"))
        .insert(HasItem {items : HashSet::default()})
        .id();

    let goal = Goal {
        pred : vec![Box::new(GoalLocation {target_loc : planets[planets.len() - 1], target_obj : ship_id}), 
            Box::new(GoalItem {target_owner : ship_id, target_obj : items[items.len() - 1]})],
        best_heter : RwLock::new(1000000)
    };
    goal

    // Goal {
    //     pred : vec![],
    //     best_heter : RwLock::new(1000000)
    // }
}