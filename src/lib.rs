use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::time::{Duration, SystemTime};

struct Task {
    id: u32,
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
            id: self.current_id,
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

    fn write_to_file(&self, task: &Task, file_path: &str) -> Result<(), std::io::Error> {
        let mut file = File::options()
            .append(true)
            .create(true)
            .open(file_path)?;

        let start_time = task.start_time.duration_since(SystemTime::UNIX_EPOCH).expect("Time went backwards");
        let end_time = task.end_time.duration_since(SystemTime::UNIX_EPOCH).expect("Time went backwards");
        writeln!(&mut file, "{},{},{},{}", task.id, task.name, start_time.as_secs(), end_time.as_secs())?;
        Ok(())
    }
}

#[cfg(test)]
use regex::Regex;

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

    fn create_new_task_and_write_to_file(tracer: &mut TimeTracer, file_path: &str, task_name: &str) -> Result<(), std::io::Error> {
        let id = tracer.new_task(task_name);
        tracer.start_task(id);
        std::thread::sleep(Duration::from_millis(500));
        tracer.end_task(id);
        let task = tracer.tasks_map.get(&id).unwrap();
        tracer.write_to_file(task, file_path)
    }

    #[test]
    fn private_if_file_is_not_exist_then_create_file() {
        let file_path = "test_work/test_save.txt";

        // if already exists, delete the save file
        if std::path::Path::new(file_path).exists() {
            std::fs::remove_file(file_path).unwrap();
        }

        let mut tracer = TimeTracer::new();

        // create a new task and write it to the file
        assert!(create_new_task_and_write_to_file(&mut tracer, file_path, "task1").is_ok());

        // check if the file is created
        assert!(std::path::Path::new(file_path).exists());

        // check content of the file using regex
        let file_content = std::fs::read_to_string(file_path).unwrap();
        let re = Regex::new(r"0,task1,\d+,\d+").unwrap();
        assert!(re.is_match(&file_content));
    }

    #[test]
    fn private_if_write_twice_then_append_to_file() {
        let file_path = "test_work/test_save_twice.txt";

        if std::path::Path::new(file_path).exists() {
            std::fs::remove_file(file_path).unwrap();
        }

        let mut tracer = TimeTracer::new();

        // create a new task and write it to the file
        assert!(create_new_task_and_write_to_file(&mut tracer, file_path, "task1").is_ok());

        // create a new task and write it again
        assert!(create_new_task_and_write_to_file(&mut tracer, file_path, "task2").is_ok());

        // check line number of the file
        let file_content = std::fs::read_to_string(file_path).unwrap();
        assert_eq!(file_content.lines().count(), 2);

        // check last line
        let last_line = file_content.lines().last().unwrap();
        let re = Regex::new(r"1,task2,\d+,\d+").unwrap();
        assert!(re.is_match(last_line));
    }
}
