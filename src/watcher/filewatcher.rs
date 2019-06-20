extern crate notify;
extern crate serde_json;

use notify::{Watcher, RecursiveMode, watcher, RecommendedWatcher, DebouncedEvent};
use serde_json::json;
use std::sync::mpsc::{channel, Receiver};
use std::time::Duration;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::watcher::base::MyWatcher;

pub struct FileWatcher<'a> {
  filepath: &'a str,
  ignore_file: &'a str,
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
    let filepath = config.get("filepath").expect("Error: Please fill the filepath section in FileWatcher");
    let ignore_file = config.get("ignore").expect("Error: Ignore file should be specified with `None` or filename");
    println!("start watching file dir: {}", filepath);
    watcher.watch(filepath, RecursiveMode::Recursive).expect("Error: Can't watch this dir");
    FileWatcher {
      filepath: filepath,
      ignore_file: ignore_file,
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
  ///   "old": "path/to/the/old/file" -- the file should be removed
  ///   "abspath": "/abspath/to/'new'/file" -- the absolute path for "new"
  /// }
  fn get(&self) -> String {
    loop {
      let mut watcher_type = String::from("");
      let mut new = String::from("");
      let mut old = String::from("");
      let mut abspath = String::from("");
      if let Ok(event) = self.recver.recv() {
        //println!("{:?}", event);
        match event {
          DebouncedEvent::Create(path) => {
            // this check is due to some bugs in notify:
            // if a file created by vs-code and then deleted without
            // any write, notify will generate a create event for the 
            // delete operation
            let rpath = path.strip_prefix(self.filepath)
                .expect("Unexpected Error: Can't get relevant filepath");
            if path.is_file() || path.is_dir() {
              watcher_type.push_str("Create");
              new.push_str(rpath.to_str().unwrap());
              abspath.push_str(path.to_str().unwrap());
            } else {
              watcher_type.push_str("Remove");
              old.push_str(rpath.to_str().unwrap());
            }
          },
          DebouncedEvent::Write(path) => {
            let rpath = path.strip_prefix(self.filepath)
                .expect("Unexpected Error: Can't get relevant filepath");
            if path.is_file() {
              watcher_type.push_str("Write");
              new.push_str(rpath.to_str().unwrap());
              abspath.push_str(path.to_str().unwrap());
            } else {
              // folder root will changed when create a new file under this folder. ignore.
              continue;
            }
          },
          DebouncedEvent::Remove(path) => {
            let rpath = path.strip_prefix(self.filepath)
                .expect("Unexpected Error: Can't get relevant filepath");
            watcher_type.push_str("Remove");
            old.push_str(rpath.to_str().unwrap());
            abspath.push_str(path.to_str().unwrap());
          },
          DebouncedEvent::Rename(orig_path, new_path) => {
            let orig_rpath = orig_path.strip_prefix(self.filepath)
                .expect("Unexpected Error: Can't get relevant filepath");
            let new_rpath = new_path.strip_prefix(self.filepath)
                .expect("Unexpected Error: Can't get relevant filepath");
            watcher_type.push_str("Rename");
            new.push_str(new_rpath.to_str().unwrap());
            old.push_str(orig_rpath.to_str().unwrap());
          },
          _ => {
            continue;
          }
        }
      } else {
        watcher_type.push_str("Error");
      }
      if self.ignore_file != "None" {
        // ignore_file will be ignored
        if new.contains(self.ignore_file) {
          continue;
        }
      }
      let out = json!({
        "event": "FileWatcher",
        "type": watcher_type,
        "new": new,
        "old": old,
        "abspath": abspath,
      });
      return out.to_string();
    }
  }
}