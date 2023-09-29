mod dependency_graph;
mod filesystem;
mod work_pool;

use nix::unistd::{execv, fork, ForkResult, getpid, write};
use nix::sys::wait::waitpid;
use libc::{_exit, STDOUT_FILENO};
use dependency_graph::{DependencyGraph};
use filesystem::read_dir; 
use work_pool::{execute_task};

fn main() {
    println!("Hello, world!");
    let pid = getpid();
    println!("I am the parent and have pid: {}", pid);

    let mut dependency_graph = DependencyGraph::new();
    let root = dependency_graph.add_executable(vec!["a".to_string(), "b".to_string()]);
    dependency_graph.add_interface(vec!["ia".to_string()], root);
    dependency_graph.add_interface(vec!["ib".to_string()], root);
    println!("Graph: {:#?}", dependency_graph);

    let dir_contents = read_dir("./data/clib");
    println!("Dir: {:#?}", dir_contents);

    // TODO, figure out how to put the files in the dependency graph

    /*
    execute_task();

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
    */
}
