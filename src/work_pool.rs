use std::process::{Command, ExitStatus};
use itertools::Itertools;

use threadpool::ThreadPool;
use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc::channel;

// TODO, create compile instruction and link instruction structures.

pub enum WorkInstruction {
    Link {
        object_files: Vec<String>,
        link_libraries: Vec<String>,
        output_file: String
    },
    Compile {
        source_file: String,
        include_dirs: Vec<String>,
        output_file: String,
    },
}

pub struct WorkPool {
    pool: ThreadPool,
    channel_receiver: Receiver<Result<String, String>>,
    worker: Worker,
}

impl WorkPool {
    pub fn new(num_workers: usize) -> WorkPool {
        let (tx, rx) = channel();

        return WorkPool {
            pool: ThreadPool::new(num_workers),
            channel_receiver: rx,
            worker: Worker { channel_sender: tx },
        };
    }

    pub fn schedule_work(&mut self, instruction: WorkInstruction) {
        let worker = self.worker.clone(); 
        self.pool.execute(|| {
            worker.execute_work(instruction)
        }); 
    }

    pub fn get_results(&self) -> Result<String, String> {
        let result = self.channel_receiver.recv();
        // TODO, handle receive errors.
        return result.unwrap();
    }
}

pub fn thread_pool_test() -> i64 {
    let n_workers = 16;
    let n_jobs = 100000;
    let pool = ThreadPool::new(n_workers);

    let (tx, rx) = channel();
    let mut num_processed = 0;
    let mut sum = 0;
    for i in 0..n_jobs {
        let tx = tx.clone();
        pool.execute(move|| {
            tx.send(i as i64).expect("channel will be there waiting for the pool");
        });

        let num_finished : i64 = (i - num_processed) - (pool.active_count() + pool.queued_count()) as i64;
        if num_finished <= 0 {
            continue;
        }
        num_processed += num_finished;
        println!("{} jobs executed", num_processed);
        
        sum += rx.iter().take(num_finished as usize).fold(0, |a, b| a + b);
    }

    let num_left = (n_jobs - num_processed) as usize;

    println!("{} jobs left", num_left);

    return rx.iter().take(num_left).fold(sum, |a, b| a + b);
}

#[derive(Clone)]
struct Worker {
    channel_sender: Sender<Result<String, String>>,
}

impl Worker {
    fn execute_work(self, instruction: WorkInstruction) {
        let result = match instruction {
            WorkInstruction::Link { object_files, link_libraries, output_file } => {
                self.execute_linker(object_files, link_libraries, output_file)
            }
            WorkInstruction::Compile { source_file, include_dirs, output_file } => {
                self.execute_compiler(source_file, include_dirs, output_file)
            }
        };

        self.channel_sender.send(result);
    }

    fn execute_compiler(&self, source_file: String, include_dirs: Vec<String>, output_file: String) -> Result<String, String> {
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

    fn execute_linker(&self, object_files: Vec<String>, link_libraries: Vec<String>, output_file: String) -> Result<String, String> {
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
}

