extern crate ini;
pub mod filewatcher;

use std::collections::HashMap;
use crate::filewatcher::FileWatcher;
use std::path::Path;
use ini::Ini;

fn main() {
  let config = read_config();
  //let path = String::from("./");
  //let watchers = FileWatcher::new(path);
  let path = Path::new(config.get("filepath").unwrap());
  let watchers = FileWatcher::new(path.to_str().unwrap());
  loop {
    match watchers.recver.recv() {
      Ok(event) => println!("{:?}", event),
      Err(e) => println!("watch error: {:?}", e),
    }
  }
}

fn read_config() -> HashMap<String, String> {
  let mut config = HashMap::new();
  let conf = Ini::load_from_file("config.ini").unwrap();
  let watcher = conf.section(Some("Watcher".to_owned())).unwrap()
                    .get("type").unwrap();
  config.insert("Watcher".to_string(), watcher.clone());
  let doer = conf.section(Some("Do".to_owned())).unwrap()
                 .get("type").unwrap();
  config.insert("Do".to_string(), doer.clone());
  // get watcher type
  let watcher_type = conf.section(Some(watcher.clone())).unwrap();
  // setup watcher value
  if watcher.trim() == "FileWatcher" {
    config.insert(
      "filepath".to_string(),
      watcher_type.get("filepath").unwrap().clone(),
    );
    config.insert(
      "recursive".to_string(),
      watcher_type.get("recursive").unwrap().clone(),
    );
    config.insert(
      "ignore".to_string(),
      watcher_type.get("ignore").unwrap().clone(),
    );
  }
  config
}
