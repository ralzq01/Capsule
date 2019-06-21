use std::collections::HashMap;

use crate::doer::base::MyDoer;
use crate::doer::base::Status;

pub struct EmptyDoer {
  
}

impl EmptyDoer {
  pub fn new(config: &HashMap<String, String>) -> EmptyDoer {
    EmptyDoer {}
  }
}

impl MyDoer for EmptyDoer {
  fn get(&self, input: String) -> Status {
    Status::OkNone
  }
}