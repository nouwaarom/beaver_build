use crate::dependency_graph::{DependencyGraph, DependencyNode, Ref};

pub trait GraphVisitor {
    fn visit_pre_dependency(&mut self, graph: &DependencyGraph, node: Ref<DependencyNode>);  
    fn visit_post_dependency(&mut self, graph: &DependencyGraph, node: Ref<DependencyNode>);  
}

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

    pub fn walk(&mut self, root: Ref<DependencyNode>, visitor: &mut dyn GraphVisitor) {
        let name = self.graph.get_name(root);
        println!("Walking from: {}", name);
        self.visit(root, visitor);
    }

    // TODO, pass a visiting class
    fn visit(&self, node: Ref<DependencyNode>, visitor: &mut dyn GraphVisitor) {
        let dependencies = self.graph.get_dependencies(node); 

        visitor.visit_pre_dependency(self.graph, node);
        // TODO, gather a complete list of dependencies that can be used to execute the build step.
        for dependency in dependencies {
            self.visit(dependency, visitor);
        }

        visitor.visit_post_dependency(self.graph, node);
    }
}
