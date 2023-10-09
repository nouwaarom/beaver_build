mod dependency_graph;
mod filesystem;
mod work_pool;

use nix::unistd::{execv, fork, ForkResult, getpid, write};
use nix::sys::wait::waitpid;
use libc::{_exit, STDOUT_FILENO};
use dependency_graph::{DependencyGraph};
use filesystem::{DirReader}; 
use work_pool::{execute_task};

fn main() {
    println!("Hello, world!");
    let pid = getpid();
    println!("I am the parent and have pid: {}", pid);

    let mut dependency_graph = DependencyGraph::new();

    let src_dir_contents = DirReader::new_for("./data/clib/src");
    let root = dependency_graph.add_executable(src_dir_contents.get_files_with_extension("c"));
    let root_interface = dependency_graph.add_interface(src_dir_contents.get_files_with_extension("h"), root);

    let common_dir_contents = DirReader::new_for("./data/clib/src/common");
    let common = dependency_graph.add_library(common_dir_contents.get_files_with_extension("c"), root);
    let common_interface = dependency_graph.add_interface(common_dir_contents.get_files_with_extension("h"), common);

    // TODO, read all directories in deps seperately and treat them as separate dependencies
    let dep_dirs = DirReader::get_subdirs("./data/clib/deps");
    for dep_dir in dep_dirs {
        let dep_dir_contents = DirReader::new_for(&dep_dir);
        let dep = dependency_graph.add_library(dep_dir_contents.get_files_with_extension("c"), root);
        let dep_interface = dependency_graph.add_interface(dep_dir_contents.get_files_with_extension("h"), dep);
    }

    println!("Graph: {:#?}", dependency_graph);

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
