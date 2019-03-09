
pub trait MyWatcher {
  /// All watcher should implement this
  /// Output should be a json format
  /// There should at least one of filed "event" to specify
  /// the identity of the watcher.
  fn get(&self) -> String;
}