extern crate notify;
extern crate serde_json;

use notify::{Watcher, RecursiveMode, watcher, RecommendedWatcher, DebouncedEvent};
use serde_json::json;
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
    let check_interval = config.get("check_interval_sec")
                         .unwrap()
                         .parse::<u64>()
                         .unwrap();
    let mut watcher = watcher(tx, Duration::from_secs(check_interval)).unwrap();
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
  /// format: json
  /// {
  ///   "event": "FileWatcher" -- every watcher should specify this
  ///   "type": "Create" / "Write" / "Remove" / "Rename" / "Error",
  ///   "new": "path/to/the/changed(new)/file"
  ///   "old": "path/to/the/old//file" -- the file should be removed
  /// }
  fn get(&self) -> String {
    loop {
      let mut watcher_type = String::from("");
      let mut new = String::from("");
      let mut old = String::from("");
      if let Ok(event) = self.recver.recv() {
        //println!("{:?}", event);
        match event {
          DebouncedEvent::Create(path) => {
            // this check is due to some bugs in notify:
            // if a file created by vs-code and then deleted without
            // any write, notify will generate a create event for the 
            // delete operation
            if path.is_file() {
              watcher_type.push_str("Create");
              new.push_str(path.to_str().unwrap());
            } else {
              watcher_type.push_str("Remove");
              old.push_str(path.to_str().unwrap());
            }
          },
          DebouncedEvent::Write(path) => {
            if path.is_file() {
              watcher_type.push_str("Write");
              new.push_str(path.to_str().unwrap());
            }
          },
          DebouncedEvent::Remove(path) => {
            watcher_type.push_str("Remove");
            old.push_str(path.to_str().unwrap());
          },
          DebouncedEvent::Rename(orig_path, new_path) => {
            watcher_type.push_str("Rename");
            new.push_str(new_path.to_str().unwrap());
            old.push_str(orig_path.to_str().unwrap());
          },
          _ => {
            continue;
          }
        }
      } else {
        watcher_type.push_str("Error");
      }
      let out = json!({
        "event": "FileWatcher",
        "type": watcher_type,
        "new": new,
        "old": old,
      });
      return out.to_string();
    }
  }
}