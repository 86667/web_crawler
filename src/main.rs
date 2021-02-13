use web_crawler::scheduler::Scheduler;
use web_crawler::task::Task;
use reqwest;

#[tokio::main]
async fn main() {
    let domain = String::from("google.com");

    let tasks = vec!(
        Task::new(domain));

    let mut scheduler = Scheduler::new(tasks, 50, false);
    scheduler.run_tasks().await;
    let _  = scheduler.get_tasks()
        .iter()
        .inspect(|task| println!("Domain: {}", task.item))
        .collect::<Vec<_>>();
}

#[allow(dead_code)]
async fn get_html(url: &str) -> Result<String, reqwest::Error> {
    let resp = reqwest::blocking::get(url).unwrap();
    resp.text()
}
