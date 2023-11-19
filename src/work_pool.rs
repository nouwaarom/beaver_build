use std::cmp;
use std::process::{Command, ExitStatus};
use itertools::Itertools;

use threadpool::ThreadPool;
use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc::channel;

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

pub struct WorkResult {
    job_id: usize,
    result: Result<String, String>,
}

pub struct WorkPool {
    pool: ThreadPool,
    channel_receiver: Receiver<WorkResult>,
    worker: Worker,
    results: Vec<WorkResult>,
    next_job_id: usize,
    number_of_jobs_waiting: usize,
}

impl WorkPool {
    pub fn new(num_workers: usize) -> WorkPool {
        let (tx, rx) = channel();

        return WorkPool {
            pool: ThreadPool::new(num_workers),
            number_of_jobs_waiting: 0,
            next_job_id: 0,
            channel_receiver: rx,
            worker: Worker { channel_sender: tx, job_id: 0 },
            results: vec![],
        };
    }

    /// Schedule the instruction, return a job_id handle with which the result can be fetched.
    pub fn schedule_work(&mut self, instruction: WorkInstruction) -> usize {
        let job_id = self.next_job_id;
        self.worker.job_id = job_id;
        let worker = self.worker.clone(); 
        self.pool.execute(|| {
            worker.execute_work(instruction)
        }); 
        self.number_of_jobs_waiting += 1;
        self.next_job_id += 1;

        return job_id;
    }

    pub fn get_result_blocking(&mut self, job_id: usize) -> Result<String, String> {
        loop {
            self.fetch_results_from_queue();
            match self.results.iter().find_position(|res| res.job_id == job_id) {
                None => {
                    // Result for this job_id not in yet, wait for the next job and check again.
                    let result = self.wait_for_result_from_channel();
                    self.results.push(result);
                },
                Some(res) => {
                    let result = self.results.remove(res.0);
                    return result.result;
                }
            }
        }
    }

    pub fn get_next_result_blocking(&mut self) -> Option<WorkResult> {
        match self.results.pop() {
            Some(result) => { return Some(result) },
            None => {
                if self.number_of_jobs_waiting > 0 {
                    let result = self.wait_for_result_from_channel();
                    return Some(result);
                } else {
                    return None;
                }
            }
        }
    }

    fn fetch_results_from_queue(&mut self) {
        let number_of_results_in_queue = cmp::max(0, (self.number_of_jobs_waiting as i64) - (self.pool.active_count() + self.pool.queued_count()) as i64);
        for _ in 0..number_of_results_in_queue {
            let result = self.wait_for_result_from_channel();
            self.results.push(result);
        }
    }

    /// Blocks waiting for the next result.
    fn wait_for_result_from_channel(&mut self) -> WorkResult {
        match self.channel_receiver.recv() {
            Ok(result) => {
                self.number_of_jobs_waiting -= 1;
                return result;
            }
            Err(_) => {panic!("Could not receive job result from channel"); }
        }
    }
}

#[derive(Clone)]
struct Worker {
    channel_sender: Sender<WorkResult>,
    job_id: usize,
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

        match self.channel_sender.send(WorkResult {
            job_id: self.job_id,
            result
        }) {
            Ok(_) => {
            },
            Err(e) => panic!("Failed to send job result: {}", e),
        }
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

