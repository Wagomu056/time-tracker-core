use std::collections::HashMap;
use std::time::{Duration, SystemTime};

struct Task {
    #[allow(dead_code)] //@TODO: name will be used in the future as save the task name
    name: String,
    start_time: SystemTime,
    end_time: SystemTime,
}

pub struct TimeTracer {
    current_id: u32,
    tasks_map: HashMap<u32, Task>,
    running_tasks: Vec<u32>,
}

impl TimeTracer {
    pub fn new() -> TimeTracer {
        TimeTracer {
            current_id: 0,
            tasks_map: HashMap::new(),
            running_tasks: Vec::new(),
        }
    }

    pub fn new_task(&mut self, name: &str) -> u32 {
        let task = Task {
            name: name.to_string(),
            start_time: SystemTime::now(),
            end_time: SystemTime::now(),
        };

        let id = self.current_id;
        self.tasks_map.insert(self.current_id, task);
        self.current_id += 1;
        id
    }

    pub fn get_task_number(&self) -> u32 {
        self.tasks_map.len() as u32
    }

    pub fn start_task(&mut self, id: u32) -> bool {
        if self.running_tasks.contains(&id) {
            return false;
        }

        let task = self.tasks_map.get_mut(&id);
        match task {
            None => false,
            Some(task) => {
                task.start_time = SystemTime::now();
                self.running_tasks.push(id);
                true
            }
        }
    }

    pub fn end_task(&mut self, id: u32) -> Option<Duration> {
        if !self.running_tasks.contains(&id) {
            return None;
        }

        let task = self.tasks_map.get_mut(&id);
        match task {
            None => None,
            Some(task) => {
                task.end_time = SystemTime::now();
                self.running_tasks.retain(|&x| x != id);
                Some(task.end_time.duration_since(task.start_time).expect("Time went backwards"))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn if_new_task_then_task_number_increases() {
        let mut tracer = TimeTracer::new();
        assert_eq!(tracer.get_task_number(), 0);

        tracer.new_task("task1");
        assert_eq!(tracer.get_task_number(), 1);
    }

    #[test]
    fn if_start_task_then_return_true() {
        let mut tracer = TimeTracer::new();
        let id = tracer.new_task("task1");
        assert_eq!(tracer.start_task(id), true);
    }

    #[test]
    fn if_end_task_then_return_duration() {
        let mut tracer = TimeTracer::new();
        let id = tracer.new_task("task1");
        tracer.start_task(id);

        // Sleep for 0.5 seconds and check if the duration is greater than 0.5 seconds
        std::thread::sleep(Duration::from_millis(500));
        let duration = tracer.end_task(id).unwrap();
        assert!(duration.subsec_millis() >= 500);
    }

    #[test]
    fn if_start_task_twice_then_return_false() {
        let mut tracer = TimeTracer::new();
        let id = tracer.new_task("task1");
        tracer.start_task(id);
        assert_eq!(tracer.start_task(id), false);
    }

    #[test]
    fn if_no_start_and_end_task_then_return_none() {
        let mut tracer = TimeTracer::new();
        let id = tracer.new_task("task1");
        assert_eq!(tracer.end_task(id), None);
    }
}
