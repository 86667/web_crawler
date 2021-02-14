use crate::{task::Task, worker::Worker};

use std::thread;
use std::sync::mpsc;
use std::time::Duration;

/// Scheduler receives a Task and performes some function on it. The data returned, if any, is
/// a new task which is ran in the same way. This repeats until all Tasks have been complete.
pub struct Scheduler {
    tasks: Vec<Task>,
    loop_delay: u64,
    test: bool
}

impl Scheduler {
    pub fn new(tasks: Vec<Task>, loop_delay: u64, test: bool) -> Self {
        Scheduler{ tasks, loop_delay, test }
    }

    pub fn get_tasks(self) -> Vec<Task> {
        self.tasks
    }

    pub async fn run_tasks(&mut self) {
        // Multi-channel to receive back results from threads before they close
        let (tx, rx) = mpsc::channel();

        let mut go = true;
        while go {
            thread::sleep(Duration::from_millis(self.loop_delay));

            // Chcek receive channel for data
            let data_iter: mpsc::TryIter<(Task, Vec<Task>)> = rx.try_iter();

            // Handle receive data
            for data in data_iter {
                self.handle_received_data(data);
            }

            // Assign new tasks or check if all complete. End if all complete.
            if self.tasks.iter().find(|x| x.status==0).is_some() {
                self.start_new_tasks(tx.clone());
            } else {
                if self.tasks.iter().find(|x| x.status!=2).is_none() {
                    go = false;
                }
            }
        }
    }

    // Loop through new tasks and add to queue
    fn handle_received_data(&mut self, data: (Task, Vec<Task>)) {
        // println!("\n\nData returned. Process: {:?}", data);
        let (task_complete, mut tasks_todo) = data;

        // Loop through tasks and:
        //      -Mark completed task as complete (status=2)
        //      -if task in tasks_todo exists, remove it from tasks_todo
        for task in &mut self.tasks {
            if task.sub_domain == task_complete.sub_domain {
                task.status=2
            };
            tasks_todo.retain(|task_todo| task.sub_domain!=task_todo.sub_domain);
        }

        // Add new tasks
        self.tasks.append(&mut tasks_todo);
    }

    fn start_new_tasks(&mut self, tx: mpsc::Sender<(Task, Vec<Task>)>) {
        // Assign new tasks to threads
        for task in &mut self.tasks {
            if task.status==0 {
                spawn_worker(task.clone(), tx.clone(), self.test);
                task.status=1;
                // println!("Started new task: {:?}", task);
            }
        }
        // println!("\nTasks list: {:?}",self.tasks);
    }

}


// Return task that was input along with vector of new tasks.
// New tasks are added if the number of chars in the original task +1 is even.
fn spawn_worker(task: Task, tx: mpsc::Sender<(Task, Vec<Task>)>, test: bool) {
    thread::spawn(move ||  {
        let worker = Worker::new(task, tx, test);
        worker.run_task();
    });
}


#[cfg(test)]
mod tests {
    use super::*;
    use tokio_test;

    macro_rules! aw {
      ($e:expr) => {
          tokio_test::block_on($e)
      };
    }

    #[test]
    fn test_dup_tasks() {
        // "a" will return a test "aa" which should be ignored becasue it is already in list
        let tasks = vec!(
            Task::new("domain".to_string(), "a".to_string()),
            Task::new("domain".to_string(), "aa".to_string())
        );
        let mut scheduler = Scheduler::new(tasks, 50, true);
        aw!(scheduler.run_tasks());

        // Expect the original tasks to be returned only
        assert_eq!(scheduler.tasks.len(), 2);
        // Expect all tasks to be marked complete
        assert!(scheduler.tasks.iter().find(|task| task.status!=2).is_none());
    }

    #[test]
    fn test_no_dups() {
        // Since "a" and "ccc" have odd number of characters we expect "aa" and "cccc" to be completed
        // along with all other tasks
        let tasks = vec!(
            Task::new("domain".to_string(), "a".to_string()),
            Task::new("domain".to_string(), "bb".to_string()),
            Task::new("domain".to_string(), "ccc".to_string()),
            Task::new("domain".to_string(), "dddd".to_string()));

        let mut scheduler = Scheduler::new(tasks, 50, true);
        aw!(scheduler.run_tasks());

        // Expect 6 tasks to be returned
        assert_eq!(scheduler.tasks.len(), 6);
        // Expect all tasks to be marked complete
        assert!(scheduler.tasks.iter().find(|task| task.status!=2).is_none());


        // All odd number of chars so expect double the number of tasks back
        let tasks = vec!(
            Task::new("domain".to_string(), "a".to_string()),
            Task::new("domain".to_string(), "b".to_string()),
            Task::new("domain".to_string(), "c".to_string()),
            Task::new("domain".to_string(), "d".to_string()));
        let mut scheduler = Scheduler::new(tasks, 50, true);
        aw!(scheduler.run_tasks());
        assert_eq!(scheduler.tasks.len(), 8);
        assert!(scheduler.tasks.iter().find(|task| task.status!=2).is_none());

        // All even number of chars so expect same number of tasks back
        let tasks = vec!(
            Task::new("domain".to_string(), "aa".to_string()),
            Task::new("domain".to_string(), "bb".to_string()),
            Task::new("domain".to_string(), "cc".to_string()),
            Task::new("domain".to_string(), "dd".to_string()));
        let mut scheduler = Scheduler::new(tasks, 50, true);
        aw!(scheduler.run_tasks());
        assert_eq!(scheduler.tasks.len(), 4);
        assert!(scheduler.tasks.iter().find(|task| task.status!=2).is_none());
    }

    #[test]
    fn test_empty_task() {
        let tasks = vec!(
            Task::new("domain".to_string(), "".to_string()));
        let mut scheduler = Scheduler::new(tasks, 50, true);
        aw!(scheduler.run_tasks());
        assert_eq!(scheduler.tasks.len(), 1);
        assert!(scheduler.tasks.iter().find(|task| task.status!=2).is_none());
    }

    #[test]
    fn test_no_loop_delay() {
        let tasks = vec!(
            Task::new("domain".to_string(), "aa".to_string()),
            Task::new("domain".to_string(), "bb".to_string()),
            Task::new("domain".to_string(), "cc".to_string()),
            Task::new("domain".to_string(), "dd".to_string()));
        let mut scheduler = Scheduler::new(tasks, 0, true);
        aw!(scheduler.run_tasks());
        assert_eq!(scheduler.tasks.len(), 4);
        assert!(scheduler.tasks.iter().find(|task| task.status!=2).is_none());
    }
}
