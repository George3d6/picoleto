use std::string::String;
use std::path::PathBuf;
use std::process::Command;

pub fn rsync(source: &String, destination: &String, host: &String) {
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

pub fn remove(destination: &String, host: &String) {
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

pub fn mkdir(destination: &String, host: &String) {
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
