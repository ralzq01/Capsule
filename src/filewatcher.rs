extern crate notify;

use notify::{Watcher, RecursiveMode, watcher, RecommendedWatcher, DebouncedEvent};
use std::sync::mpsc::{channel, Receiver};
use std::time::Duration;
use std::collections::HashMap;
use std::path::Path;

pub struct FileWatcher {
  recver: Receiver<DebouncedEvent>,
  watcher_map: HashMap<String, RecommendedWatcher>,
}

impl FileWatcher {
  pub fn new(file_path: String) -> FileWatcher {
    // make channels for sending events
    let (tx, rx) = channel();
    // create watcher to watch the changes of files
    let mut watcher = watcher(tx, Duration::from_secs(10)).unwrap();
    watcher.watch(&file_path[..], RecursiveMode::Recursive).unwrap();
    let mut watcher_map = HashMap::new();
    watcher_map.insert(file_path, watcher);
    // return rx to recv the events
    FileWatcher {
      recver: rx,
      watcher_map: watcher_map,
    }
  }
}