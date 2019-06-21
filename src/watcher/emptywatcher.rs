extern crate serde_json;

use std::collections::HashMap;
use serde_json::json;

use crate::watcher::base::MyWatcher;

pub struct EmptyWatcher {

}

impl EmptyWatcher {
  pub fn new(config: &HashMap<String, String>) -> EmptyWatcher {
    EmptyWatcher{}
  }
}

impl MyWatcher for EmptyWatcher {
  fn get(&self) -> String {
    let out = json!({
      "event": "EmptyWatcher",
    });
    out.to_string()
  }
}