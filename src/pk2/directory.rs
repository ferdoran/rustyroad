use std::collections::HashMap;
use std::fs::{File};
use std::path::{PathBuf};

use crate::pk2::entry::{Entry};
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
}

