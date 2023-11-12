use std::collections::HashMap;
use std::path::{Path};
use crate::dependency_graph::{DependencyGraph, DependencyNode, DependencyType, DependencyOptions, Ref};
use crate::graph_walker::{GraphVisitor};
use crate::work_pool::{WorkPool, WorkInstruction};

/// Caches build information for this target. 
struct LibraryNodeCache {
    is_built: bool,
    objects: Vec<String>,
}

pub struct Builder<'a> {
    work_pool: &'a mut WorkPool,
    build_dir: String,
    headers: Vec<Vec<String>>, // Stack of header files
    objects: Vec<Vec<String>>, // Stack of object files
    library_cache: HashMap<Ref<DependencyNode>, LibraryNodeCache>
}

impl Builder<'_> {
    pub fn new(build_dir: String, work_pool: &mut WorkPool) -> Builder {
        return Builder {
            work_pool,
            build_dir,
            headers: vec![],
            objects: vec![],
            library_cache: HashMap::new(),
        };
    }

    /// Reset the internal state of the builder so it can perform another build.
    pub fn reset(&mut self) {
        self.headers.clear();
        self.objects.clear();
    }
}

impl GraphVisitor for Builder<'_> {
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
                // TODO, implement caching for executables.
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
                // TODO, move to base handler.
                let headers = self.headers.pop().unwrap();

                // TODO, move to cache handler.
                if self.library_cache.contains_key(&node) && self.library_cache[&node].is_built {
                    let objects = self.library_cache[&node].objects.clone();
                    self.objects.last_mut().unwrap().extend(objects);
                    self.headers.last_mut().unwrap().extend(headers);
                    return; // Library is already built, return cached objects.
                }

                // TODO, move to builder handler
                let mut job_ids = vec![];
                let mut objects = vec![];
                let sources = graph.get_files(node);
                for source in sources {
                    let source_path = Path::new(&source);
                    let source_name = source_path.file_name().unwrap().to_str().unwrap().to_owned(); 
                    let object_file = format!("{}/{}.o", self.build_dir, source_name);
                    println!("Compiling {}", source);
                    let compile_instruction = WorkInstruction::Compile {
                        source_file: source.clone(),
                        include_dirs: headers.clone(),
                        output_file: object_file.clone(),
                    };
                    let job_id = self.work_pool.schedule_work(compile_instruction);
                    job_ids.push(job_id);
                    objects.push(object_file);
                }

                let mut target_built = true;
                for job_id in job_ids {
                    match self.work_pool.get_result_blocking(job_id) {
                        Ok(_) => {
                            continue;
                        },
                        Err(output) => {
                            target_built = false;
                            println!("Failed to compile, error: {}", output);
                        }
                    }
                }

                // TODO, mark target as failed so that targets depending on this one
                // will not be build.

                // TODO, move to cache handler
                let node_cache = LibraryNodeCache {
                    is_built: target_built,
                    objects: objects.clone(),
                };
                self.library_cache.insert(node, node_cache);

                // TODO, move to base.
                self.objects.last_mut().unwrap().extend(objects);
                self.headers.last_mut().unwrap().extend(headers);
            },
            DependencyType::EXECUTABLE => {
                let headers = self.headers.pop().unwrap();
                // Step 1, build our own sources.
                let mut own_objects = vec![];
                let sources = graph.get_files(node);
                for source in sources {
                    println!("Compiling executable source: {}", source);
                    let source_path = Path::new(&source);
                    let source_name = source_path.file_name().unwrap().to_str().unwrap().to_owned(); 
                    let object_file = format!("{}/{}.o", self.build_dir, source_name);

                    let compile_instruction = WorkInstruction::Compile {
                        source_file: source.clone(),
                        include_dirs: headers.clone(),
                        output_file: object_file.clone(),
                    };
                    let job_id = self.work_pool.schedule_work(compile_instruction);
                    match self.work_pool.get_result_blocking(job_id) {
                        Ok(_) => {
                            println!("Compiled {}", source);
                        },
                        Err(output) => {
                            // TODO, mark target as failed so that targets depending on this one
                            // will not be build.
                            println!("Failed to compile {}, error: {}", source, output);
                            println!("ABORTING");
                            return;
                        }
                    }
                    // TODO, fix unreliable build if more then one executable file is
                    // specified. 
                    own_objects.push(object_file);
                }

                // Step 2, combine our object files and that of our dependencies
                let mut objects = vec![];
                objects.extend(own_objects);
                for dependency_objects in self.objects.iter() {
                    objects.extend(dependency_objects.clone());
                }

                // TODO, add link flags.
                // Step 3, execute the linker to combine all object files into one executable
                let executable_file = format!("{}/{}", self.build_dir, name);

                let executable_options = graph.get_options(node).unwrap();
                let link_libraries = if let DependencyOptions::ExecutableOptions { link_flags, link_libraries } = executable_options {
                    link_libraries.clone()
                } else {
                    vec![]
                };

                let link_instruction = WorkInstruction::Link {
                    object_files: objects,
                    link_libraries: link_libraries,
                    output_file: executable_file.clone(),
                };
                let job_id = self.work_pool.schedule_work(link_instruction);
                match self.work_pool.get_result_blocking(job_id) {
                    Ok(_) => {
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
