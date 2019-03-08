use std::collections::HashMap;
use ini::Ini;

mod watcher;
use crate::watcher::base::MyWatcher;
use crate::watcher::filewatcher::FileWatcher;

fn main() {
  let config = read_config();
  let watcher = FileWatcher::new(&config);
  loop {
    let modified = watcher.get();
    println!("{}", modified);
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
  let watcher_map = conf.section(Some(watcher.clone())).unwrap();
  // merge watcher reference map section
  config.extend(watcher_map.into_iter().map(|(k, v)| (k.clone(), v.clone())));
  // get doer type
  let doer_map = conf.section(Some(doer.clone())).unwrap();
  // merge reference doer map section
  config.extend(doer_map.into_iter().map(|(k, v)| (k.clone(), v.clone())));
  config
}
