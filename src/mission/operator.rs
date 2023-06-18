use std::fmt::Debug;

use bevy::prelude::*;
use super::state::State;
use super::{atom::*, FindNode};


pub trait OperatorRule {
    fn name(&self) -> String;
    fn can_effect(&self, state : &mut State) -> Vec<Box<dyn Operator>>;
    fn batch_effect(&self, state : &mut State) -> Vec<(FindNode, i32)> {
        let mut ops = self.can_effect(state);
        let mut res = vec![];
        for op in ops {
            res.push((FindNode { state : op.effect(state), op : op.clone_operator() }, 1));
        }
        res
    }
}

pub trait Operator : Debug {
    fn to_string(&self) -> String;
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
