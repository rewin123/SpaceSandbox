use std::fmt::Debug;
use super::state::State;
use bevy::prelude::*;
use super::atom::*;

pub struct Goal {
    pub pred : Vec<Box<dyn GoalPred + Send + Sync>>
}

impl Goal {
    pub fn precondition(&self, state : &State) -> bool {
        for pred in self.pred.iter() {
            if !pred.precondition(state) {
                return false
            }
        }
        true
    }

    pub fn heteruistic(&self, state : &State) -> u64 {
        let mut res = 0;
        for pred in self.pred.iter() {
            res += pred.heteruistic(state);
        }
        res
    }
}

impl Clone for Goal {
    fn clone(&self) -> Self {
        Goal {
            pred : self.pred.iter().map(|x| x.clone_goal()).collect()
        }
    }
}

pub trait GoalPred : Debug {
    fn name(&self) -> String;
    fn precondition(&self, state : &State) -> bool;
    fn heteruistic(&self, state : &State) -> u64;
    fn clone_goal(&self) -> Box<dyn GoalPred + Send + Sync>;
}

#[derive(Debug, Clone)]
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

    fn clone_goal(&self) -> Box<dyn GoalPred + Send + Sync> {
        Box::new(self.clone())
    }

    fn heteruistic(&self, state : &State) -> u64 {
        10
    }
}

#[derive(Debug, Clone)]
pub struct GoalItem {
    pub target_owner : Entity,
    pub target_obj : Entity
}

impl GoalPred for GoalItem {
    fn name(&self) -> String {
        "GoalItem".to_string()
    }

    fn precondition(&self, state : &State) -> bool {
        if let Some(has) = state.world.get::<HasItem>(self.target_owner) {
            return has.items.contains(&self.target_obj);
        }
        false
    }

    fn clone_goal(&self) -> Box<dyn GoalPred + Send + Sync> {
        Box::new(self.clone())
    }

    fn heteruistic(&self, state : &State) -> u64 {
        10
    }
}