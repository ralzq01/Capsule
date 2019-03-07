extern crate notify;

use notify::{Watcher, RecursiveMode, watcher, RecommendedWatcher, DebouncedEvent};
use std::sync::mpsc::{channel, Receiver};
use std::time::Duration;
use std::collections::HashMap;

use crate::watcher::base::MyWatcher;

pub struct FileWatcher<'a> {
  filepath: &'a str,
  watcher: RecommendedWatcher,
  pub recver: Receiver<DebouncedEvent>,
}

impl<'a> FileWatcher<'a> {
  pub fn new(config: &'a HashMap<String, String>) -> FileWatcher<'a> {
    // make channels for sending events
    let (tx, rx) = channel();
    // create watcher to watch the changes of files
    let mut watcher = watcher(tx, Duration::from_secs(2)).unwrap();
    let filepath = config.get("filepath").unwrap();
    watcher.watch(filepath, RecursiveMode::Recursive).unwrap();
    FileWatcher {
      filepath: filepath,
      watcher: watcher,
      recver: rx,
    }
  }
}

impl<'a> MyWatcher for FileWatcher<'a> {
  fn get(&self) -> String {
    // not implemented yet
    String::from("not implemented yet")
  }
}