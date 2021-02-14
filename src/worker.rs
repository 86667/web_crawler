use crate::task::Task;

use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use std::iter::repeat;
#[allow(unused_imports)]
use std::fs;

use rand;
use rand::thread_rng;
use rand::Rng;

use regex::Regex;
use reqwest;

// REGEX captures paths that follow a domain URL
const REGEX: &str = r####"^.*(?:https?://)?(?:w{3}.)?/(.*)[\\"'](?:[>])?(?:\n)?.*$"####;

/// Worker pulls a given webpage and returns all instances of a provided REGEX
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
        regex.insert_str(27, &task.domain);

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
                let res = self.get_html(&self.task.get_url());
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
                    // Add Url Base if necessary
                    let mut url_with_base = String::from("https://");
                    url_with_base.insert_str(7, url);
                    let resp = reqwest::blocking::get(&url_with_base).unwrap();
                    return Ok(resp.text().unwrap())
                } else {
                    return Err(Box::new(e))
                }
            }
        };
        Ok(resp.text().unwrap())
    }

    // Mock GET request with test data
    // fn get_html(&self, url: &str) -> Result<String, std::io::Error> {
    //     fs::read_to_string("test_data/test_5_instances.txt")
    // }


    /// Iterate through string pulling out each instance captured by this.regex
    pub fn search_for_links(&self, html: String) -> Vec<Task>{
        let new_tasks: Vec<Task> = html.split_whitespace()
            .filter_map(|item| self.is_link_regex(&item))
            .map(|item| Task::new(self.task.domain.clone(), item.to_owned()))
            .collect();
        if new_tasks.len() > 0 {
            println!("\nUrl {:?} \nFound links:", self.task.get_url());
            for new_task in &new_tasks {
                println!("{:?}", new_task.get_url());
            }
        }
        new_tasks
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
    let mut num_chars: usize = task.sub_domain.len();
    num_chars+=1;
    if num_chars%2==0 {
        let new_item = repeat(task.clone().sub_domain.chars().nth(0).unwrap()).take(num_chars).collect::<String>();
        tx.send((task.to_owned(),vec!(Task::new(task.domain, new_item)))).unwrap();
    } else {
        tx.send((task,vec!())).unwrap();
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    fn spawn_worker() -> Worker {
        let (tx, _) = mpsc::channel();
        let task = Task::new("monzo.com".to_string(), "".to_string());
        Worker::new(task, tx, true)
    }

    #[test]
    fn test_regex_gen() {
        let (tx, _) = mpsc::channel();
        let task = Task::new("domain.com".to_string(),"aa".to_string());
        let worker = Worker::new(task, tx, true);
        assert_eq!(worker.regex.to_string(), "^.*(?:https?://)?(?:w{3}.)?domain.com/(.*)[\\\\\"\'](?:[>])?(?:\\n)?.*$");
    }

    #[test]
    fn test_regex_all() {
        let worker = spawn_worker();
        // After path
        assert_eq!(worker.is_link_regex("monzo.com/path\""), Some("path".to_string()));
        assert_eq!(worker.is_link_regex("monzo.com/path\"\n"), Some("path".to_string()));
        assert_eq!(worker.is_link_regex("monzo.com/path\">"), Some("path".to_string()));
        assert_eq!(worker.is_link_regex("monzo.com/path\">\n"), Some("path".to_string()));
        assert_eq!(worker.is_link_regex("monzo.com/path\'"), Some("path".to_string()));
        assert_eq!(worker.is_link_regex("monzo.com/path\'>"), Some("path".to_string()));
        assert_eq!(worker.is_link_regex("monzo.com/path\"aaa"), Some("path".to_string()));
        assert_eq!(worker.is_link_regex("monzo.com/path\">!!"), Some("path".to_string()));
        assert_eq!(worker.is_link_regex("monzo.com/path\","), Some("path".to_string()));
        // before path
        assert_eq!(worker.is_link_regex("monzo.com//path\""), Some("/path".to_string()));
        assert_eq!(worker.is_link_regex("www.monzo.com/path\""), Some("path".to_string()));
        assert_eq!(worker.is_link_regex("http://www.monzo.com/path\""), Some("path".to_string()));
        assert_eq!(worker.is_link_regex("https://www.monzo.com/path\""), Some("path".to_string()));
        assert_eq!(worker.is_link_regex("http://monzo.com/path\""), Some("path".to_string()));
        assert_eq!(worker.is_link_regex("https://monzo.com/path\""), Some("path".to_string()));
        assert_eq!(worker.is_link_regex("link=monzo.com/path\""), Some("path".to_string()));
        assert_eq!(worker.is_link_regex("2134https://monzo.com/path\""), Some("path".to_string()));
        assert_eq!(worker.is_link_regex("ww.https://www.monzo.com/path\""), Some("path".to_string()));

        // failures
        assert_eq!(worker.is_link_regex("monzo.cm/path\""), None);
        assert_eq!(worker.is_link_regex("monzo.cm/path\""), None);
        assert_eq!(worker.is_link_regex("monzo.cpm//path\""), None);
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
