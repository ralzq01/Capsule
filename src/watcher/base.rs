use std::sync::mpsc::Receiver;

pub trait MyWatcher {
  fn get(&self) -> String;
}