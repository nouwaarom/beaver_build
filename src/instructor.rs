use std::path::{Path};
use crate::work_pool::{WorkInstruction};
use crate::dependency_graph::{DependencyGraph, DependencyNode, DependencyType, DependencyOptions, Ref};
use crate::target_data::{TargetData};

pub struct Instructor<'a> {
    graph: &'a DependencyGraph,
    build_dir: String,
    node: Option<Ref<DependencyNode>>,
    target_data: Option<TargetData>,
    dependency_data: Vec<TargetData>,
    work_instructions: Vec<WorkInstruction>,
}

impl Instructor<'_> {
    pub fn new(graph: &DependencyGraph, build_dir: String) -> Instructor {
        return Instructor {
            graph,
            build_dir,
            node: None,
            target_data: None,
            dependency_data: vec![],
            work_instructions: vec![],
        }
    }

    pub fn reset(&mut self) {
        self.target_data = None; 
        self.node = None;
        self.work_instructions.clear();
        self.dependency_data.clear();
    }

    pub fn set_node(&mut self, node: Ref<DependencyNode>) {
        self.node = Some(node);
    }

    pub fn set_dependency_targetdata(&mut self, data: Vec<TargetData>) {
        self.dependency_data = data;
    }

    pub fn get_targetdata(&self) -> TargetData {
        match &self.target_data {
            Some(data) => { return data.clone(); }
            None => { panic!("Misuse of instructor"); }
        }
    }

    pub fn get_instructions(&self) -> Vec<WorkInstruction> {
        return self.work_instructions.to_vec();
    }

    pub fn process(&mut self) {
        let node = match self.node {
            Some(node) => node,
            None => { panic!("Misuse of instructor detected"); }
        };

        let name = self.graph.get_name(node);

        match self.graph.get_type(node) {
            DependencyType::INTERFACE => {
                let headers = self.graph.get_files(node);
                let header_path = Path::new(headers.first().unwrap());
                let include_dir = header_path.parent().unwrap();
                self.target_data = Some(TargetData::INTERFACE {
                    include_dirs: vec![include_dir.to_str().unwrap().to_owned()],
                });
            },
            DependencyType::LIBRARY => {
                // Step 1, get required headers.
                let mut include_dirs = vec![];
                for dep_data in self.dependency_data.iter() {
                    match dep_data {
                        TargetData::LIBRARY { include_dirs: lib_include_dirs, .. } => {
                            include_dirs.extend(lib_include_dirs.to_vec());
                        },
                        TargetData::INTERFACE { include_dirs: lib_include_dirs } => {
                            include_dirs.extend(lib_include_dirs.to_vec());
                        },
                        _ => panic!("Unhelpful error message (:"),
                    }
                }

                let mut object_files = vec![];
                let sources = self.graph.get_files(node);
                for source in sources {
                    let source_path = Path::new(&source);
                    let source_name = source_path.file_name().unwrap().to_str().unwrap().to_owned(); 
                    let object_file = format!("{}/{}.o", self.build_dir, source_name);
                    println!("Compiling {}", source);
                    let compile_instruction = WorkInstruction::Compile {
                        source_file: source.clone(),
                        include_dirs: include_dirs.clone(),
                        output_file: object_file.clone(),
                    };
                    object_files.push(object_file);
                    self.work_instructions.push(compile_instruction);
                }

                // TODO, create target data
                self.target_data = Some(TargetData::LIBRARY {
                    include_dirs: include_dirs,
                    object_files: object_files,
                });
            },
            DependencyType::EXECUTABLE => {
                // Step 1, get all object files. 
                let mut object_files = vec![];
                for dep_data in self.dependency_data.iter() {
                    match dep_data {
                        TargetData::LIBRARY {object_files: lib_object_files, ..} => {
                            object_files.extend(lib_object_files.to_vec());
                        },
                        TargetData::INTERFACE { .. } => {},
                        _ => panic!("Unhelpful error message (:"),
                    }
                }

                // TODO, add link flags.
                // Step 2, execute the linker to combine all object files into one executable
                let executable_file = format!("{}/{}", self.build_dir, name);

                let executable_options = self.graph.get_options(node).unwrap();
                let link_libraries = if let DependencyOptions::ExecutableOptions { link_flags, link_libraries } = executable_options {
                    link_libraries.clone()
                } else {
                    vec![]
                };

                let link_instruction = WorkInstruction::Link {
                    object_files: object_files,
                    link_libraries: link_libraries,
                    output_file: executable_file.clone(),
                };
                self.work_instructions.push(link_instruction);
            },
        }
    }
}

