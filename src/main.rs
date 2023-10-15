mod dependency_graph;
mod filesystem;
mod work_pool;
mod graph_walker;
mod builder;

use std::env;
use std::fs;
use std::io::{ErrorKind};
use nix::unistd::{execv, fork, ForkResult, getpid, write};
use nix::sys::wait::waitpid;
use libc::{_exit, STDOUT_FILENO};
use builder::{Builder}; 
use dependency_graph::{DependencyGraph};
use filesystem::{DirReader}; 
use graph_walker::{GraphWalker, GraphVisitor};

fn main() {
    println!("Hello, world!");

    let mut build_directory = env::current_dir().unwrap();
    build_directory.push("beaver_build_debug");

    println!("Build directory: {}", build_directory.display());
    match fs::create_dir(build_directory.clone()) {
        Ok(_) => {
            println!("Created build directory");
        },
        Err(e) if e.kind() == ErrorKind::AlreadyExists => {
            println!("Build directory already exists");
        }, 
        Err(e) => {
            println!("FATAL: Could not create build directory {}", e);
            return;
        },
    }

    let mut dependency_graph = DependencyGraph::new();

    let src_dir_contents = DirReader::new_for("./data/clib/src");
    let root = dependency_graph.add_executable("clib", src_dir_contents.get_files_with_extension("c"));
    let root_interface = dependency_graph.add_interface("clib_headers", src_dir_contents.get_files_with_extension("h"), root);

    let common_dir_contents = DirReader::new_for("./data/clib/src/common");
    let common = dependency_graph.add_library("common_lib", common_dir_contents.get_files_with_extension("c"), root);
    let common_interface = dependency_graph.add_interface("common_headers", common_dir_contents.get_files_with_extension("h"), common);

    // TODO, read all directories in deps seperately and treat them as separate dependencies
    let dep_dirs = DirReader::get_subdirs("./data/clib/deps");
    for dep_dir in dep_dirs {
        let dep_dir_contents = DirReader::new_for(&dep_dir);
        let dep_name = format!("{}_lib", dep_dir);
        let dep = dependency_graph.add_library(&dep_name, dep_dir_contents.get_files_with_extension("c"), common);
        let dep_interface_name = format!("{}_headers", dep_dir);
        let dep_interface = dependency_graph.add_interface(&dep_interface_name, dep_dir_contents.get_files_with_extension("h"), dep);
    }

    //println!("Graph: {:#?}", dependency_graph);

    let mut graph_walker = GraphWalker::new(&mut dependency_graph);

    let mut builder = Builder::new(build_directory.to_str().unwrap().to_owned());

    graph_walker.walk(root, &mut builder as &mut dyn GraphVisitor);

    /*
    execute_task();
    //let pid = getpid();
    //println!("I am the parent and have pid: {}", pid);

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
