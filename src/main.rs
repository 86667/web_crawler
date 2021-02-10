use web_crawler::scheduler::{Task, Scheduler};

use std::collections::HashMap;
use std::fs;
use regex::Regex;


#[tokio::main]
async fn main() {
    // let domain = "https://monzo.com/";
    // let html = get_html(domain);
    // let html = fs::read_to_string("test_data/monzo.txt").unwrap();
    // search_for_links(&html)

    let tasks = vec!(Task::TaskTester(1),Task::TaskTester(2),Task::TaskTester(3),Task::TaskTester(4));

    let scheduler = Scheduler::new(tasks);
    scheduler.run_tasks().await;
}

const REGEX: &str = r#"^.*(?:https?://)?(?:w{3}.)?monzo.com/?(.*)$"#;

async fn get_html(url: &str) -> Result<String, Box<dyn std::error::Error>> {
    let resp = reqwest::get(url)
        .await?;
    Ok(resp.text().await?)
}


/// Worker pulls a given webpage and returns all instances of provided REGEX.
pub struct Worker {
    regex: Regex
}

impl Worker {
    fn run_task() -> Self {
        Worker{
            regex: Regex::new(REGEX).unwrap()
        }
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
        let res = self.regex.captures(is_link_regex)?;
        match res.get(1) {
            Some(item) => return Some(item.as_str().to_string()),
            None => return None
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_regex() {
        let worker = Worker::run_task();
        assert_eq!(worker.is_link_regex("monzo.com/path"), Some("path".to_string()));
        assert_eq!(worker.is_link_regex("monzo.com//path"), Some("/path".to_string()));
        assert_eq!(worker.is_link_regex("www.monzo.com/path"), Some("path".to_string()));
        assert_eq!(worker.is_link_regex("http://www.monzo.com/path"), Some("path".to_string()));
        assert_eq!(worker.is_link_regex("https://www.monzo.com/path"), Some("path".to_string()));
        assert_eq!(worker.is_link_regex("http://monzo.com/path"), Some("path".to_string()));
        assert_eq!(worker.is_link_regex("https://monzo.com/path"), Some("path".to_string()));
        assert_eq!(worker.is_link_regex("ww.monzo.com/path"), Some("path".to_string()));
        assert_eq!(worker.is_link_regex("a.monzo.com/path"), Some("path".to_string()));
        assert_eq!(worker.is_link_regex(".monzo.com/path"), Some("path".to_string()));
        assert_eq!(worker.is_link_regex("wwwmonzo.com/path"), Some("path".to_string()));

        assert_eq!(worker.is_link_regex("monzo.cm/path"), None);
        assert_eq!(worker.is_link_regex("monzo.cm/path"), None);
        assert_eq!(worker.is_link_regex("monzo.cpm//path"), None);
    }
}
