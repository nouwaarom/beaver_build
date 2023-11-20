mod configurator;
mod dependency_graph;
mod filesystem;
mod instructor;
mod scheduler;
mod target_data;
mod work_pool;
mod graph_walker;
mod builder;

use std::env;
use std::fs;
use std::io::{ErrorKind};
use std::time::{Instant};
use builder::{Builder}; 
use configurator::{configure_clib_project};
use graph_walker::{GraphWalker, GraphVisitor};
use scheduler::{Scheduler};
use work_pool::{WorkPool};

fn main() {
    println!("Beavers will start building!");

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

    // Build all targets sequentially, uncomment to enable
    //let mut work_pool = WorkPool::new(4);
    //let roots = dependency_graph.get_roots();
    //let mut builder = Builder::new(build_directory.to_str().unwrap().to_owned(), &mut work_pool);
    //let mut graph_walker = GraphWalker::new(&mut dependency_graph);
    //// Build all top levels (executables)
    //for root in roots {
    //    graph_walker.walk(root, &mut builder as &mut dyn GraphVisitor);
    //    builder.reset();
    //}

    // Scheduler builds all targets parallel, depending on dependency
    let start = Instant::now();
    let mut scheduler = Scheduler::new(16);
    scheduler.build_all(&dependency_graph);
    let duration = start.elapsed();

    println!("Build time is: {} s", duration.as_secs_f32());
}
