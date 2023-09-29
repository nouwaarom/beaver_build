use std::process::Command;

pub fn execute_task() {
    let output = Command::new("/usr/bin/gcc").arg("--version").output().expect("Failed to execute command");

    let output_string = String::from_utf8(output.stdout.as_slice().to_vec()).expect("Invalid characters in output");
    println!("GCC stdout: {}", output_string);
}
