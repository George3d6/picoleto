extern crate notify;

use std::env;
use std::path::PathBuf;
use std::sync::mpsc::{Sender, Receiver, channel};
use std::time::Duration;
use std::thread;
use std::vec::Vec;
use std::string::String;
use std::process::Command;

use notify::{RecommendedWatcher, Watcher, RecursiveMode, DebouncedEvent};
use notify::DebouncedEvent::*;

fn rsync(source : String, destination : String, host : String) {
    let mut command = Command::new("rsync");
    command.arg("-zrupogtaH");
    command.arg("--stats");
    command.arg("--verbose");
    command.arg("--progress");
    command.arg("-e");
    command.arg("ssh");
    command.arg(source);
    command.arg(format!("{}:{}", host, destination));
    command.spawn().expect("Command failed to start");
}

fn remove(destination : String, host : String) {
    let v: Vec<&str> = destination.rsplit(':').collect();
    if v.len() == 2 {
        let mut command = Command::new("ssh");
        command.arg(" -rf ");
        command.arg(format!("'{}'", v[1].clone()));
        command.spawn().expect("Command failed to start");
    } else {
        let mut command = Command::new("rm");
        command.arg("-rf");
        command.arg(&format!("'{}'", v[0]));
        command.spawn().expect("Command failed to start");
    }
}

fn mkdir(destination : String, host : String) {
    let v: Vec<&str> = destination.rsplit(':').collect();
    if v.len() == 2 {
        let mut command = Command::new("ssh");
        command.arg(v[0]);
        command.arg(&format!("'mkdir -p {}'", v[1]));
        command.spawn().expect("Command failed to start");
    } else {
        let mut command = Command::new("mkdir");
        command.arg(&format!("' -p {}'", v[0]));
        command.spawn().expect("Command failed to start");
    }
}

fn rename(destination : String, new_path : String, host : String) {
    let v: Vec<&str> = destination.rsplit(':').collect();
    let mut command = Command::new("");
    if v.len() == 2 {
        command = Command::new("ssh");
        command.arg(v[0])
        .arg(&format!("'mv {} {}'", v[1], new_path));
    } else {
        command.arg(&format!("'mv {} {}'", v[0], new_path));
    }
    command.spawn().expect("Command failed to start");
}

fn rsync_many(path: PathBuf, destinations : &Vec<String>) {
    for des in destinations {
        rsync(String::from(path.to_str().unwrap()), String::from(path.to_str().unwrap()), des.clone());
    }
}

fn remove_many(path: PathBuf, destinations : &Vec<String>) {
    for des in destinations {
        remove( String::from(path.to_str().unwrap()), des.clone());
    }
}

fn mkdir_many(path: PathBuf, destinations : &Vec<String>) {
    for des in destinations {
        mkdir( String::from(path.to_str().unwrap()), des.clone());
    }
}

fn rename_many(path: PathBuf, new_path: PathBuf, destinations : &Vec<String>) {
    for des in destinations {
        rename(  String::from(path.to_str().unwrap()),  String::from(new_path.to_str().unwrap()), des.clone());
    }
}

fn sync_dir(receiver : Receiver<DebouncedEvent>, destinations : Vec<String>) {
    let sync_loop_thread = thread::spawn(move || {
        loop {
            match receiver.recv() {
                Ok(Write(path)) => rsync_many(path, &destinations),
                Ok(Create(path)) => {
                    if path.is_dir() {
                        mkdir_many(path, &destinations);
                    } else {
                        rsync_many(path, &destinations);
                    }
                },
                Ok(Remove(path)) => remove_many(path, &destinations),
                Ok(Rename(path, new_path)) => rename_many(path, new_path, &destinations),
                Ok(_) => { },
                Err(e) => println!("watch error: {:?}", e),
            }
        }
    });
    sync_loop_thread.join();
}

fn add_watch(mut watcher : &mut RecommendedWatcher, dir : &PathBuf) {
    watcher.watch(dir, RecursiveMode::Recursive).expect("Failed to watch directory");
}

fn main() {
    //Get the path to the testing directory
    let execution_dir = env::current_dir().expect("Failed to determine current directory");

    let mut monitored_dir_buf  = PathBuf::from(&execution_dir);
    monitored_dir_buf.push("test_dir");
    let monitored_dir = monitored_dir_buf;

    let (tx, rx) = channel();
    let mut watcher: RecommendedWatcher = Watcher::new(tx, Duration::from_secs(0)).expect("Failed to initialize notification mechanism");
    add_watch(&mut watcher, & monitored_dir);
    let hosts = vec!( String::from("ryzen3") );
    rsync_many(monitored_dir, &hosts);
    sync_dir(rx, hosts);
}

//#[cfg(test)] mod tests { user super::* }
