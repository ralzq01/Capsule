pub mod filewatcher;

use filewatcher::FileWatcher;

fn main() {
    let path = "./";
    let watchers = FileWatcher::new(path);
    loop {
        match watchers.recv() {
            Ok(event) => println!("{:?}", event),
            Err(e) => println!("watch error: {:?}", e),
        }
    }
}
