use crate::task::Task;

use regex::Regex;
use std::sync::mpsc;
use std::fs;

use std::thread;
use std::time::Duration;
use std::iter::repeat;

use rand;
use rand::thread_rng;
use rand::Rng;

use reqwest;

const REGEX: &str = r#"^(?:https?://)?(?:w{3}.)?/?(.*)$"#;

/// Worker pulls a given webpage and returns all instances of a provided REGEX.
pub struct Worker {
    regex: Regex,
    task: Task,
    tx: mpsc::Sender<(Task, Vec<Task>)>,
    test: bool
}

impl Worker {
    pub fn new(task: Task, tx: mpsc::Sender<(Task, Vec<Task>)>, test: bool) -> Self {
        // insert domain into REGEX capture string
        let mut regex = String::from(REGEX);
        regex.insert_str(25, &task.item);

        Worker{
            regex: Regex::new(&regex).unwrap(),
            task,
            tx,
            test
        }
    }

    pub fn run_task(self) {
        match self.test {
            false => {
                // Fetch website
                let res = self.get_html(&self.task.item);
                match res {
                    Ok(html) => {
                        // Pull out links
                        let return_tasks = self.search_for_links(html);
                        // Return to main thread
                        self.tx.send((self.task,return_tasks)).unwrap();
                    },
                    Err(_) => {
                        self.tx.send((self.task,vec!())).unwrap();
                    }
                }
            },
            true => {
                // Run testing version
                test_worker(self.task.clone(), self.tx);
            }
        }
    }

    /// Basic GET request
    fn get_html(&self, url: &str) -> Result<String, Box<dyn std::error::Error>> {
        let resp = match reqwest::blocking::get(url) {
            Ok(resp) => resp,
            Err(e) => {
                if e.is_builder() {
                    let mut url_with_base = String::from("http://");
                    url_with_base.insert_str(7, url);
                    reqwest::blocking::get(&url_with_base).unwrap();
                }
                return Err(Box::new(e))
            }
        };
        Ok(resp.text().unwrap())
    }

    // fn get_html(&self, url: &str) -> Result<String, std::io::Error> {
    //     fs::read_to_string("test_data/test_5_instances.txt")
    // }


    /// Iterate through string pulling out each instance captured by this.regex
    pub fn search_for_links(&self, html: String) -> Vec<Task>{
        html.split_whitespace()
            .filter(|item| self.is_link_regex(&item).is_some())
            .map(|item| Task::new(item.to_owned()))
            .collect()
    }

    // Apply capture REGEX
    pub fn is_link_regex(&self, is_link_regex: &str) -> Option<String> {
        let res = self.regex.captures(is_link_regex)?;
        match res.get(1) {
            Some(item) => return Some(item.as_str().to_string()),
            None => return None
        }
    }
}

// Tester worker simulates html-grabber by:
//  Count number of chars in task item and +1. If even return new task with chars+1 number of characters.
fn test_worker(task: Task, tx: mpsc::Sender<(Task, Vec<Task>)>) {
    thread::sleep(Duration::from_millis(thread_rng().gen_range(0..1000) as u64));
    let mut num_chars: usize = task.item.len();
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

    fn spawn_worker() -> Worker {
        let (tx, _) = mpsc::channel();
        let task = Task::new("monzo.com".to_string());
        Worker::new(task, tx, true)
    }

    #[test]
    fn test_regex_gen() {
        let (tx, _) = mpsc::channel();
        let task = Task::new("domain.com".to_string());
        let worker = Worker::new(task, tx, true);
        assert_eq!(worker.regex.to_string(), "^(?:https?://)?(?:w{3}.)?domain.com/?(.*)$");
    }

    #[test]
    fn test_regex() {
        let worker = spawn_worker();
        assert_eq!(worker.is_link_regex("monzo.com/path"), Some("path".to_string()));
        assert_eq!(worker.is_link_regex("monzo.com/monzo.com"), Some("monzo.com".to_string()));
        assert_eq!(worker.is_link_regex("monzo.com//path"), Some("/path".to_string()));
        assert_eq!(worker.is_link_regex("www.monzo.com/path"), Some("path".to_string()));
        assert_eq!(worker.is_link_regex("http://www.monzo.com/path"), Some("path".to_string()));
        assert_eq!(worker.is_link_regex("https://www.monzo.com/path"), Some("path".to_string()));
        assert_eq!(worker.is_link_regex("http://monzo.com/path"), Some("path".to_string()));
        assert_eq!(worker.is_link_regex("https://monzo.com/path"), Some("path".to_string()));

        assert_eq!(worker.is_link_regex("ww.monzo.com/path"), None);
        assert_eq!(worker.is_link_regex(".monzo.com/path"), None);
        assert_eq!(worker.is_link_regex("wwwmonzo.com/path"), None);
        assert_eq!(worker.is_link_regex("monzo.cm/path"), None);
        assert_eq!(worker.is_link_regex("monzo.cm/path"), None);
        assert_eq!(worker.is_link_regex("monzo.cpm//path"), None);
    }

    #[test]
    fn test_search_for_links() {
        let worker = spawn_worker();
        let links = worker.search_for_links(
            fs::read_to_string("test_data/test_5_instances.txt").unwrap()
        );
        assert_eq!(links.len(), 5);
    }
}
