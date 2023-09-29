use core::fmt::Debug;
use std::collections::HashMap;
use std::fmt::Formatter;
use std::marker::PhantomData;


#[derive(Debug)]
enum DependencyType {
    INTERFACE,
    LIB,
    EXECUTABLE,
}

#[derive(Debug)]
pub struct DependencyNode {
    // TODO, add name
    dep_type: DependencyType,
    files: Vec<String>,
    // TODO, maybe make these private
    children: Vec<Ref<DependencyNode>>,
    parent: Option<Ref<DependencyNode>>,
}

#[derive(Default, Debug)]
pub struct DependencyGraph {
    arena: Vec<DependencyNode> 
}

impl DependencyGraph {
    pub fn new() -> DependencyGraph {
        return DependencyGraph::default();
}

    pub fn add_executable(&mut self, files: Vec<String>) -> Ref<DependencyNode> {
        let node = DependencyNode {
            dep_type: DependencyType::EXECUTABLE,
            files: files,
            children: vec!(),
            parent: None,
        };

        return self.add_node(node);
    }

    pub fn add_interface(&mut self, files: Vec<String>, parent: Ref<DependencyNode>) -> Ref<DependencyNode> {
        let node = DependencyNode {
            dep_type: DependencyType::INTERFACE,
            files: files,
            children: vec!(),
            parent: Some(parent),
        };

        return self.add_node(node);
    }

    fn add_node(&mut self, node: DependencyNode) -> Ref<DependencyNode> {
        let index = self.arena.len();
        let parent = node.parent.clone();
        self.arena.push(node);

        let node_ref: Ref<DependencyNode> = Ref {
            idx: index,
            _type: std::marker::PhantomData,
        };

        // Add child to parent. 
        if parent.is_some() {
            self.add_child(parent.unwrap(), node_ref);
        }

        return node_ref;
    }

    fn add_child(&mut self, parent: Ref<DependencyNode>, child: Ref<DependencyNode>) {
        self.arena[parent.idx].children.push(child);
    }
}


pub struct Ref<T> {
    idx: usize,
    _type: std::marker::PhantomData<T>,
}

impl<T> PartialEq for Ref<T> {
    fn eq(&self, other: &Self) -> bool {
        self.idx == other.idx
    }
}
impl<T> Eq for Ref<T> {}

impl<T> std::hash::Hash for Ref<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.idx.hash(state);
    }
}

impl<T> Debug for Ref<T> {
    fn fmt(&self, fmt: &mut Formatter) -> core::fmt::Result {
        write!(fmt, "Ref {{ idx: {} }} ", self.idx)
    }
}

impl<T> Clone for Ref<T> {
    fn clone(&self) -> Self {
        Ref {
            idx: self.idx,
            _type: std::marker::PhantomData,
        }
    }
}

impl<T> Copy for Ref<T> {}
