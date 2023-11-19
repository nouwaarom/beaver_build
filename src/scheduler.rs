use std::collections::{HashMap};
use crate::graph_walker::{GraphVisitor};
use crate::dependency_graph::{DependencyNode, DependencyGraph, Ref};

struct TargetStatus {
    // Target can only be built when all dependencies have been built.
    number_of_unbuilt_dependencies: usize,
    is_built: bool,
}

struct TargetData {
    // TODO, define format, should be dynamic, based on target type
}

/// Scheduler will walk the dependencygraph and whenever a node is free of dependencies it will
/// mark the node unlocked.
/// The scheduler use the workpool to process the unlocked nodes.
/// Upon completion of the processing of an unlocked node its dependents will be updated.
pub struct Scheduler {
    // TODO, replace hashmap by something smarter because we know how many nodes there are.
    target_status_map: HashMap<Ref<DependencyNode>, TargetStatus>,
    target_data_map: HashMap<Ref<DependencyNode>, TargetData>,
}

impl Scheduler {
    pub fn new() -> Scheduler {
        return Scheduler {
            target_status_map: HashMap::new(),
            target_data_map: HashMap::new(),
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
        
        // This might not be optimal, but it is a strategy that is guaranteed to finish and build
        // everything.
        // Step 2, loop, while there are unbuilt nodes with no unbuilt dependencies.
        //  Step 2a) Create work instructions for these nodes.
        //  Step 2b) Schedule work instructions for these nodes.
        //  Step 2c) Wait for work a complete target in the work results, store result in
        //  target_data_map.
        //  Step 2d) Decrease the number of unbuilt dependencies for dependents of the target. 
    }

    fn visit_node(&mut self, graph: &DependencyGraph, node: Ref<DependencyNode>) {
        // TODO, insert to target status map and target data map.
        // TODO, visit all children.
    }
}
