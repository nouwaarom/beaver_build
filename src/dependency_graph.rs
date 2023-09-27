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
    dep_type: DependencyType,
    files: Vec<String>,
    // TODOm maybe make these private
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

    pub fn add_dependency() {
        }
}

// TODO:
// - Define struct for node (type, location, dependencies)
// - create a graph datastructure based on an arena


pub fn create_graph() {
    println!("Creating a graph")
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
