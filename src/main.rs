use std::process::{Command};
use std::fs::File;
use std::io::{Error,Write};

extern crate nix;
use nix::sched::{unshare,CloneFlags};
use nix::unistd::{getuid,getgid};

struct UidGidMap{
    container_id: u32,
    host_id: u32,
    size: u32,
}

fn main() {
    let uid_map = UidGidMap{
        container_id: 0,
        host_id: getuid().as_raw(),
        size: 1,
    };
    let gid_map = UidGidMap{
        container_id: 0,
        host_id: getgid().as_raw(),
        size: 1,
    };
    unshare(CloneFlags::CLONE_NEWUSER|CloneFlags::CLONE_NEWNET|CloneFlags::CLONE_NEWIPC).expect("Failed to unshare.");
    add_mapping("/proc/self/uid_map", &uid_map).expect("failed add uid");
    init_setgroups();
    add_mapping("/proc/self/gid_map", &gid_map).expect("failed add gid");
    let mut p = Command::new("sh").spawn().expect("sh command failed to start");
    p.wait().expect("[Error]: failed to wait");
}

fn init_setgroups(){
    let mut fd = File::create("/proc/self/setgroups").expect("failed to open file");
    fd.write_all(b"deny").expect("failed edit file");
}

fn add_mapping(path: &str,map: &UidGidMap) -> Result<(),Error>{
    let data = String::from(format!("{} {} {}\n",map.container_id,map.host_id,map.size));
    if !data.is_empty(){
        let mut fd = File::create(path).expect("failed to open file");
        fd.write_all(data.as_bytes()).expect(&format!("failed to write file{}",path));
    }
    Ok(())
}
