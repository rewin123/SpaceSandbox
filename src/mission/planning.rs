use super::state::State;

use super::operator::*;
use bevy::prelude::*;
use bevy::utils::Instant;

use std::hash::{Hash, Hasher};

use std::time::Duration;
use super::goal::Goal;
use std::thread;


#[derive(Debug)]
pub struct FindNode {
    pub state : State,
    pub op : Box<dyn Operator + Send + Sync>
}

impl FindNode {
    fn successors(&self) -> Vec<(FindNode, i32)> {
        
        self.state.clone().successors()
    }
}

impl Clone for FindNode {
    fn clone(&self) -> Self {
        FindNode {
            state : self.state.clone(),
            op : self.op.clone_operator()
        }
    }
}

impl Eq for FindNode {}

impl Hash for FindNode {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.state.hash(state);
    }
}

impl PartialEq for FindNode {
    fn eq(&self, other: &Self) -> bool {
        self.state == other.state
    }
}

#[derive(Debug)]
struct EmptyOp;

impl Operator for EmptyOp {
    fn to_string(&self) -> String {
        "EmptyOp".to_string()
    }

    fn to_pretty(&self, _world : &World) -> String {
        "Chilling".to_string()
    }

    fn effect(&self, state : &mut State) -> State {
        state.clone()
    }

    fn clone_operator(&self) -> Box<dyn Operator + Send + Sync> {
        Box::new(EmptyOp)
    }
}


pub fn find_sequence(s0 : &State, goal : &Goal) -> Option<Vec<Box<dyn Operator + Send + Sync>>> {

    let find_node = FindNode {
        state : s0.clone(),
        op : Box::new(EmptyOp)
    };

    let start_time = Instant::now();
    let goal = goal.clone();
    let find_thr = thread::spawn(move || {
        let start_node = find_node;
        
        pathfinding::prelude::fringe(
            &start_node, 
            |s| s.successors(), 
            |s| goal.heteruistic(&s.state),
            |s| goal.precondition(&s.state))
    });
    while !find_thr.is_finished() && start_time.elapsed() < Duration::from_secs(10) {
        thread::sleep(Duration::from_millis(1));
    }

    if find_thr.is_finished() {
        if let Some((res,_cost)) = find_thr.join().unwrap() {
            let mut op_vec = vec![];
            for node in res {
                op_vec.push(node.op);
            }
            return Some(op_vec);
        } else {
            return None;
        }
    }

    None
}

pub fn print_planning_plan(world : &World, plan : &Vec<Box<dyn Operator + Send + Sync>>) -> String {
    let mut res = String::new();

    for op in plan {
        res.push_str(&op.to_pretty(world));
        res.push_str(" \n");
    }
    res
}

