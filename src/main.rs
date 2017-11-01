extern crate inotify;
#[macro_use]
extern crate serde_derive;

mod aux;
mod config;

use inotify::{EventMask, WatchMask, Inotify, WatchDescriptor};
use std::path::PathBuf;
use std::{env, fs, thread};
use std::vec::Vec;
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
                WatchMask::MODIFY | WatchMask::CREATE | WatchMask::DELETE
                | WatchMask::MOVED_FROM | WatchMask::MOVED_TO,
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
                    Some(new_dir) => self.watch_rec(&root, &new_dir),
                    None => {
                        println!("Crashing calling inotify's watch rec,this shouldn't happen,check source code");
                        std::process::exit(1)
                    }
                }
            }
        }
    }
}

fn monitor_dir(monitored_dir_buf: PathBuf, remote_dir: PathBuf, host: String, key: String) {
    let mut watcher = Watcher::new();
    watcher.watch_rec(&monitored_dir_buf, &PathBuf::from(""));

    //First rsync the directories in case the user is expecting them to sync automatically
    let mut monitored_dir_buf_all = path_to_str(&monitored_dir_buf);
    monitored_dir_buf_all.push_str("/");
    let mut remote_dir_target = path_to_str(&remote_dir);
    remote_dir_target.push_str("/");
    aux::rsync(
        &monitored_dir_buf_all,
        &remote_dir_target,
        &host,
        &key,
    );

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
            match event.name {
                Some(name) => modified.push(PathBuf::from(name)),
                None => continue,
            }
            let modified_name = String::from(modified.clone().as_os_str().to_string_lossy());
            let mut modified_local_path = monitored_dir_buf.clone();
            modified_local_path.push(modified_name.clone());
            let mut modified_host_path = remote_dir.clone();
            modified_host_path.push(modified_name.clone());

            if event.mask.contains(EventMask::CREATE) {
                if event.mask.contains(EventMask::ISDIR) {
                    watcher.watch_rec(&monitored_dir_buf, &modified);
                    aux::mkdir(&path_to_str(&modified_host_path), &host, &key);
                } else {
                    aux::rsync(
                        &path_to_str(&modified_local_path),
                        &path_to_str(&modified_host_path),
                        &host,
                        &key,
                    );
                }
            } else if event.mask.contains(EventMask::DELETE) {
                aux::remove(&path_to_str(&modified_host_path), &host, &key);
            } else if event.mask.contains(EventMask::MODIFY) {
                aux::rsync(
                    &path_to_str(&modified_local_path),
                    &path_to_str(&modified_host_path),
                    &host,
                    &key,
                );
            } else if event.mask.contains(EventMask::MOVED_FROM) {
                if event.mask.contains(EventMask::ISDIR) {
                    for(wd,path) in &watcher.descriptor_to_dir {
                        if path_to_str(path) == path_to_str(&modified_host_path) {
                            watcher.inotify.rm_watch(wd.clone());
                        }
                    }
                }
                aux::remove(&path_to_str(&modified_host_path), &host, &key);
            } else if event.mask.contains(EventMask::MOVED_TO) {
                aux::remove(&path_to_str(&modified_host_path), &host, &key);
                if event.mask.contains(EventMask::ISDIR) {
                    watcher.watch_rec(&monitored_dir_buf, &modified);
                }
                aux::rsync(
                    &path_to_str(&modified_local_path),
                    &path_to_str(&modified_host_path),
                    &host,
                    &key,
                );
            }
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let cfg_file_path;
    if args.len() > 1 {
        cfg_file_path = args[1].clone();
    } else {
        cfg_file_path = String::from("/etc/picoleto.config.json");
    }
    let cfg = config::read_cfg(cfg_file_path);
    let mut threads = Vec::new();

    for block in cfg.synchronize {
        let handle = thread::spawn(move || {
            monitor_dir(
                PathBuf::from(block.local),
                PathBuf::from(block.remote),
                block.host,
                block.key,
            );
        });
        threads.push(handle);
    }
    for thread in threads {
        let _ = thread.join();
    }
}
