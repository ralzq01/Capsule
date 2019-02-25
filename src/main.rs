pub mod filewatcher;

use crate::filewatcher::FileWatcher;

fn main() {
    let path = String::from("./");
    let watchers = FileWatcher::new(path);
    loop {
        match watchers.recver.recv() {
            Ok(event) => println!("{:?}", event),
            Err(e) => println!("watch error: {:?}", e),
        }
    }
}
