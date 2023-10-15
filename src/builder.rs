use crate::dependency_graph::{DependencyGraph, DependencyNode, DependencyType, Ref};
use crate::graph_walker::{GraphVisitor};
use crate::work_pool::execute_compiler;
use std::path::{Path};

#[derive(Default)]
pub struct Builder {
    build_dir: String,
    headers: Vec<Vec<String>>,
    objects: Vec<Vec<String>>,
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
        println!("Pre process: {}", name);
        // TODO, create a stack for headers.
        match graph.get_type(node) {
            DependencyType::INTERFACE => {
            },
            DependencyType::LIBRARY => {
                println!("Lib Pre process: {}", name);
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
                println!("Lib Post process: {}", name);
                let headers = self.headers.pop().unwrap();

                let mut objects = vec![];
                // TODO, generate objects from sources and add them to our list of objects
                let sources = graph.get_files(node);
                for source in sources {
                    let source_path = Path::new(&source);
                    let source_name = source_path.file_name().unwrap().to_str().unwrap().to_owned(); 
                    let object_file = format!("{}/{}.o", self.build_dir, source_name);
                    // TODO, compiler output should be handled
                    match execute_compiler(source.clone(), headers.clone(), object_file.clone()) {
                        Ok(output) => {
                            println!("Compiled {}, output: {}", source, output);
                        },
                        Err(output) => {
                            println!("Failed to compile {}, error: {}", source, output);
                        }
                    }
                    objects.push(object_file);
                }

                println!("Objects: {:#?}", objects);

                // Add our headers to our parents headers.
                self.objects.last_mut().unwrap().extend(objects);
                self.headers.last_mut().unwrap().extend(headers);
            },
            DependencyType::EXECUTABLE => {
                println!("Executable Post process: {}", name);
                let headers = self.headers.pop().unwrap();
                println!("Headers: {:#?}", headers);
            },
        }
    }
}
