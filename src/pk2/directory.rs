use std::collections::HashMap;
use std::fs::{create_dir_all, File};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

use crate::pk2::entry::Entry;
use crate::pk2::errors::Error;
use crate::pk2::util::read_block;

/// Represents a directory entry in the PK2 archive.
#[derive(Clone)]
pub struct Directory {
    entry: Entry,
    pub entries: HashMap<PathBuf, Entry>,
    pub directories: HashMap<PathBuf, Directory>,
}

impl From<Entry> for Directory {
    /// Creates a [Directory] from a given [Entry].
    fn from(entry: Entry) -> Self {
        if !entry.is_dir() {
            panic!("{:?} is not a directory", entry.path_buf().as_path());
        }
        Directory {
            entry,
            entries: HashMap::new(),
            directories: HashMap::new(),
        }
    }
}

impl Directory {
    /// Expands a directory recursively. Used for indexing.
    pub fn expand(&mut self, file: &File) -> Result<(), Error> {
        let entries = read_block(file, self.entry.position)?;
        let path = self.entry.path_buf().clone();
        let mapped_entries: HashMap<PathBuf, Entry> = entries.iter()
            .filter(|e| !e.is_empty())
            .filter(|e| e.name[0] != 0x2E)
            .map(|entry| {
                let mut cloned_path = path.clone();
                cloned_path.push(entry.path_buf());
                (cloned_path, *entry)
            })
            .collect();
        self.entries.extend(mapped_entries);

        let mut dirs: HashMap<PathBuf, Directory> = entries.iter()
            .filter(|e| e.is_dir())
            .filter(|e| e.name[0] != 0x2E) // 0x2E -> "."
            .map(|e| Directory::from(*e))
            .map(|d| (d.entry.path_buf(), d))
            .collect();

        dirs.iter_mut()
            .for_each(|(_, d)| {
                d.expand(file).unwrap()
            });

        self.directories.extend(dirs);

        Ok(())
    }

    /// Prints out all entries of a directory recursively.
    pub fn print_entries(&self) {
        self.directories.iter()
            .for_each(|(p, d)| {
                info!("Dir: {:?}", p.as_path());
                d.print_entries();
            });
        self.entries.iter()
            .filter(|(_, e)| e.is_file())
            .for_each(|(p, _)| {
                info!("File: {:?}", p.as_path());
            });
    }

    /// Extracts the directory at given location. Requires the file to read it.
    pub fn extract(&self, location: &Path, file: &mut File) {
        if !location.exists() {
            create_dir_all(location).expect(format!("failed to create target directory {:?}", location).as_str());
        }

        self.entries.iter()
            .filter(|(_p, e)| e.is_file())
            .for_each(|(p, e)| {
                file.seek(SeekFrom::Start(e.position)).expect("failed to move to entry's file offset");
                let mut data_buf= vec![0u8; e.size as usize];
                let bytes_read = file.read(&mut data_buf).expect("failed to read file content");
                if bytes_read != e.size as usize {
                    warn!("read {} bytes for entry although it has size {}", bytes_read, e.size);
                }
                let file_name = p.file_name().expect("failed to extract file name from its path");
                let f_loc = location.join(file_name);
                let mut f = File::create(&f_loc).expect(format!("failed to create file {:?}", f_loc).as_str());
                f.write_all(&data_buf).expect("failed to write file data");
            });

        self.directories.iter()
            .for_each(|(p, dir)| {
                let new_loc = location.join(p);
                if !new_loc.exists() {
                    create_dir_all(&new_loc).expect(format!("failed to create target directory {:?}", &new_loc).as_str());
                }
                dir.extract(&new_loc, file);
            });
    }
}

