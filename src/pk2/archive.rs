use std::fs::File;
use std::io::{Read};
use std::path::Path;

use crate::pk2::constants::{HEADER_SIZE};
use crate::pk2::directory::Directory;
use crate::pk2::errors::Error;
use crate::pk2::errors::Error::{IO};
use crate::pk2::header::Header;
use crate::pk2::util::read_block;

/// A structure to access an SRO PK2 archive.
pub struct Archive {
    file: File,
    pub root: Directory
}

impl From<File> for Archive {
    fn from(mut file: File) -> Self {
        let mut header_buf: [u8; HEADER_SIZE] = [0; HEADER_SIZE];
        match file.read(&mut header_buf) {
            Ok(bytes_read) => {
                if bytes_read != HEADER_SIZE {
                    panic!("Header length too short: {}", bytes_read);
                }
                let header = Header::from(header_buf);
                header.verify().unwrap();
                let root = Archive::index(&file).expect("failed to index archive file");
                Archive{file, root}
            }
            Err(err) => panic!("{}", err)
        }
    }
}

impl Archive {
    /// Opens the PK2 archive file at given path and creates an accessible instance
    pub fn open(file_path: &Path) -> Result<Archive, Error> {
        let file_result = File::open(file_path);

        return match file_result {
            Ok(file) => {
                Ok(Archive::from(file))
            },
            Err(err) => Err(IO(err))
        }
    }

    /// Indexes all archive entries recursively
    fn index(file: &File) -> Result<Directory, Error> {
        let entries = read_block(file, HEADER_SIZE as u64)?;
        // 0x2E -> "."
        let mut root_dir_entry = *entries.iter().find(|e| e.is_dir() && e.name[0] == 0x2e).unwrap();
        root_dir_entry.name[0] = 0;
        let mut root_dir = Directory::from(root_dir_entry);
        root_dir.expand(file)?;
        Ok(root_dir)
    }
}