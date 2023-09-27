use std::process::Command;
use nix::unistd::{execv, fork, ForkResult, getpid, write};
use nix::sys::wait::waitpid;
use libc::{_exit, STDOUT_FILENO};
mod dependency_graph;
use dependency_graph::{DependencyGraph};

fn main() {
    println!("Hello, world!");
    let pid = getpid();
    println!("I am the parent and have pid: {}", pid);

    let output = Command::new("/usr/bin/gcc").arg("--version").output().expect("Failed to execute command");

    let output_string = String::from_utf8(output.stdout.as_slice().to_vec()).expect("Invalid characters in output");
    println!("GCC stdout: {}", output_string);

    let mut dependency_graph = DependencyGraph::new();

    match unsafe{fork()} {
        Ok(ForkResult::Parent { child, .. }) => {
            println!("Continuing execution in parent process, new child has pid: {}", child);
            waitpid(child, None).unwrap();
        }
        Ok(ForkResult::Child) => {
            // Unsafe to use `println!` (or `unwrap`) here. See Safety.
            let pid = getpid();
            let message = format!("I'm a new child process and have pid: {}\n", pid);
            write(libc::STDOUT_FILENO, message.as_bytes()).ok();

            unsafe { libc::_exit(0) };
        }
        Err(_) => println!("Fork failed"),
    }
}
