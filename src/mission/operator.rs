use std::fmt::Debug;

use bevy::prelude::*;
use bevy::utils::HashMap;
use super::state::State;
use super::{atom::*, FindNode};


pub trait OperatorRule {
    fn name(&self) -> String;
    fn can_effect(&self, state : &mut State) -> Vec<Box<dyn Operator>>;
    fn batch_effect(&self, state : &mut State) -> Vec<(FindNode, i32)> {
        let ops = self.can_effect(state);
        let mut res = vec![];
        for op in ops {
            let mut state = op.effect(state);
            state.setup_hash();
            res.push((FindNode { state, op : op.clone_operator() }, 1));
        }
        res
    }
}

pub trait Operator : Debug {
    fn to_string(&self) -> String;
    fn to_pretty(&self, world : &World) -> String;
    fn effect(&self, state : &mut State) -> State;
    fn clone_operator(&self) -> Box<dyn Operator + Send + Sync>;
}

pub struct GoRule {}

#[derive(Debug, Clone)]
pub struct Go {
    pub id : Entity,
    pub move_to : Entity
}

impl OperatorRule for GoRule {
    fn name(&self) -> String {
        "GoRule".to_string()
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
    fn to_string(&self) -> String {
        format!("Go({:?}, {:?})", self.id, self.move_to)
    }

    fn to_pretty(&self, world : &World) -> String {
        let none = Name::new("None");
        let obj_name = world.get::<Name>(self.id).unwrap_or(&none).to_string();
        let loc_name = world.get::<Name>(self.move_to).unwrap_or(&none).to_string();
        format!("Go {} to {}", obj_name, loc_name)
    }

    fn effect(&self, state : &mut State) -> State {
        let mut new_state = state.clone();
        if let Some(mut new_at_loc) = new_state.world.get_mut::<AtLocation>(self.id) {
            new_at_loc.id = self.move_to;
        }
        new_state
    }

    fn clone_operator(&self) -> Box<dyn Operator + Send + Sync> {
        Box::new(self.clone())
    }
}


pub struct TakeRule {}

impl OperatorRule for TakeRule {
    fn name(&self) -> String {
        "TakeRule".to_string()
    }

    fn can_effect(&self, state : &mut State) -> Vec<Box<dyn Operator>> {
        
        let mut query = state.world.query::<(Entity, &HasItem)>();

        let mut loc_item_map = HashMap::new();
        for (e, has) in query.iter(&state.world) {
            for item in has.items.iter() {
                loc_item_map.insert(e, *item);
            }
        }

        let mut res : Vec<Box<dyn Operator>> = vec![];
        let mut player_query = state.world.query::<(Entity, &AtLocation, &Ship)>();
        for (e, at_loc, _ship) in player_query.iter(&state.world) {
            if let Some(item) = loc_item_map.get(&at_loc.id) {
                res.push(Box::new(Take {
                    to : e,
                    from : at_loc.id,
                    item : *item
                }));

            }
        }

        // println!("{:?}", loc_item_map);

        res
    }
}

#[derive(Debug, Clone, Component, PartialEq, Eq, Hash)]
pub struct Take {
    pub to : Entity,
    pub from : Entity,
    pub item : Entity
}

impl Operator for Take {
    fn to_string(&self) -> String {
        format!("Take({:?}, {:?}, {:?})", self.from, self.to, self.item)
    }

    fn to_pretty(&self, world : &World) -> String {
        let none = Name::new("None");
        let from_name = world.get::<Name>(self.from).unwrap_or(&none).to_string();
        let to_name = world.get::<Name>(self.to).unwrap_or(&none).to_string();
        let item_name = world.get::<Name>(self.item).unwrap_or(&none).to_string();
        format!("Take {} from {} to {}", item_name, from_name, to_name)
    }

    fn effect(&self, state : &mut State) -> State {
        let mut new_state = state.clone();
        // println!("Take {:?} from {:?} to {:?}", self.item, self.from, self.to);
        
        if let Some(mut has_from) = new_state.world.get_mut::<HasItem>(self.from) {
            has_from.items.remove(&self.item);
        } 
        if let Some(mut has_to) = new_state.world.get_mut::<HasItem>(self.to) {
            has_to.items.insert(self.item);
        }
        new_state
    }

    fn clone_operator(&self) -> Box<dyn Operator + Send + Sync> {
        Box::new(self.clone())
    }
}