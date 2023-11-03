use crate::dependency_graph::{DependencyGraph, DependencyNode, Ref};

pub trait GraphVisitor {
    /// Will be executed before all the dependencies of the current node are processed.
    fn visit_pre_dependency(&mut self, graph: &DependencyGraph, node: Ref<DependencyNode>);  
    /// Will be executed after all the dependencies of the current node are processed.
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

    /// Recursively walk a dependency graph.
    pub fn walk(&mut self, root: Ref<DependencyNode>, visitor: &mut dyn GraphVisitor) {
        let name = self.graph.get_name(root);
        println!("Walking from: {}", name);
        self.visit(root, visitor);
    }

    fn visit(&self, node: Ref<DependencyNode>, visitor: &mut dyn GraphVisitor) {
        let dependencies = self.graph.get_dependencies(node); 
    
        visitor.visit_pre_dependency(self.graph, node);
        for dependency in dependencies {
            self.visit(dependency, visitor);
        }

        visitor.visit_post_dependency(self.graph, node);
    }
}
