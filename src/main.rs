extern crate inotify;

 use inotify::{event_mask, watch_mask, Inotify, WatchDescriptor};
use std::env;
use std::path::PathBuf;
use std::io;
use std::fs::{self, DirEntry};
use std::thread;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;

struct Watcher {
    descriptor_to_dir : Arc<Mutex<HashMap<WatchDescriptor,PathBuf>>>,
    inotify : Inotify,
}

impl Watcher {
    pub fn new() ->Watcher {
        let mut watcher : Watcher;
        watcher.descriptor_to_dir = Arc::new(Mutex::new(HashMap::new()));
        watcher.inotify = Inotify::init().expect("Error, failed to initialize inotify");
        return watcher;
    }

    fn watch(&mut self, root : &PathBuf, root_remote : &PathBuf, dir : &PathBuf) {
        let watch_descriptor = self.inotify.add_watch(dir.clone(), watch_mask::MODIFY | watch_mask::CREATE | watch_mask::DELETE,)
            .expect( &format!("Failed to add inotify watch to directory: {}", dir.clone().as_os_str().to_string_lossy()) );
        self.descriptor_to_dir.lock().unwrap().insert(watch_descriptor, dir)
    }

    fn watch_rec(&mut self, root : &PathBuf, root_remote : &PathBuf, dir : &PathBuf) {
        for entry in fs::read_dir(root).unwrap() {
            let path =  entry.unwrap().path();
            if path.is_dir() {
                self.watch(&path, &root_remote, &dir);
                self.watch_rec(&path, &root_remote, &dir);
            }
        }
    }
}

fn monitor_dir(monitored_dir_buf : &PathBuf, remote_dir : &PathBuf) {
    let watcher = Watcher::new();
    //Start watching it for changes
    let remote = &PathBuf::from("/tmp/test");
    watcher.watch_rec(&monitored_dir_buf, &remote, &PathBuf::from(""));
    //Do this for all watched directories
    let monitoring_loop = thread::spawn(move || {
        let mut event_buf = [0u8; 4096];
        loop {
            let events = watcher.inotify.read_events_blocking(&mut event_buf).expect("Failed to read inotify events");
            for event in events {
                if event.mask.contains(event_mask::CREATE) {
                    if event.mask.contains(event_mask::ISDIR) {
                        let mut new_dir = PathBuf::new();
                        new_dir.push(PathBuf::from(event.name));
                        watcher.watch_rec(&monitored_dir_buf, &remote, &new_dir)
                        println!("Directory created: {:?}", event.name);
                        } else {
                            println!("File created: {:?}", event.name);
                        }
                    } else if event.mask.contains(event_mask::DELETE) {
                        if event.mask.contains(event_mask::ISDIR) {
                            println!("Directory deleted: {:?}", event.name);
                        } else {
                            println!("File deleted: {:?}", event.name);
                        }
                    } else if event.mask.contains(event_mask::MODIFY) {
                        if event.mask.contains(event_mask::ISDIR) {
                            println!("Directory modified: {:?}", event.name);
                        } else {
                            println!("File modified: {:?}", event.name);
                        }
                }
            }
        }
    });
    monitoring_loop.join();
}

fn main() {
    //Get the path to the testing directory
    let execution_dir = env::current_dir().expect("Failed to determine current directory");
    let mut monitored_dir_buf  = PathBuf::from(execution_dir);
    monitored_dir_buf.push("test_dir");
    let monitored_dir = monitored_dir_buf;
    monitor_dir(&monitored_dir_buf);

}

//#[cfg(test)] mod tests { user super::* }
