extern crate inotify;

use std::env;
use std::path::PathBuf;
use std::vec::Vec;

use inotify::{event_mask, watch_mask, Inotify,};

fn main() {
    //Instantiate inotify
    let mut inotify = Inotify::init().expect("Error, failed to initialize inotify");
    //Get the path to the testing directory
    let execution_dir = env::current_dir().expect("Failed to determine current directory");
    let mut monitored_dir_buf  = PathBuf::from(execution_dir);
    monitored_dir_buf.push("test_dir");
    let monitored_dir = monitored_dir_buf;
    //Start watching it for changes

    inotify.add_watch(monitored_dir.clone(), watch_mask::MODIFY | watch_mask::CREATE | watch_mask::DELETE,)
        .expect( &format!("Failed to add inotify watch to directory: {}", monitored_dir.as_os_str().to_string_lossy()) );

    let mut event_buf = [0u8; 4096];

    loop {
        let events = inotify.read_events_blocking(&mut event_buf).expect("Failed to read inotify events");

        for event in events {
            if event.mask.contains(event_mask::CREATE) {
                if event.mask.contains(event_mask::ISDIR) {
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

}

//
