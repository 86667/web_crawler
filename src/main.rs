use std::env;
use web_crawler::scheduler::Scheduler;
use web_crawler::task::Task;

use reqwest;

#[tokio::main]
async fn main() {
    let arg = env::args().collect::<Vec<_>>();
    let domain = match arg.get(1) {
        Some(item) => item,
        None => {
            println!("Provide domain as program argument.");
            return ;
        }
    };

    let tasks = vec!(
        Task::new(domain.to_owned(), "".to_string()));

    let mut scheduler = Scheduler::new(tasks, 50, false);
    scheduler.run_tasks().await;
    
    println!("\n");
    let _  = scheduler.get_tasks()
        .iter()
        .inspect(|task| println!("Url searched: {}", task.get_url()))
        .collect::<Vec<_>>();
}

#[allow(dead_code)]
async fn get_html(url: &str) -> Result<String, reqwest::Error> {
    let resp = reqwest::blocking::get(url).unwrap();
    resp.text()
}
