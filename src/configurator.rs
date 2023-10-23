// Configurator reads project structure and creates a dependency graph
use serde_json::{Result, Value};
use crate::dependency_graph::{DependencyGraph};
use crate::filesystem::{DirReader};

pub fn configure_clib_project(directory: &str) -> DependencyGraph {
    // TODO, load a project based on clib package.json files.
    let mut dependency_graph = DependencyGraph::new();

    let src_dir = format!("{}/src", directory);
    let src_dir_contents = DirReader::new_for(&src_dir);
    let root = dependency_graph.add_executable("clib", src_dir_contents.get_files_with_extension("c"));
    let root_interface = dependency_graph.add_interface("clib_headers", src_dir_contents.get_files_with_extension("h"), root);

    let common_dir = format!("{}/src/common", directory);
    let common_dir_contents = DirReader::new_for(&common_dir);
    let common = dependency_graph.add_library("common_lib", common_dir_contents.get_files_with_extension("c"), root);
    let common_interface = dependency_graph.add_interface("common_headers", common_dir_contents.get_files_with_extension("h"), common);

    let mut unresolved_dependencies = vec![];

    let dep_dirs = DirReader::get_subdirs("./data/clib/deps");
    for dep_dir in dep_dirs {
        let dep_dir_contents = DirReader::new_for(&dep_dir);

        let dep_name = format!("{}_lib", dep_dir);
        let dep = dependency_graph.add_library(&dep_name, dep_dir_contents.get_files_with_extension("c"), common);
        let dep_interface_name = format!("{}_headers", dep_dir);
        let dep_interface = dependency_graph.add_interface(&dep_interface_name, dep_dir_contents.get_files_with_extension("h"), dep);

        // TODO, add dependencies to the interfaces
        let dependencies = get_clib_dependencies(&dep_dir_contents);
        for dependency in dependencies {
            unresolved_dependencies.push((dep, dependency));
        }
    }

    for (origin, dependency) in unresolved_dependencies {
        // TODO, lookup dependency interface.
        //dependency_graph.get
        // TODO add requirement
    }

    return dependency_graph;
}

fn get_clib_dependencies(dir_contents: &DirReader) -> Vec<String> {
    if !dir_contents.has_file("package.json") {
        return vec![];
    }
    let config_file_content = dir_contents.get_file_contents("package.json");
    let config_file_json: Value = serde_json::from_str(&config_file_content).unwrap();

    let mut dependency_names = vec![];
    let dependencies = &config_file_json["dependencies"];
    if !dependencies.is_null() {
        let dependencies = dependencies.as_object().unwrap();
        for (dependency_name, _version) in dependencies {
            let split_by_slash: Vec<_> = dependency_name.trim_end_matches(".c").split("/").collect();
            if split_by_slash.len() == 2 {
                dependency_names.push(split_by_slash.last().unwrap().to_string()); 
            } else {
                dependency_names.push(split_by_slash.first().unwrap().to_string()); 
            }
        }
    }

    return dependency_names;
}
