use std::process::{Command, ExitStatus};
use itertools::Itertools;

// TODO, create builder objects for compiler.
pub fn execute_compiler(source_file: String, include_dirs: Vec<String>, output_file: String) -> Result<String, String> {
    let mut command_root = Command::new("/usr/bin/gcc");

    let mut command = command_root.arg(source_file);

    // We want to compile only
    command = command.arg("-c");

    for include_dir in include_dirs {
        command = command.arg("-I");
        command = command.arg(include_dir);
     }

    command = command.arg("-o");
    command = command.arg(output_file);

    match command.output() {
        Ok(output) => {
            match output.status.code().unwrap() {
                0 => {
                    let output_string = String::from_utf8(output.stdout.as_slice().to_vec()).expect("Invalid characters in output");
                    return Ok(output_string);
                },
                a => {
                    // Add extra debug information in case of a compile failure
                    let args = command.get_args().into_iter().map(|a| a.to_str().unwrap() ).join(" ");
                    println!("gcc {}", args);

                    let error_string = String::from_utf8(output.stderr.as_slice().to_vec()).expect("Invalid characters in output");
                    return Err(format!("Failed to compile, exit status: {}, error: {}", a, error_string));
                }
            }
        },
        Err(e) => {
            return Err(format!("Failed to compile: {}", e));
        }
    }
}

/// Manages construction of a linker command.
pub struct LinkerBuilder {
}

impl LinkerBuilder {
}

pub fn execute_linker(object_files: Vec<String>, link_libraries: Vec<String>, output_file: String) -> Result<String, String> {
    let mut command_root = Command::new("/usr/bin/gcc");

    for object_file in object_files {
        command_root.arg(object_file);
     }

    for link_library in link_libraries {
        let link_flag = format!("-l{}", link_library);
        command_root.arg(link_flag);
    }

    command_root.arg("-o");
    command_root.arg(output_file);

    match command_root.output() {
        Ok(output) => {
            match output.status.code().unwrap() {
                0 => {
                    let output_string = String::from_utf8(output.stdout.as_slice().to_vec()).expect("Invalid characters in output");
                    return Ok(output_string);
                },
                a => {
                    // Add extra debug information in case of a linking failure
                    let args = command_root.get_args().into_iter().map(|a| a.to_str().unwrap() ).join(" ");
                    println!("gcc {}", args);

                    let error_string = String::from_utf8(output.stderr.as_slice().to_vec()).expect("Invalid characters in output");
                    let error_truncated: String = error_string.chars().take(2000).collect();
                    return Err(format!("Failed to link, exit status: {}, error: {}", a, error_truncated));
                }
            }
        },
        Err(e) => {
            return Err(format!("Failed to compile: {}", e));
        }
    }
}
