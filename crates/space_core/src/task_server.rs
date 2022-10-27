use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};

#[derive(Copy, Clone)]
pub enum TaskState {
    Created,
    Running,
    Finished
}

pub struct Task {
    idx : usize,
    name : String,
    state : RwLock<TaskState>
}

impl Task {
    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    pub fn get_state(&self) -> TaskState {
        *self.state.read().unwrap()
    }

    pub fn set_state(&self, state : TaskState) {
        *self.state.write().unwrap() = state;
    }
}

pub struct TaskServer {
    tasks : Arc<Mutex<HashMap<usize, Arc<Task>>>>,
    index : Mutex<usize>
}

impl TaskServer {
    pub fn new() -> Self {
        Self {
            tasks : Arc::new(Mutex::new(HashMap::new())),
            index : Mutex::new(0)
        }
    }

    pub fn spawn<F : FnOnce() + Send + 'static + Sync>(&self, name : &String, f : F) {
        let mut global_index = self.index.lock().unwrap();
        let task = Arc::new(Task {
          idx : *global_index,
          name : name.clone(),
            state : RwLock::new(TaskState::Created)
        });


        self.tasks.lock().unwrap().insert(task.idx, task.clone());
        *global_index += 1;

        let cloned_task = task.clone();
        let cloned_hash_map = self.tasks.clone();
        rayon::spawn(move || {
            cloned_task.set_state(TaskState::Running);
           f();
            cloned_task.set_state(TaskState::Finished);
            cloned_hash_map.lock().unwrap().remove(&cloned_task.idx);
        });
    }

    pub fn get_task_count(&self) -> usize {
        self.tasks.lock().unwrap().len()
    }

    pub fn clone_task_list(&self) -> Vec<Arc<Task>> {
        let iter = self.tasks.lock().unwrap().iter().map(|(_, v)| v).cloned().collect();
        iter
    }
}