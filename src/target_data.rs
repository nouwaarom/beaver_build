use crate::dependency_graph::{DependencyType};

#[derive(Clone)]
pub enum TargetData {
    INTERFACE {
        include_dirs: Vec<String>,
    },
    LIBRARY {
        include_dirs: Vec<String>,
        object_files: Vec<String>,
    },
    EXECUTABLE {
        executable_file: String,
    },
}
