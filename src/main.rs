use std::process::{Command};

extern crate nix;
use nix::sched::{unshare,CloneFlags};

fn main() {
    unshare(CloneFlags::CLONE_NEWUSER|CloneFlags::CLONE_NEWNET|CloneFlags::CLONE_NEWIPC).expect("Failed to unshare.");
    let mut p = Command::new("sh").spawn().expect("sh command failed to start");
    p.wait().expect("[Error]: failed to wait");
}
