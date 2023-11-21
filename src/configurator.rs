// Configurator reads project structure and creates a dependency graph
use serde_json::{Result, Value};
use crate::dependency_graph::{DependencyGraph, DependencyOptions};
use crate::filesystem::{DirReader};

/// Loads a project based on a predefined structure and clib package.json files
pub fn configure_clib_project(directory: &str) -> DependencyGraph {
    let mut dependency_graph = DependencyGraph::new();

    let src_dir = format!("{}/src", directory);
    let src_dir_contents = DirReader::new_for(&src_dir);

    // All executable require this interface
    let root_interface = dependency_graph.add_interface("clib_headers", src_dir_contents.get_files_with_extension("h"));

    // Common dir is sort of a dependency dir.
    let common_dir = format!("{}/src/common", directory);
    let common_dir_contents = DirReader::new_for(&common_dir);
    let common_library = dependency_graph.add_library("common_lib", common_dir_contents.get_files_with_extension("c"));
    let common_interface = dependency_graph.add_interface("common_headers", common_dir_contents.get_files_with_extension("h"));
    dependency_graph.add_requirement(common_library, common_interface);

    let mut roots = vec![];
    // All files in the root directories are executables and should be build.
    for executable_src in src_dir_contents.get_files_with_extension("c") {
        let split_by_slash: Vec<_> = executable_src.trim_end_matches(".c").split("/").collect();
        let executable_name = split_by_slash.last().unwrap();
        let executable_lib_name = format!("{}_lib", executable_name);
        let executable_lib = dependency_graph.add_library(&executable_lib_name, vec![executable_src.clone()]);
        let executable = dependency_graph.add_executable(&executable_name, vec![]);
        // TODO, add a more modular way to configure this globally
        let executable_options = DependencyOptions::ExecutableOptions {
            link_libraries: vec!["curl".to_string()],
            link_flags: vec![],
        };
        dependency_graph.set_executable_options(executable, executable_options);
        dependency_graph.add_requirement(executable, executable_lib);
        dependency_graph.add_requirement(executable_lib, root_interface);
        dependency_graph.add_requirement(executable_lib, common_library);

        roots.push(executable);
    }

    // A bit hacky, but need to include the deps folder.
    let deps_dir_dummy = format!("{}/deps/dummy.h", directory);
    let deps_interface = dependency_graph.add_interface("deps_headers", vec![deps_dir_dummy]);
    dependency_graph.add_requirement(common_library, deps_interface);

    let mut unresolved_dependencies = vec![];

    // First pass, loop trough dependencies and create lib and interface targets for them
    let deps_dir_name = format!("{}/deps", directory);
    let dep_dirs = DirReader::get_subdirs(&deps_dir_name);
    for dep_dir in dep_dirs {
        let dep_dir_contents = DirReader::new_for(&dep_dir);

        let dep_name = format!("{}_lib", dep_dir);
        let dep = dependency_graph.add_library(&dep_name, dep_dir_contents.get_files_with_extension("c"));
        dependency_graph.add_requirement(common_library, dep);
        let dep_interface_name = format!("{}_headers", dep_dir);
        let dep_interface = dependency_graph.add_interface(&dep_interface_name, dep_dir_contents.get_files_with_extension("h"));
        dependency_graph.add_requirement(dep, dep_interface);

        let dependencies = get_clib_dependencies(&dep_dir_contents);
        if !dependencies.is_empty() { // Clib headers includes are of the form "dep_name/header.h"
            dependency_graph.add_requirement(dep, deps_interface);
        }

        for dependency in dependencies {
            unresolved_dependencies.push((dep, dependency));
        }
    }

    // Second pass, add interface targets as requirements for libs
    for (origin, dependency) in unresolved_dependencies {
        let dependency_interface_name = format!("{}_headers", dependency);
        match dependency_graph.find_interface(&dependency_interface_name) {
            Some(dependency_interface) => {
                dependency_graph.add_requirement(origin, dependency_interface);
            },
            None => {
                panic!("Missing dependency: {} for {} {:#?}", dependency, dependency_graph.get_name(origin), dependency_graph);
            }
        }
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
            let split_by_slash: Vec<_> = dependency_name.trim_end_matches(".c").trim_end_matches(".h").split("/").collect();
            if split_by_slash.len() == 2 {
                dependency_names.push(split_by_slash.last().unwrap().to_string()); 
            } else {
                dependency_names.push(split_by_slash.first().unwrap().to_string()); 
            }
        }
    }

    return dependency_names;
}
