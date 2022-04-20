use std::borrow::{Borrow, BorrowMut};
use std::collections::HashMap;
use std::fs::{DirEntry, File};
use std::io::{Read, Seek, SeekFrom};
use std::iter::Map;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use crate::blowfish::Blowfish;
use crate::pk2::constants::{BLOCK_SIZE, ENTRY_SIZE, HEADER_SIZE, KEY_BYTES, SALT};
use crate::pk2::entry::{Entry, EntryType};
use crate::pk2::errors::Error;
use crate::pk2::errors::Error::{InvalidBlock, IO};
use crate::pk2::util::read_block;

#[derive(Clone)]
pub struct Directory {
    entry: Entry,
    pub entries: HashMap<PathBuf, Entry>,
    pub directories: HashMap<PathBuf, Directory>,
}

impl From<Entry> for Directory {
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
    pub fn expand(&mut self, file: &mut File) -> Result<(), Error> {
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
            .filter(|e| e.name[0] != 0x2E)
            .map(|e| Directory::from(*e))
            .map(|d| (d.entry.path_buf(), d))
            .collect();

        dirs.iter_mut()
            .for_each(|(p, mut d)| {
                debug!("extending dir {:?} at offset {}", d.entry.path_buf(), self.entry.position);
                d.expand(file).unwrap()
            });

        self.directories.extend(dirs);

        Ok(())
    }

    pub fn print_entries(&self) {
        self.directories.iter()
            .for_each(|(p, d)| {
                info!("Dir: {:?}", p.as_path());
                d.print_entries();
            });
        self.entries.iter()
            .filter(|(p, e)| e.is_file())
            .for_each(|(p, e)| {
                info!("File: {:?}", p.as_path());
            });
    }
}

