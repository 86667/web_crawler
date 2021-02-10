use std::thread;
use std::sync::mpsc;
use std::sync::mpsc::TryRecvError;
use std::time::Duration;

// Task is a single "task" item of data to pass to a worker.
// Scheduler decides which worker type to assign based on Task's type.
#[derive(Clone, Debug, PartialEq)]
pub enum Task {
    // TaskWebCrawl(String),
    TaskTester(usize)
}


pub struct Scheduler {
    tasks: Vec<Task>,
    awaiting: Vec<Task>,
    completed: Vec<Task>
}
impl Scheduler {
    pub fn new(tasks: Vec<Task>) -> Self {
        Scheduler{ tasks, awaiting: vec!(), completed: vec!() }
    }

    pub async fn run_tasks(mut self) -> Vec<Task> {
        let (tx, rx) = mpsc::channel();

        let mut go = true;
        while go {
            // thread::sleep(Duration::from_secs(1));
            thread::sleep(Duration::from_millis(500));
            // println!("\nstart.");

            let data: Result<(Task, Vec<Task>), TryRecvError> = rx.try_recv();

            // Handle receive data
            match data {
                Ok(return_data) => {
                    println!("data returned: {:?}", return_data);
                    let (task_complete, mut task_todo) = return_data;
                    self.awaiting.retain(|x| *x != task_complete); // rm all instances of x

                    self.tasks.append(&mut task_todo);    // add list of new tasks to queue
                    self.completed.push(task_complete);   // add task value to list of completed
                    println!("tasks: {:?}",self.tasks);
                    println!("awaiting: {:?}",self.awaiting);
                    println!("completed: {:?}",self.completed);
                },
                Err(_) => ()
            }

            // Assign new task or check if all complete. End if all complete.
            if self.tasks.len()!=0 {
                let task = self.tasks.pop().unwrap();
                self.awaiting.push(task.clone());
                self.spawn_worker(task.clone(), tx.clone());
                println!("Started new task: {:?}", task);
            } else {
                if self.awaiting.len() == 0 {
                    go = false;
                }
            }
            // println!("end.");
        }

        println!("tasks: {:?}",self.tasks);
        println!("awaiting: {:?}",self.awaiting);
        println!("completed: {:?}",self.completed);

        return self.completed
    }

    fn spawn_worker(&self, task: Task, tx: mpsc::Sender<(Task, Vec<Task>)>) -> std::thread::JoinHandle<()> {
        thread::spawn(move || {
            match task {
                Task::TaskTester(i) => test_worker(i, tx),
                _ => {}
            }

        })
    }
}


fn test_worker(i: usize, tx: mpsc::Sender<(Task, Vec<Task>)>) {
    thread::sleep(Duration::from_secs(i as u64));
    let new_task = i+1;
    if new_task%2==0 {
        tx.send((Task::TaskTester(i),vec!(Task::TaskTester(new_task)))).unwrap();
    } else {
        tx.send((Task::TaskTester(i),vec!())).unwrap();
    }
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
    fn test_run() {
        let tasks = vec!(Task::TaskTester(1),Task::TaskTester(2),Task::TaskTester(3),Task::TaskTester(4));
        let scheduler = Scheduler::new(tasks);

        let completed = aw!(scheduler.run_tasks());
        assert_eq!(completed.len(), 6);
    }
}
