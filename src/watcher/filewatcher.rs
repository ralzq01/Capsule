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
  /// will return a String
  /// format: type:filepath(absolute)
  fn get(&self) -> String {
    loop {
      if let Ok(event) = self.recver.recv() {
        //println!("{:?}", event);
        match event {
          DebouncedEvent::Create(path) => {
            // this check is due to some bugs in notify:
            // if a file created by vs-code and then deleted without
            // any write, notify will generate a create event for the 
            // delete operation
            if path.is_file() {
              let mut res = String::from("Create:");
              res.push_str(path.to_str().unwrap());
              return res;
            } else {
              let mut res = String::from("Remove:");
              res.push_str(path.to_str().unwrap());
              return res;
            }
          },
          DebouncedEvent::Write(path) => {
            if path.is_file() {
              let mut res = String::from("Write:");
              res.push_str(path.to_str().unwrap());
              return res;
            }
          },
          DebouncedEvent::Remove(path) => {
            let mut res = String::from("Remove:");
            res.push_str(path.to_str().unwrap());
            return res;
          },
          DebouncedEvent::Rename(orig_path, new_path) => {
            let mut res = String::from("Rename:");
            res.push_str(orig_path.to_str().unwrap());
            res.push_str(";");
            res.push_str(new_path.to_str().unwrap());
            return res
          },
          _ => {
            continue;
          }
        }
      } else {
        return String::from("Error:Detect A Error");
      }
    }
  }
}