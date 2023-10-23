use core::fmt::Debug;
use std::collections::HashMap;
use std::fmt::Formatter;
use std::marker::PhantomData;


#[derive(Debug, Copy, Clone)]
pub enum DependencyType {
    INTERFACE,
    LIBRARY,
    EXECUTABLE,
}

#[derive(Debug)]
pub struct DependencyNode {
    name: String,
    dep_type: DependencyType,
    files: Vec<String>,
    // TODO, maybe make these private
    requires: Vec<Ref<DependencyNode>>,
    is_required_by: Vec<Ref<DependencyNode>>,
}

#[derive(Default, Debug)]
pub struct DependencyGraph {
    arena: Vec<DependencyNode>,
    roots: Vec<Ref<DependencyNode>>,
}

impl DependencyGraph {
    pub fn new() -> DependencyGraph {
        return DependencyGraph::default();
    }

    pub fn add_executable(&mut self, name: &str, files: Vec<String>) -> Ref<DependencyNode> {
        let node = DependencyNode {
            dep_type: DependencyType::EXECUTABLE,
            name: name.to_owned(),
            files: files,
            requires: vec![],
            is_required_by: vec![],
        };

        let node_ref = self.add_node(node);

        self.roots.push(node_ref);

        return node_ref;
    }

    pub fn add_interface(&mut self, name: &str, files: Vec<String>, parent: Ref<DependencyNode>) -> Ref<DependencyNode> {
        let node = DependencyNode {
            dep_type: DependencyType::INTERFACE,
            name: name.to_owned(),
            files: files,
            requires: vec![],
            is_required_by: vec![parent],
        };

        return self.add_node(node);
    }

    pub fn add_library(&mut self, name: &str, files: Vec<String>, parent: Ref<DependencyNode>) -> Ref<DependencyNode> {
        let node = DependencyNode {
            dep_type: DependencyType::LIBRARY,
            name: name.to_owned(),
            files: files,
            requires: vec![],
            is_required_by: vec![parent],
        };

        return self.add_node(node);
    }

    pub fn get_roots(&self) -> Vec<Ref<DependencyNode>> {
        return self.roots.clone();
    }

    pub fn get_name(&self, node: Ref<DependencyNode>) -> String {
        let node = self.get_node(node);
        return node.name.clone();
    }

    pub fn get_dependencies(&self, node: Ref<DependencyNode>) -> Vec<Ref<DependencyNode>> {
        let node = self.get_node(node);
        return node.requires.clone();
    }

    pub fn get_type(&self, node: Ref<DependencyNode>) -> DependencyType {
        let node = self.get_node(node);
        return node.dep_type;
    }

    pub fn get_files(&self, node: Ref<DependencyNode>) -> Vec<String> {
        let node = self.get_node(node);
        return node.files.clone();
    }

    // TODO, find by name

    fn get_node(&self, node: Ref<DependencyNode>) -> &DependencyNode {
        return &self.arena[node.idx];
    }

    fn add_node(&mut self, node: DependencyNode) -> Ref<DependencyNode> {
        let index = self.arena.len();
        let is_required_by = node.is_required_by.clone();
        self.arena.push(node);

        let node_ref: Ref<DependencyNode> = Ref {
            idx: index,
            _type: std::marker::PhantomData,
        };

        // Add child to is_required_by. 
        for requiree in is_required_by {
            self.add_requirement(requiree, node_ref);
        }

        return node_ref;
    }

    fn add_requirement(&mut self, origin: Ref<DependencyNode>, requirement: Ref<DependencyNode>) {
        self.arena[origin.idx].requires.push(requirement);
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
