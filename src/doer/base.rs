
pub enum Status {
  OkNone,
  OkText(String),
  Error(String),
}

pub trait MyDoer {
  /// Every doer should at least implemented this
  /// Recv an output of a watcher in json format
  /// and handle the recved event in this function
  /// Returned result should be Ok() or Error("Error Info")
  fn get(&self, event: String) -> Status;
}