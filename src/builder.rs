use crate::dependency_graph::{DependencyGraph, DependencyNode, DependencyType, DependencyOptions, Ref};
use crate::graph_walker::{GraphVisitor};
use crate::work_pool::{execute_compiler, execute_linker};
use std::path::{Path};

#[derive(Default)]
pub struct Builder {
    build_dir: String,
    headers: Vec<Vec<String>>, // Stack of header files
    objects: Vec<Vec<String>>, // Stack of object files
}

impl Builder {
    pub fn new(build_dir: String) -> Builder {
        return Builder {
            build_dir,
            headers: vec![],
            objects: vec![],
        };
    }
}

impl GraphVisitor for Builder {
    fn visit_pre_dependency(&mut self, graph: &DependencyGraph, node: Ref<DependencyNode>) {
        let name = graph.get_name(node);
        match graph.get_type(node) {
            DependencyType::INTERFACE => {
            },
            DependencyType::LIBRARY => {
                self.headers.push(vec![]);
                self.objects.push(vec![]);
            }
            DependencyType::EXECUTABLE => {
                println!("Executable Pre process: {}", name);
                self.headers.push(vec![]);
                self.objects.push(vec![]);
            },
        }
    }

    fn visit_post_dependency(&mut self, graph: &DependencyGraph, node: Ref<DependencyNode>) {
        let name = graph.get_name(node);

        match graph.get_type(node) {
            DependencyType::INTERFACE => {
                let headers = graph.get_files(node);
                let header_path = Path::new(headers.first().unwrap());
                let include_dir = header_path.parent().unwrap();
                self.headers.last_mut().unwrap().push(include_dir.to_str().unwrap().to_owned());
            },
            DependencyType::LIBRARY => {
                let headers = self.headers.pop().unwrap();

                let mut objects = vec![];
                let sources = graph.get_files(node);
                for source in sources {
                    let source_path = Path::new(&source);
                    let source_name = source_path.file_name().unwrap().to_str().unwrap().to_owned(); 
                    let object_file = format!("{}/{}.o", self.build_dir, source_name);
                    match execute_compiler(source.clone(), headers.clone(), object_file.clone()) {
                        Ok(output) => {
                            println!("Compiled {}", source);
                        },
                        Err(output) => {
                            // TODO, mark target as failed so that targets depending on this one
                            // will not be build.
                            println!("Failed to compile {}, error: {}", source, output);
                        }
                    }
                    objects.push(object_file);
                }

                // Add our headers to our parents headers.
                self.objects.last_mut().unwrap().extend(objects);
                self.headers.last_mut().unwrap().extend(headers);
            },
            DependencyType::EXECUTABLE => {
                let headers = self.headers.pop().unwrap();
                // Step 1, build our own sources.
                let mut main_object = "".to_string();
                let mut own_objects = vec![];
                let sources = graph.get_files(node);
                for source in sources {
                    let source_path = Path::new(&source);
                    let source_name = source_path.file_name().unwrap().to_str().unwrap().to_owned(); 
                    let object_file = format!("{}/{}.o", self.build_dir, source_name);
                    match execute_compiler(source.clone(), headers.clone(), object_file.clone()) {
                        Ok(output) => {
                            println!("Compiled {}", source);
                        },
                        Err(output) => {
                            // TODO, mark target as failed so that targets depending on this one
                            // will not be build.
                            println!("Failed to compile {}, error: {}", source, output);
                        }
                    }
                    // TODO, really hacky, define a better way to define the main object.
                    if object_file.ends_with("clib-search.c.o") {
                        main_object = object_file;
                    } else {
                       own_objects.push(object_file);
                    }
                }

                // Step 2, combine our object files and that of our dependencies
                let mut objects = vec![];
                objects.extend(own_objects);
                for dependency_objects in self.objects.iter() {
                    objects.extend(dependency_objects.clone());
                }

                // TODO, figure out how to specify extra libraries and link flags.
                // Step 3, execute the linker to combine all object files into one executable
                let executable_file = format!("{}/{}", self.build_dir, name);

                let executable_options = graph.get_options(node).unwrap();
                let link_libraries = if let DependencyOptions::ExecutableOptions { link_flags, link_libraries } = executable_options {
                    link_libraries.clone()
                } else {
                    vec![]
                };
                match execute_linker(main_object, objects, link_libraries, executable_file.clone()) {
                    Ok(output) => {
                        println!("Linked {}", executable_file);
                    },
                    Err(output) => {
                        // TODO, mark target as failed so that targets depending on this one
                        // will not be build.
                        // TODO, mark build as failed.
                        println!("Failed to link {}, error: {}", executable_file, output);
                    }
                }
            },
        }
    }
}
