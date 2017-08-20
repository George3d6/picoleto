extern crate inotify;

mod aux;

use inotify::{event_mask, watch_mask, Inotify, WatchDescriptor};

use std::env;
use std::path::PathBuf;
use std::fs;
use std::thread;
use std::collections::HashMap;

fn pop_til_equal(against: PathBuf, mut compared: PathBuf) -> Option<PathBuf> {
    let mut difference_rev = PathBuf::new();
    while against != compared {
        if compared.file_name() == None || against.file_name() == None {
            return None;
        }
        difference_rev.push(compared.iter().last().unwrap());
        compared.pop();
    }
    let mut difference = PathBuf::new();
    for element in difference_rev.iter().rev() {
        difference.push(element);
    }
    return Some(difference);
}

fn path_to_str(to_string: &PathBuf) -> String {
    return String::from(to_string.clone().as_os_str().to_string_lossy());
}

struct Watcher {
    descriptor_to_dir: HashMap<WatchDescriptor, PathBuf>,
    inotify: Inotify,
}

impl Watcher {
    pub fn new() -> Watcher {
        return Watcher {
            descriptor_to_dir: HashMap::new(),
            inotify: Inotify::init().expect("Error, failed to initialize inotify"),
        };
    }

    fn watch(&mut self, root: &PathBuf, dir: &PathBuf) {
        let mut to_watch = root.clone();
        to_watch.push(dir);
        let watch_descriptor = self.inotify
            .add_watch(
                to_watch.clone(),
                watch_mask::MODIFY | watch_mask::CREATE | watch_mask::DELETE,
            )
            .expect(&format!(
                "Failed to add inotify watch to directory: {}",
                to_watch.as_os_str().to_string_lossy()
            ));
        self.descriptor_to_dir.insert(watch_descriptor, dir.clone());
    }

    fn watch_rec(&mut self, root: &PathBuf, dir: &PathBuf) {
        let mut to_watch = root.clone();
        to_watch.push(dir);
        self.watch(&root, &dir);
        for entry in fs::read_dir(to_watch.clone()).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                match pop_til_equal(root.clone(), path.clone()) {
                    Some(new_dir) => {
                        println!(
                            "Adding: {0} with prefix {1} from path {2}",
                            new_dir.display(),
                            root.display(),
                            path.display()
                        );
                        self.watch_rec(&root, &new_dir);
                    }
                    None => {
                        println!("Crashing in watch_rec,this shouldn't happen,check source code");
                        std::process::exit(42)
                    }
                }
            }
        }
    }
}

fn monitor_dir(monitored_dir_buf: PathBuf, remote_dir: PathBuf, host: String) {
    let mut watcher = Watcher::new();
    //Start watching it for changes
    watcher.watch_rec(&monitored_dir_buf, &PathBuf::from(""));
    //Do this for all watched directories
    let mut event_buf = [0u8; 4096];
    loop {
        let events = watcher
            .inotify
            .read_events_blocking(&mut event_buf)
            .expect("Failed to read inotify events");
        for event in events {

            let mut modified = PathBuf::new();
            match watcher.descriptor_to_dir.get(&event.wd) {
                Some(dir) => modified.push(dir),
                None => modified.push(""),
            }
            modified.push(PathBuf::from(event.name));
            let modified_name = String::from(modified.clone().as_os_str().to_string_lossy());
            let mut modified_local_path = monitored_dir_buf.clone();
            modified_local_path.push(modified_name.clone());
            let mut modified_host_path = remote_dir.clone();
            modified_host_path.push(modified_name.clone());

            if event.mask.contains(event_mask::CREATE) {
                if event.mask.contains(event_mask::ISDIR) {
                    watcher.watch_rec(&monitored_dir_buf, &modified);
                    aux::mkdir(&path_to_str(&modified_host_path), &host);
                    println!(
                        "Directory created: {0} resulting in creation of {1}",
                        modified_name,
                        path_to_str(&modified_host_path)
                    );
                } else {
                    aux::rsync(
                        &path_to_str(&modified_local_path),
                        &path_to_str(&modified_host_path),
                        &host,
                    );
                    println!("File created: {}", modified_name);
                }
            } else if event.mask.contains(event_mask::DELETE) {
                aux::remove(&path_to_str(&modified_host_path), &host);
                println!("Deleted: {}", modified_name);
            } else if event.mask.contains(event_mask::MODIFY) {
                aux::rsync(
                    &path_to_str(&modified_local_path),
                    &path_to_str(&modified_host_path),
                    &host,
                );
                println!("Modified: {}", modified_name);
            } else if event.mask.contains(event_mask::MOVED_FROM) {
                aux::remove(&path_to_str(&modified_host_path), &host);
                println!("Moved From: {}", modified_name);
            } else if event.mask.contains(event_mask::MOVED_TO) {
                aux::remove(&path_to_str(&modified_host_path), &host);
                aux::rsync(
                    &path_to_str(&modified_local_path),
                    &path_to_str(&modified_host_path),
                    &host,
                );
                println!("Moved to: {}", modified_name);
            }
        }
    }
}

fn main() {
    //Get the path to the testing directory
    let execution_dir = env::current_dir().expect("Failed to determine current directory");
    let mut monitored_dir_buf = PathBuf::from(execution_dir);
    monitored_dir_buf.push("test_dir");
    let monitored_dir = monitored_dir_buf;
    monitor_dir(
        monitored_dir,
        PathBuf::from("/tmp/test"),
        String::from("george@localhost"),
    );
}

//#[cfg(test)] mod tests { user super::* }
