extern crate notify;

use notify::{Watcher, RecursiveMode, watcher, RecommendedWatcher, DebouncedEvent};
use std::sync::mpsc::{channel, Receiver};
use std::time::Duration;
use std::collections::HashMap;

pub struct FileWatcher {
  pub recver: Receiver<DebouncedEvent>,
  watcher: RecommendedWatcher,
}

impl FileWatcher {
  pub fn new(config: &HashMap<String, String>) -> FileWatcher {
    // make channels for sending events
    let (tx, rx) = channel();
    // create watcher to watch the changes of files
    let mut watcher = watcher(tx, Duration::from_secs(2)).unwrap();
    let filepath = config.get("filepath").unwrap();
    watcher.watch(filepath, RecursiveMode::Recursive).unwrap();
    FileWatcher {
      recver: rx,
      watcher: watcher,
    }
  }
}