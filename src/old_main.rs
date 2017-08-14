extern crate inotify;

use std::env;
use std::path::PathBuf;
use std::io;
use std::fs::{self, DirEntry};
use std::thread;

use inotify::{event_mask, watch_mask, Inotify,};

fn watch(mut inotify : &mut Inotify, dir : &PathBuf) {
    inotify.add_watch(dir.clone(), watch_mask::MODIFY | watch_mask::CREATE | watch_mask::DELETE,)
        .expect( &format!("Failed to add inotify watch to directory: {}", dir.clone().as_os_str().to_string_lossy()) );
}

fn watch_rec(mut inotify : &mut Inotify, root : &PathBuf) {
    for entry in fs::read_dir(root).unwrap() {
        let path =  entry.unwrap().path();
        if path.is_dir() {
            watch(&mut inotify, &path);
            watch_rec(&mut inotify, &path);
        }
    }
}

fn main() {
    //Instantiate inotify
    let mut inotify = Inotify::init().expect("Error, failed to initialize inotify");
    //Get the path to the testing directory
    let execution_dir = env::current_dir().expect("Failed to determine current directory");
    let mut monitored_dir_buf  = PathBuf::from(execution_dir);
    monitored_dir_buf.push("test_dir");
    let monitored_dir = monitored_dir_buf;
    //Start watching it for changes
    watch_rec(&mut inotify, &monitored_dir);

    let monitoring_loop = thread::spawn(move || {
        let mut event_buf = [0u8; 4096];
        loop {
            let events = inotify.read_events_blocking(&mut event_buf).expect("Failed to read inotify events");
            for event in events {
                if event.mask.contains(event_mask::CREATE) {
                    if event.mask.contains(event_mask::ISDIR) {
                        let mut new_dir = monitored_dir.clone();
                        new_dir.push(PathBuf::from(event.name));
                        inotify.add_watch(new_dir.clone() , watch_mask::MODIFY | watch_mask::CREATE | watch_mask::DELETE,)
                        .expect( &format!("Failed to add inotify watch to newly created dir {}", new_dir.display()) );
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

//#[cfg(test)] mod tests { user super::* }
