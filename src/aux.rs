use std::string::String;
use std::path::PathBuf;
use std::process::Command;

pub fn rsync(source: &String, destination: &String, host: &String, key : &String) {
    let mut command = Command::new("rsync");
    command.arg("-r");
    if host == "" {
        command.arg(source);
        command.arg(destination);
    } else {
        command.arg("-e");
        if key == "" {
            command.arg("ssh");
        } else {
            command.arg(format!("ssh -i {}", key));
        }
        command.arg(source);
        command.arg(format!("{}:{}", host, destination));
    }
    command.spawn().expect("Command failed to start");
}

pub fn remove(destination: &String, host: &String, key : &String) {
    if host == "" {
        let mut command = Command::new("rm");
        command.arg("-r");
        command.arg(destination);
        command.spawn().expect("Command failed to start");
    } else {
        let mut command = Command::new("ssh");
        if key != "" {
            command.arg("-i");
            command.arg(key);
        }
        command.arg(host);
        command.arg(format!("rm -r {}", destination));
            command.spawn().expect("Command failed to start");
    }
}

pub fn mkdir(destination: &String, host: &String, key : &String) {
    if host == "" {
        let mut command = Command::new("mkdir");
        command.arg("-p");
        command.arg(destination);
        command.spawn().expect("Command failed to start");
    } else {
        let mut command = Command::new("ssh");
        if key != "" {
            command.arg("-i");
            command.arg(key);
        }
        command.arg(host);
        command.arg(format!("mkdir -p {}", destination));
            command.spawn().expect("Command failed to start");
    }
}
/*
pub fn rename(destination: &String, new_path: &String, host: &String) {
    let v: Vec<&str> = destination.rsplit(':').collect();
    let mut command = Command::new("");
    if v.len() == 2 {
        command = Command::new("ssh");
        command
            .arg(v[0])
            .arg(&format!("'mv {} {}'", v[1], new_path));
    } else {
        command.arg(&format!("'mv {} {}'", v[0], new_path));
    }
    command.spawn().expect("Command failed to start");
}
*/
