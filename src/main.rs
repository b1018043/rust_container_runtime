use std::process::{Command};
use std::fs::{File,remove_dir_all};
use std::io::{Error,Write};

extern crate nix;
use nix::sched::{unshare,CloneFlags};
use nix::unistd::{getuid,getgid,sethostname,fork,ForkResult,chdir,mkdir,pivot_root};
use nix::sys::wait::{waitpid};
use nix::sys::stat::{Mode};
use nix::mount::{mount,umount2,MsFlags,MntFlags};

struct UidGidMap{
    container_id: u32,
    host_id: u32,
    size: u32,
}

fn main() {
    run_container().expect("failed to run container");
}

fn run_container()->Result<(),Error>{

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

    unshare(
        CloneFlags::CLONE_NEWUSER|
        CloneFlags::CLONE_NEWUTS|
        CloneFlags::CLONE_NEWIPC|
        CloneFlags::CLONE_NEWNET|
        CloneFlags::CLONE_NEWPID|
        CloneFlags::CLONE_NEWNS
    ).expect("Failed to unshare.");

    add_mapping("/proc/self/uid_map", &uid_map).expect("failed add uid");
    init_setgroups();
    add_mapping("/proc/self/gid_map", &gid_map).expect("failed add gid");

    match unsafe{fork().expect("aa")}{
        ForkResult::Child =>{

            sethostname("container").expect("failed to hostname");

            mount(
                Some("proc"),"/root/rootfs/proc",
                Some("proc"),MsFlags::empty(),
                None::<&str>
            ).expect("failed to mount fs");

            chdir("/root").expect("failed to /root");

            mount(Some("rootfs"), "/root/rootfs",
                None::<&str>,
                MsFlags::MS_BIND|MsFlags::MS_REC,
                None::<&str>
            ).expect("failed to mount rootfs");

            mkdir("/root/rootfs/oldrootfs",Mode::S_IRWXU).expect("failed to mkdir oldrootfs");

            pivot_root("rootfs", "/root/rootfs/oldrootfs").expect("failed to pivot_root");

            umount2("/oldrootfs", MntFlags::MNT_DETACH).expect("failed to unmount");

            remove_dir_all("/oldrootfs").expect("failed to remove dir");

            chdir("/").expect("failed to chdir /");
        
            let mut p = Command::new("/bin/sh").spawn().expect("sh command failed to start");
            p.wait().expect("[Error]: failed to wait");
            
        },
        ForkResult::Parent{child} =>{
            waitpid(child, None).expect("err:wait");
        }
    };
    Ok(())
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
