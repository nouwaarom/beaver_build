use std::io;
use std::fs::{self, DirEntry};
use std::path::Path;


#[derive(Default)]
pub struct DirReader {
    files: Vec<String>,
}

impl DirReader {
    pub fn new_for(dir: &str) -> DirReader {
        let path = Path::new(dir);
        let mut dir_reader = DirReader::default();
        dir_reader.read_files_in_dir(path).unwrap();

        return dir_reader;
    }

    pub fn new_recursive_for(dir: &str) -> DirReader {
        let path = Path::new(dir);
        let mut dir_reader = DirReader::default();
        dir_reader.read_files_in_dir_recursive(path).unwrap();

        return dir_reader;
    }

    pub fn get_subdirs(dir: &str) -> Vec<String> {
        let path = Path::new(dir);
        let mut dir_reader = DirReader::default();
        dir_reader.read_dirs_in_dir(path).unwrap();

        return dir_reader.files;
    }

    pub fn get_files_with_extension(&self, extension: &str) -> Vec<String> {
        let filtered = self.files.iter().filter(|file| {
            let end = format!(".{}", extension);
            return file.ends_with(&end);
        }).map(|file| { file.clone()}).collect();

        return filtered;
    }

    fn read_files_in_dir_recursive(&mut self, dir: &Path) -> io::Result<()> {
        if dir.is_dir() {
            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    let is_hidden = path.file_name().unwrap().to_str().unwrap().to_owned().starts_with(".");
                    if is_hidden {
                        println!("Ignoring hidden directory: {}", path.to_str().unwrap());
                    } else {
                        self.read_files_in_dir_recursive(&path)?;
                    }
                } else {
                    self.dir_read_closure(&entry);
                }
            }
        }
        Ok(())
    }

    fn read_files_in_dir(&mut self, dir: &Path) -> io::Result<()> {
        if dir.is_dir() {
            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    continue;
                }
                self.dir_read_closure(&entry);
            }
        }
        Ok(())
    }

    fn read_dirs_in_dir(&mut self, dir: &Path) -> io::Result<()> {
        if dir.is_dir() {
            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                if !path.is_dir() {
                    continue;
                }
                self.dir_read_closure(&entry);
            }
        }
        Ok(())
    }

    fn dir_read_closure(&mut self, entry: &DirEntry) {
        self.files.push(entry.path().to_str().unwrap().to_owned());
    }
}
