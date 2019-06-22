use std::collections::HashMap;
use ini::Ini;

mod watcher;
use crate::watcher::base::MyWatcher;
use crate::watcher::emptywatcher::EmptyWatcher;
use crate::watcher::filewatcher::FileWatcher;

mod doer;
use crate::doer::base::{MyDoer, Status};
use crate::doer::emptydoer::EmptyDoer;
use crate::doer::remotesync::RemoteSync;
use crate::doer::emailsender::EmailSender;

fn main() {
  let config = read_config();

  // get watcher
  let watcher_filewatcher;
  let watcher_empty;
  let watcher = match config.get("Watcher").unwrap().as_ref() {
    "FileWatcher" => {
      watcher_filewatcher = FileWatcher::new(&config);
      &watcher_filewatcher as &MyWatcher
    },
    _ => {
      watcher_empty = EmptyWatcher::new(&config);
      &watcher_empty as &MyWatcher
    }
  };

  // get doer
  let doer_remotesync;
  let doer_emailsender;
  let doer_empty;
  let doer = match config.get("Do").unwrap().as_ref() {
    "RemoteSync" => {
      doer_remotesync = RemoteSync::new(&config);
      &doer_remotesync as &MyDoer
    },
    "EmailSender" => {
      doer_emailsender = EmailSender::new(&config);
      &doer_emailsender as &MyDoer
    },
    _ => {
      doer_empty = EmptyDoer::new(&config);
      &doer_empty as &MyDoer
    }
  };

  // build capsule
  loop {
    let out = watcher.get();
    println!("{}", &out);
    let res = doer.get(out);

    if let Status::Error(msg) = res {
      println!("An Error Occured: {}", msg);
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
  let watcher_map = conf.section(Some(watcher.clone())).unwrap();
  // merge watcher reference map section
  config.extend(watcher_map.into_iter().map(|(k, v)| (k.clone(), v.clone())));
  // get doer type
  let doer_map = conf.section(Some(doer.clone())).unwrap();
  // merge reference doer map section
  config.extend(doer_map.into_iter().map(|(k, v)| (k.clone(), v.clone())));
  config
}
