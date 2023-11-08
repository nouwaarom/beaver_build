mod configurator;
mod dependency_graph;
mod filesystem;
mod work_pool;
mod graph_walker;
mod builder;

use std::env;
use std::fs;
use std::io::{ErrorKind};
use builder::{Builder}; 
use configurator::{configure_clib_project};
use graph_walker::{GraphWalker, GraphVisitor};
use work_pool::thread_pool_test;
use work_pool::{WorkPool, WorkInstruction};

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

    let mut dependency_graph = configure_clib_project("./data/clib");
    println!("Graph: {}", dependency_graph);

    let mut work_pool = WorkPool::new(4);
    
    //work_pool.schedule_work(WorkInstruction::Compile {
    //    include_dirs: vec![],
    //    output_file: "./data/clib/deps/list/list.c".to_string(),
    //    source_file: "./beaver_build_debug/list.c.o".to_string(),
    //});

    //let result = work_pool.get_results();
    //println!("Result: {:?}", result);

    //return;

    let roots = dependency_graph.get_roots();

    let mut builder = Builder::new(build_directory.to_str().unwrap().to_owned(), &mut work_pool);
    let mut graph_walker = GraphWalker::new(&mut dependency_graph);

    // Build all top levels (executables)
    for root in roots {
        graph_walker.walk(root, &mut builder as &mut dyn GraphVisitor);
        builder.reset();
    }

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
