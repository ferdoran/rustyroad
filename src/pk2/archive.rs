use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::os::unix::fs::FileExt;
use std::path::Path;
use crate::blowfish::Blowfish;
use crate::pk2::constants::{BLOCK_SIZE, ENTRY_SIZE, HEADER_SIZE, KEY_BYTES, SALT};
use crate::pk2::directory::Directory;
use crate::pk2::entry::Entry;
use crate::pk2::errors::Error;
use crate::pk2::errors::Error::{InvalidBlock, InvalidHeader, IO};
use crate::pk2::header::Header;
use crate::pk2::util::read_block;

pub struct Archive {
    file: File,
    bf: Blowfish,
    pub entries: Vec<Entry>
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
                Archive{file, bf: Blowfish::new(KEY_BYTES, SALT).unwrap(), entries: Vec::new()}
            }
            Err(err) => panic!("{}", err)
        }
    }
}

impl Archive {
    pub fn open(file_path: &Path) -> Result<Archive, Error> {
        let file_result = File::open(file_path);

        return match file_result {
            Ok(file) => {
                Ok(Archive::from(file))
            },
            Err(err) => Err(IO(err))
        }
    }

    pub fn index(&mut self) -> Result<Directory, Error> {
        let entries = read_block(&mut self.file, HEADER_SIZE as u64)?;
        let root_dir_entry = *entries.iter().find(|e| e.is_dir() && e.name[0] == 0x2e).unwrap();
        let mut root_dir = Directory::from(root_dir_entry);
        root_dir.expand(&mut self.file)?;
        self.entries = entries;
        Ok(root_dir)
    }

    fn read_entries(&mut self) -> Result<Vec<Entry>, Error> {
        read_block(&mut self.file, HEADER_SIZE as u64)
    }
}