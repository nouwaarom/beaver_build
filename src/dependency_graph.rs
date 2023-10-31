use std::fmt;
use core::fmt::Debug;
use std::collections::HashMap;
use std::fmt::Formatter;
use std::marker::PhantomData;


#[derive(Debug, Copy, Clone, PartialEq)]
pub enum DependencyType {
    INTERFACE,
    LIBRARY,
    EXECUTABLE,
}

#[derive(Debug, Clone)]
pub enum DependencyOptions {
    ExecutableOptions {
        link_flags: Vec<String>,
        link_libraries: Vec<String>,
    },
}

#[derive(Debug)]
pub struct DependencyNode {
    name: String,
    dep_type: DependencyType,
    files: Vec<String>,
    options: Option<DependencyOptions>, 
    requires: Vec<Ref<DependencyNode>>,
    is_required_by: Vec<Ref<DependencyNode>>,
}

#[derive(Default, Debug)]
pub struct DependencyGraph {
    arena: Vec<DependencyNode>,
    roots: Vec<Ref<DependencyNode>>,
}

impl fmt::Display for DependencyGraph {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Write strictly the first element into the supplied output
        // stream: `f`. Returns `fmt::Result` which indicates whether the
        // operation succeeded or failed. Note that `write!` uses syntax which
        // is very similar to `println!`.
        for root_ref in self.roots.iter() {
            writeln!(f, "Root").unwrap();
            self.print_node(*root_ref, 0, f);
        }

        return Ok(());
    }

}
impl DependencyGraph {
    fn print_node(&self, node_ref: Ref<DependencyNode>, indent: usize, f: &mut fmt::Formatter) {
        let node = self.get_node(node_ref);
        let space = String::from_utf8(vec![b' '; indent*2]).unwrap(); 
        let dep_type = match node.dep_type {
            DependencyType::LIBRARY    => "library:   ",
            DependencyType::INTERFACE  => "interface: ",
            DependencyType::EXECUTABLE => "executable:",
        };
        writeln!(f, "{}{} {}", space, dep_type, node.name).unwrap();
        for dependency in node.requires.iter() {
            self.print_node(*dependency, indent+1, f);
        }
    }
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
            options: None,
            requires: vec![],
            is_required_by: vec![],
        };

        let node_ref = self.add_node(node);

        self.roots.push(node_ref);

        return node_ref;
    }

    pub fn add_interface(&mut self, name: &str, files: Vec<String>) -> Ref<DependencyNode> {
        let node = DependencyNode {
            dep_type: DependencyType::INTERFACE,
            name: name.to_owned(),
            files: files,
            options: None,
            requires: vec![],
            is_required_by: vec![],
        };

        return self.add_node(node);
    }

    pub fn add_library(&mut self, name: &str, files: Vec<String>) -> Ref<DependencyNode> {
        let node = DependencyNode {
            dep_type: DependencyType::LIBRARY,
            name: name.to_owned(),
            files: files,
            options: None,
            requires: vec![],
            is_required_by: vec![],
        };

        return self.add_node(node);
    }

    /// Adds a requirement relation between origin and requires
    /// Store that origin requires requires
    /// And that requires is required by origin
    pub fn add_requirement(&mut self, origin: Ref<DependencyNode>, requires: Ref<DependencyNode>) {
        self.add_requirement_to_node(origin, requires);
        self.add_is_required_by_to_node(requires, origin);
    }

    pub fn set_executable_options(&mut self, executable_ref: Ref<DependencyNode>, options: DependencyOptions) {
        let executable = self.get_node_mut(executable_ref);
        if let DependencyOptions::ExecutableOptions {..} = options {
            executable.options = Some(options);
        } else {
            panic!("Trying to set non-executable options {:#?} to an executable {}!", options, executable.name);
        }
    }

    pub fn get_options(&self, node_ref: Ref<DependencyNode>) -> Option<DependencyOptions> {
        let node = self.get_node(node_ref);

        if let Some(opt) = node.options.as_ref() {
            Some(opt.clone())
        } else {
            None
        }
    }

    pub fn find_interface(&self, name: &str) -> Option<Ref<DependencyNode>> {
        for (index, node) in self.arena.iter().enumerate() {
            if node.dep_type != DependencyType::INTERFACE {
                continue;
            }
            if node.name.ends_with(name) { // Fix this, name is full path spec but dependency name
                                           // is not.
                let node_ref: Ref<DependencyNode> = Ref {
                    idx: index,
                    _type: std::marker::PhantomData,
                };
                return Some(node_ref);
            }
        }

        return None;
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

    fn get_node(&self, node: Ref<DependencyNode>) -> &DependencyNode {
        return &self.arena[node.idx];
    }

    fn get_node_mut(&mut self, node: Ref<DependencyNode>) -> &mut DependencyNode {
        return &mut self.arena[node.idx];
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
            self.add_requirement_to_node(requiree, node_ref);
        }

        return node_ref;
    }
    
    fn add_requirement_to_node(&mut self, origin: Ref<DependencyNode>, requirement: Ref<DependencyNode>) {
        self.arena[origin.idx].requires.push(requirement);
    }

    fn add_is_required_by_to_node(&mut self, origin: Ref<DependencyNode>, is_required_by: Ref<DependencyNode>) {
        self.arena[origin.idx].is_required_by.push(is_required_by);
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
