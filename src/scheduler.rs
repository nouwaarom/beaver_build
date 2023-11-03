use crate::graph_walker::{GraphVisitor};

/// Scheduler will walk the dependencygraph and whenever a node is free of dependencies it will
/// mark the node unlocked.
/// The scheduler has a pool of workers that process the unlocked nodes.
/// Upon completion of the processing of an unlocked node its dependents will be updated.
pub struct Scheduler {
    unlocked_nodes: Vec<Ref<DependencyNode>>,
    //builders:  // TODO, create a thread pool for workers. 
}

impl GraphVisitor for Scheduler {
    fn visit_pre_dependency(&mut self, graph: &DependencyGraph, node: Ref<DependencyNode>) {
        // Execute all pre actions in one builder (assumption is that pre actions are inexpensive)
    }

    fn visit_post_dependency(&mut self, graph: &DependencyGraph, node: Ref<DependencyNode>) {
        // Divide over builders.
    } 
}
