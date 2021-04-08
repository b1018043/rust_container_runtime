use std::process::{Command};
use std::fs::{File,remove_dir_all};
use std::io::{Error,Write};

extern crate nix;
use nix::sched::{unshare,CloneFlags};
use nix::unistd::{getuid,getgid,sethostname,fork,ForkResult,chdir,mkdir,pivot_root,getpid};
use nix::sys::wait::{waitpid};
use nix::sys::stat::{Mode};
use nix::mount::{mount,umount2,MsFlags,MntFlags};

extern crate clap;
use clap::{Arg,App,SubCommand};

use anyhow::Result;

struct UidGidMap{
    container_id: u32,
    host_id: u32,
    size: u32,
}

fn main(){
    let matches = App::new("Rust container").version("0.1")
        .author("Taito Morikawa").about("container runtime")
        .subcommand(SubCommand::with_name("run"))
            .about("run container")
            .version("0.1")
            .author("Taito Morikawa")
        .get_matches();

    if let Some(_) = matches.subcommand_matches("run") {
        run_container().expect("Error:run_container() failed.");
    }
}

fn run_container()->Result<()>{

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

    println!("{}",getpid());

    unshare(
        CloneFlags::CLONE_NEWUSER|
        CloneFlags::CLONE_NEWUTS|
        CloneFlags::CLONE_NEWIPC|
        CloneFlags::CLONE_NEWNET|
        CloneFlags::CLONE_NEWPID|
        CloneFlags::CLONE_NEWNS
    )?;

    add_mapping("/proc/self/uid_map", &uid_map)?;
    init_setgroups()?;
    add_mapping("/proc/self/gid_map", &gid_map)?;

    match unsafe{fork()?}{
        ForkResult::Child =>{

            sethostname("container")?;

            println!("{}",getpid());

            // mount(
            //     Some("proc"),"/root/rootfs/proc",
            //     Some("proc"),MsFlags::empty(),
            //     None::<&str>
            // )?;

            mount(
                Some("proc"),"/proc",
                Some("proc"),MsFlags::empty(),
                None::<&str>
            )?;

            // TODO: fix already exist pattern
            // mkdir("/sys/fs/cgroup/cpu/my-container", Mode::S_IRWXU)?;

            // let mut fd = File::create("/sys/fs/cgroup/cpu/my-container/tasks")?;
            // fd.write_all(format!("{}\n",getpid()).as_bytes())?;

            // let mut fd = File::create("/sys/fs/cgroup/cpu/my-container/cpu.cfs_quota_us")?;
            // fd.write_all(b"1000\n").expect("failed edit file");

            //initialize_pivot_root();
        
            let mut p = Command::new("/bin/sh").spawn().expect("sh command failed to start");
            p.wait()?;
            
        },
        ForkResult::Parent{child} =>{
            waitpid(child, None)?;
        }
    };
    Ok(())
}

fn initialize_pivot_root(){
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
}

fn init_setgroups()->Result<()>{
    let mut fd = File::create("/proc/self/setgroups")?;
    fd.write_all(b"deny")?;
    Ok(())
}

fn add_mapping(path: &str,map: &UidGidMap) -> Result<()>{
    let data = String::from(format!("{} {} {}\n",map.container_id,map.host_id,map.size));
    if !data.is_empty(){
        let mut fd = File::create(path)?;
        fd.write_all(data.as_bytes())?;
    }
    Ok(())
}
