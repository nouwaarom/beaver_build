use std::collections::{HashMap};
use crate::dependency_graph::{DependencyNode, DependencyGraph, Ref};
use crate::graph_walker::{GraphVisitor};
use crate::instructor::{Instructor};
use crate::target_data::{TargetData};
use crate::work_pool::{WorkPool};

struct TargetStatus {
    // Target can only be built when all dependencies have been built.
    number_of_unbuilt_dependencies: usize,
    is_built: bool,
    is_scheduled: bool,
    // Stores the job_ids this target is waiting for.
    job_ids: Vec<usize>,
}

impl TargetStatus {
    fn new_unbuilt() -> TargetStatus {
        return TargetStatus {
            number_of_unbuilt_dependencies: 0,
            is_built: false,
            is_scheduled: false,
            job_ids: vec![],
        }
    }
}

/// Scheduler will walk the dependencygraph and whenever a node is free of dependencies it will
/// mark the node unlocked.
/// The scheduler use the workpool to process the unlocked nodes.
/// Upon completion of the processing of an unlocked node its dependents will be updated.
pub struct Scheduler {
    // TODO, replace hashmap by something smarter because we know how many nodes there are.
    target_status_map: HashMap<Ref<DependencyNode>, TargetStatus>,
    target_data_map: HashMap<Ref<DependencyNode>, TargetData>,
    workpool: WorkPool,
    build_dir: String,
}

impl Scheduler {
    pub fn new(number_of_workers: usize, build_dir: String) -> Scheduler {
        return Scheduler {
            target_status_map: HashMap::new(),
            target_data_map: HashMap::new(),
            workpool: WorkPool::new(number_of_workers),
            build_dir,
        };
    }

    pub fn build_all(&mut self, graph: &DependencyGraph) {
        // Step 1, create annotations for all nodes.
        let roots = graph.get_roots();
        for root in roots {
            let name = graph.get_name(root);
            println!("Scheduler starting from {}", name);
            self.visit_node(graph, root);
        }
        
        let mut instructor = Instructor::new(graph, self.build_dir.clone());
        // This might not be optimal, but it is a strategy that is guaranteed to finish and build
        // everything.
        loop {
            // Step 2, loop, while there are unbuilt nodes with no unbuilt dependencies.
            let buildable_nodes: Vec<_> = self.target_status_map.iter().filter(|(node_ref, status)| {
                return !status.is_scheduled && status.number_of_unbuilt_dependencies == 0;
            }).map(|(node_ref, status)| { node_ref.clone() }).collect(); 

            let number_of_unbuilt_nodes = self.target_status_map.iter().filter(|(node_ref, status)| {
                return !status.is_built;
            }).count(); 

            if number_of_unbuilt_nodes == 0 {
                println!("Building done!");
                break;
            }

            for node in buildable_nodes {
                let node_name = graph.get_name(node);
                println!("Scheduling node: {}", node_name);
                // Step 2a) Create work instructions for these nodes.
                instructor.reset();
                instructor.set_node(node);

                let mut dependencies_targetdata = vec![];
                for dependency in graph.get_dependencies(node) {
                    dependencies_targetdata.push(self.target_data_map[&dependency].clone());
                }
                instructor.set_dependency_targetdata(dependencies_targetdata);
                instructor.process();
                // Set target data so that dependants know what data this target produces.
                self.target_data_map.insert(node, instructor.get_targetdata());
                let instructions = instructor.get_instructions();

                // Step 2b) Schedule work instructions for these nodes.
                let mut job_ids = vec![];
                for instruction in instructions {
                    job_ids.push(self.workpool.schedule_work(instruction));
                }

                // If no work is to be performed. Reduce dependency count of dependants.
                if job_ids.is_empty() {
                    self.target_status_map.get_mut(&node).unwrap().is_built = true;
                    for dependant in graph.get_nodes_that_depend_on(node) {
                        let dependant_name = graph.get_name(dependant);
                        println!("Node {} has {} remaining dependencies", dependant_name,
                                 self.target_status_map.get_mut(&dependant).unwrap().number_of_unbuilt_dependencies);
                        self.target_status_map.get_mut(&dependant).unwrap().number_of_unbuilt_dependencies -= 1;
                    }
                }

                self.target_status_map.get_mut(&node).unwrap().job_ids = job_ids;
                self.target_status_map.get_mut(&node).unwrap().is_scheduled = true;
            }

            // Step 2c) Wait for work a complete target in the work results.
            let result = match self.workpool.get_next_result_blocking() {
                Some(r) => r,
                None => { // No job waiting, all work is done.
                    continue;
                }
            };

            let mut node_status_for_result = self.target_status_map.iter_mut().filter(|(_, status)| {
                return status.is_scheduled &&
                        !status.is_built &&
                        status.job_ids.contains(&result.job_id);
            }); 
            let (node, target_data) = match node_status_for_result.next() {
                Some(node_status) => node_status,
                None => { panic!("Found a result without corresponding target"); }
            };

            let index = target_data.job_ids.iter().position(|job_id| *job_id == result.job_id).unwrap();
            target_data.job_ids.remove(index);

            // Step 2d) Decrease the number of unbuilt dependencies for dependents of a finished target. 
            if target_data.job_ids.is_empty() {
                target_data.is_built = true;

                for dependant in graph.get_nodes_that_depend_on(*node) {
                    let dependant_name = graph.get_name(dependant);
                    println!("Node {} has {} remaining dependencies", dependant_name,
                             self.target_status_map.get_mut(&dependant).unwrap().number_of_unbuilt_dependencies);
                    self.target_status_map.get_mut(&dependant).unwrap().number_of_unbuilt_dependencies -= 1;
                }
            }
        }
    }

    fn visit_node(&mut self, graph: &DependencyGraph, node: Ref<DependencyNode>) {
        if self.target_status_map.contains_key(&node) {
            return; // Node is already visited.
        }
        // Create target status
        let mut target_status = TargetStatus::new_unbuilt();
        let dependencies = graph.get_dependencies(node);
        let node_name = graph.get_name(node);
        println!("Node {} has {} dependencies", node_name, dependencies.len());
        target_status.number_of_unbuilt_dependencies = dependencies.len();
        self.target_status_map.insert(node, target_status);

        for dependency in dependencies {
            self.visit_node(graph, dependency);
        }
    }
}
