use std::thread;
use std::sync::mpsc;
use std::sync::mpsc::TryRecvError;
use std::time::Duration;
use std::iter::repeat;
use rand;

// pub struct TaskTester{ i: usize }

#[derive(Clone, Debug, PartialEq)]
pub struct Task {
    item: String,
    status: usize
}
impl Task {
    pub fn new(item: String) -> Self { Self{item, status: 0} }
}

// Task is a single "task" item of data to pass to a worker.
// Scheduler decides which worker type to assign based on Task's type.
// #[derive(Clone, Debug, PartialEq)]
// pub enum Task {
//     // TaskWebCrawl(String),
//     TaskTester(usize, usize)
// }

// impl TaskTester { fn new(i: usize) -> Self { Self{i} } }

pub struct Scheduler {
    tasks: Vec<Task>,
}
impl Scheduler {
    pub fn new(tasks: Vec<Task>) -> Self {
        Scheduler{ tasks }
    }

    pub async fn run_tasks(&mut self) {
        let (tx, rx) = mpsc::channel();

        let mut go = true;
        while go {

            thread::sleep(Duration::from_millis(50));
            println!("\nstart.");

            let data_iter: mpsc::TryIter<(Task, Vec<Task>)> = rx.try_iter();
            println!("data_iter: {:?}",data_iter);
            // Handle receive data
            for data in data_iter {
                println!("data returned: {:?}", data);
                let (task_complete, mut tasks_todo) = data;
                // Loop through tasks and:
                //      -Mark completed task as complete (status=2)
                //      -if task in tasks_todo exists, remove it from tasks_todo
                for task in &mut self.tasks {
                    if task.item == task_complete.item {
                        task.status=2
                    };
                    tasks_todo.retain(|task_todo| task.item!=task_todo.item);
                }

                // Add new tasks
                self.tasks.append(&mut tasks_todo);

                 // rm all instances of x
                println!("tasks: {:?}",self.tasks);
            }


            // Assign new tasks or check if all complete. End if all complete.
            if self.tasks.iter().find(|x| x.status==0).is_some() {
                // Assign new tasks to threads
                for mut task in &mut self.tasks {
                    if task.status==0 {
                        self.spawn_worker(task.clone(), tx.clone());
                        task.status=1;
                    }
                    println!("Started new task: {:?}", task);
                }
                println!("tasks: {:?}",self.tasks);
            } else {
                if self.tasks.iter().find(|x| x.status!=2).is_some() {
                    go = false;
                }
            }
            println!("end.");
        }

        println!("tasks: {:?}",self.tasks);

    }

    fn spawn_worker(self, task: Task, tx: mpsc::Sender<(Task, Vec<Task>)>) -> std::thread::JoinHandle<()> {
        thread::spawn(move || {
            test_worker(task, tx)
        })
    }
}

// Return task that was input along with vector of new tasks.
// New tasks are added if the number of chars in the original task +1 is even.
// Then return a new task which is the same as the original but with an extra character.
fn test_worker(task: Task, tx: mpsc::Sender<(Task, Vec<Task>)>) {
    let mut num_chars: usize = task.item.len();
    thread::sleep(Duration::from_millis(num_chars as u64));
    num_chars+=1;
    if num_chars%2==0 {
        let new_item = repeat(task.clone().item.chars().nth(0).unwrap()).take(num_chars).collect::<String>();
        tx.send((task.to_owned(),vec!(Task::new(new_item)))).unwrap();
    } else {
        tx.send((task,vec!())).unwrap();
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
        let tasks = vec!(
            Task::new("a".to_string()),
            Task::new("aa".to_string()),
            Task::new("aaa".to_string()),
            Task::new("aaaa".to_string()));

        let mut scheduler = Scheduler::new(tasks);

        aw!(scheduler.run_tasks());
        println!("scheduler.tasks: {:?}", scheduler.tasks);
        // assert_eq!(completed.len(), 6);
    }
}
