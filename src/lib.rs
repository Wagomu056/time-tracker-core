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
    save_file_path: String,
}

impl TimeTracer {
    pub fn new() -> TimeTracer {
        Self::new_with_file_path("time_tracer_save")
    }

    fn new_with_file_path(save_file_path: &str) -> TimeTracer {
        TimeTracer {
            current_id: 0,
            tasks_map: HashMap::new(),
            running_tasks: Vec::new(),
            save_file_path: save_file_path.to_string(),
        }
    }

    pub fn delete_save_file(&self) -> Result<(), std::io::Error> {
        if !std::path::Path::new(&self.save_file_path).exists() {
            return Ok(());
        }
        std::fs::remove_file(&self.save_file_path)
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
                let duration = Some(task.end_time.duration_since(task.start_time).expect("Time went backwards"));

                let write_result = Self::write_to_file(task, &self.save_file_path);
                if write_result.is_err() {
                    eprintln!("Failed to write to file: {}", write_result.err().unwrap());
                }
                duration
            }
        }
    }

    fn write_to_file(task: &Task, file_path: &str) -> Result<(), std::io::Error> {
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

    fn create_task_and_start_end(tracer: &mut TimeTracer, task_name: &str) -> bool {
        let id = tracer.new_task(task_name);
        let start_result = tracer.start_task(id);
        if !start_result {
            return false;
        }

        std::thread::sleep(Duration::from_millis(500));
        let end_result = tracer.end_task(id);
        end_result.is_some()
    }

    #[test]
    fn if_new_task_then_task_number_increases() {
        let mut tracer = TimeTracer::new_with_file_path("test_work/tmp.txt");
        assert_eq!(tracer.get_task_number(), 0);

        tracer.new_task("task1");
        assert_eq!(tracer.get_task_number(), 1);
    }

    #[test]
    fn if_start_task_then_return_true() {
        let mut tracer = TimeTracer::new_with_file_path("test_work/tmp.txt");
        let id = tracer.new_task("task1");
        assert_eq!(tracer.start_task(id), true);
    }

    #[test]
    fn if_end_task_then_return_duration() {
        let mut tracer = TimeTracer::new_with_file_path("test_work/tmp.txt");
        let id = tracer.new_task("task1");
        tracer.start_task(id);

        // Sleep for 0.5 seconds and check if the duration is greater than 0.5 seconds
        std::thread::sleep(Duration::from_millis(500));
        let duration = tracer.end_task(id).unwrap();
        assert!(duration.subsec_millis() >= 500);
    }

    #[test]
    fn if_start_task_twice_then_return_false() {
        let mut tracer = TimeTracer::new_with_file_path("test_work/tmp.txt");
        let id = tracer.new_task("task1");
        tracer.start_task(id);
        assert_eq!(tracer.start_task(id), false);
    }

    #[test]
    fn if_no_start_and_end_task_then_return_none() {
        let mut tracer = TimeTracer::new_with_file_path("test_work/tmp.txt");
        let id = tracer.new_task("task1");
        assert_eq!(tracer.end_task(id), None);
    }

    #[test]
    fn if_task_is_started_and_ended_then_write_to_file() {
        let file_path = "test_work/test_save.txt";

        let mut tracer = TimeTracer::new_with_file_path(file_path);
        assert!(tracer.delete_save_file().is_ok());

        // create a new task and write it to the file
        assert!(create_task_and_start_end(&mut tracer, "task1"));

        // check if the file is created
        assert!(std::path::Path::new(file_path).exists());

        // check content of the file using regex
        let file_content = std::fs::read_to_string(file_path).unwrap();
        let re = Regex::new(r"0,task1,\d+,\d+").unwrap();
        assert!(re.is_match(&file_content));
    }

    #[test]
    fn if_task_is_started_and_ended_twice_then_append_to_file() {
        let file_path = "test_work/test_save_twice.txt";

        let mut tracer = TimeTracer::new_with_file_path(file_path);
        assert!(tracer.delete_save_file().is_ok());

        // create a new task and write it to the file
        assert!(create_task_and_start_end(&mut tracer, "task1"));

        // create a new task and write it again
        assert!(create_task_and_start_end(&mut tracer, "task2"));

        // check line number of the file
        let file_content = std::fs::read_to_string(file_path).unwrap();
        assert_eq!(file_content.lines().count(), 2);

        // check last line
        let last_line = file_content.lines().last().unwrap();
        let re = Regex::new(r"1,task2,\d+,\d+").unwrap();
        assert!(re.is_match(last_line));
    }
}
