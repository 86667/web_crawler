// Task is a single "task" item of data to pass to a worker.
#[derive(Clone, Debug, PartialEq)]
pub struct Task {
    pub item: String,
    pub status: usize
}
impl Task {
    pub fn new(item: String) -> Self { Self{item, status: 0} }
}
