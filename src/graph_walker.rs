use crate::dependency_graph::{DependencyGraph, DependencyNode, Ref};

pub struct GraphWalker<'a> {
    graph: &'a mut DependencyGraph,
}

impl GraphWalker<'_> {
    pub fn new<'b>(graph: &'b mut DependencyGraph) -> GraphWalker<'b> {
       let walker = GraphWalker {
           graph,
       };

       return walker;
    }

    pub fn walk(&mut self, root: Ref<DependencyNode>) {
        let name = self.graph.get_name(root);
        println!("Walking from: {}", name);
        self.visit(root);
    }

    // TODO, pass a visiting class
    fn visit(&self, node: Ref<DependencyNode>) {
        let dependencies = self.graph.get_dependencies(node); 
        // TODO, gather a complete list of dependencies that can be used to execute the build step.
        for dependency in dependencies {
            self.visit(dependency);
        }

        let name = self.graph.get_name(node);
        println!("Ready to process: {}", name);
        // Maybe use: https://docs.rs/enum_dispatch/latest/enum_dispatch/
        // TODO, define a command class that can execute the build step 
    }
}
