// Task is a single "task" item of data to pass to a worker.
#[derive(Clone, Debug, PartialEq)]
pub struct Task {
    pub domain: String,
    pub sub_domain: String,
    pub status: usize
}
impl Task {
    pub fn new(domain: String, sub_domain: String) -> Self { Self{domain, sub_domain, status: 0} }

    pub fn get_url(&self) -> String {
        let mut url = self.domain.clone();
        url.push_str("/");
        url.push_str(&self.sub_domain);
        url
    }
}




#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_rul() {
        let task = Task::new("domain".to_string(), "a".to_string());
        assert_eq!(task.get_url(), String::from("domain/a"));
    }
}
