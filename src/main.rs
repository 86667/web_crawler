use std::collections::HashMap;
use std::fs;
use regex::Regex;

use std::thread;
use std::sync::mpsc;
use std::sync::mpsc::TryRecvError;
use std::time::Duration;
// use rayon::prelude::*;

#[tokio::main]
async fn main() {
    // let domain = "https://monzo.com/";
    // let html = get_html(domain);
    // let html = fs::read_to_string("test_data/monzo.txt").unwrap();
    // search_for_links(&html)

    let scheduler = Scheduler::new();
    scheduler.run_workers().await;
}

const REGEX: &str = r"^(?:.*)?monzo.com/(.*)$";

async fn get_html(url: &str) -> Result<String, Box<dyn std::error::Error>> {
    let resp = reqwest::get(url)
        .await?;
    Ok(resp.text().await?)
}

struct Scheduler {
    tasks: Vec<usize>,
    awaiting: Vec<usize>,
    completed: Vec<usize>
}
impl Scheduler {
    fn new() -> Self {
        Scheduler{
            tasks: vec!(1,2,3,4),
            awaiting: vec!(),
            completed: vec!(),
        }
    }

    async fn run_workers(mut self) {
        let (tx, rx) = mpsc::channel();

        let mut go = true;
        while go {
            thread::sleep(Duration::from_secs(1));
            println!("\nstart.");

            let data: Result<(usize, usize), TryRecvError> = rx.try_recv();

            // Handle receive data
            match data {
                Ok(return_data) => {
                    println!("data returned: {:?}", return_data);
                    let (task_complete, task_todo) = return_data;
                    self.awaiting.retain(|x| *x != task_complete); // rm all instances of x
                    if task_todo%2==0 {      // add return value to todos if even
                        self.tasks.push(task_todo);
                        println!("Added new task: {}", task_todo);
                    }
                    self.completed.push(task_complete);   // add task value to list of completed
                },
                Err(_) => ()
            }

            // Assign new task or check if all complete. End if all complete.
            if self.tasks.len()!=0 {
                let task = self.tasks.pop().unwrap();
                self.awaiting.push(task);
                worker(task.clone(), tx.clone());
                println!("Started new task: {}", task);
            } else {
                if self.awaiting.len() == 0 {
                    go = false;
                }
            }
            println!("end.");
        }

        println!("tasks: {:?}",self.tasks);
        println!("awaiting: {:?}",self.awaiting);
        println!("completed: {:?}",self.completed);
    }
}


fn worker(i: usize, tx: mpsc::Sender<(usize, usize)>) -> std::thread::JoinHandle<()> {
    thread::spawn(move || {
        thread::sleep(Duration::from_secs(i as u64));
        tx.send((i,i+1)).unwrap();
    })
}

pub struct Worker {
    // regex: String
}

impl Worker {
    fn run() -> Self {
        // Worker{regex: REGEX.to_string()}
        Worker{}
    }

    pub fn search_for_links(&self, html: &String) {
        let arr = html.split_whitespace()
            // .inspect(|x| println!("x {:?}", x))
            // .fold(vec!(), |mut acc, item| if item.contains("monzo.com") { acc.push(item); acc } else {acc});
            .inspect(|item| {let _ = self.is_link_regex(item);})
            .collect::<Vec<_>>();
            // println!("arr: {:?}", arr);
    }

    pub fn is_link_regex(&self, is_link_regex: &str) -> Option<String> {
        let re = Regex::new(REGEX).unwrap();
        let res = re.captures(is_link_regex)?;
        match res.get(1) {
            Some(item) => return Some(item.as_str().to_string()),
            None => return None
        }
    }
}


// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[test]
//     fn test_regex() {
//         assert_eq!(is_link_regex("monzo.com/sdfhrl"), Some("sdfhrl"));
//         assert_eq!(is_link_regex("www.monzo.com/sdfhrl"), Some("sdfhrl"));
//         assert_eq!(is_link_regex("http://www.monzo.com/sdfhrl"), Some("sdfhrl"));
//         assert_eq!(is_link_regex("https://www.monzo.com/sdfhrl"), Some("sdfhrl"));
//         assert_eq!(is_link_regex("ww.monzo.com/sdfhrl"), Some("sdfhrl"));
//         assert_eq!(is_link_regex("wwwmonzo.com/sdfhrl"), Some("sdfhrl"));
//         assert_eq!(is_link_regex("monzo.cm/sdfhrl"), None);
//         assert_eq!(is_link_regex("monzo.cm//sdfhrl"), None);
//     }
//
// }
