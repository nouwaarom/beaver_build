use crate::dependency_graph::{DependencyType};

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
