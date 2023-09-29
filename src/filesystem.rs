use std::io;
use std::fs::{self, DirEntry};
use std::path::Path;


#[derive(Default)]
struct DirReader {
    files: Vec<String>,
}

impl DirReader {
    pub fn visit_dirs(&mut self, dir: &Path) -> io::Result<()> {
        if dir.is_dir() {
            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    let is_hidden = path.file_name().unwrap().to_str().unwrap().to_owned().starts_with(".");
                    if is_hidden {
                        println!("Ignoring hidden directory: {}", path.to_str().unwrap());
                    } else {
                        self.visit_dirs(&path)?;
                    }
                } else {
                    self.dir_read_closure(&entry);
                }
            }
        }
        Ok(())
    }

    pub fn dir_read_closure(&mut self, entry: &DirEntry) {
        self.files.push(entry.path().to_str().unwrap().to_owned());
    }
}

pub fn read_dir(dir: &str) -> Vec<String> {
    let path = Path::new(dir);
    let mut dir_reader = DirReader::default();
    dir_reader.visit_dirs(path).unwrap();

    let mut entries = dir_reader.files;
    // The order in which `read_dir` returns entries is not guaranteed. If reproducible
    // ordering is required the entries should be explicitly sorted.

    entries.sort();

    // The entries have now been sorted by their path.
    entries
}
