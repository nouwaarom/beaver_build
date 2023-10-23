use std::process::{Command, ExitStatus};

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

pub fn execute_linker(main_file: String, object_files: Vec<String>, output_file: String) -> Result<String, String> {
    let mut command_root = Command::new("/usr/bin/gcc");

    // We want to link
    let command = command_root.arg("-o");

    let mut command = command_root.arg(main_file);

    for object_file in object_files {
        command = command.arg("-I");
        command = command.arg(object_file);
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
